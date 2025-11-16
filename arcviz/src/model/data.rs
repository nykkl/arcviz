use std::{str::FromStr, usize};

use serde::{Deserialize, Serialize};

use crate::{
	common::{Bounds, Number, Vector},
	model::{Classes, SizeId},
	render::RenderTarget,
};

use super::{
	Arc, ArcIntersection, Connection, ConnectionOrientation, Connections, Vertex, VertexId, Vertices,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "DataRepresentation", into = "DataRepresentation")]
pub struct Data {
	pub vertices: Vertices,
	connections: Connections,
	pub classes: Classes,
}

impl Default for Data {
	fn default() -> Self {
		let mut this =
			Self { vertices: Vertices::default(), connections: Connections::new(0), classes: Classes::default() };

		this.add_vertex(Vertex::new(Vector::new(50.0, 50.0)));
		this.add_vertex(Vertex::new(Vector::new(50.0, 150.0)));
		this.add_vertex(Vertex::new(Vector::new(150.0, 150.0)));
		this.add_vertex(Vertex::new(Vector::new(150.0, 50.0)));
		this.add_connection(0, 1, ConnectionOrientation::InnerRight, 2);
		this.add_connection(2, 3, ConnectionOrientation::InnerRight, 2);
		this.add_connection(3, 0, ConnectionOrientation::InnerRight, 2);
		this.add_connection(0, 2, ConnectionOrientation::InnerRight, 2);
		this.add_connection(1, 3, ConnectionOrientation::InnerRight, 2);

		this
	}
}

impl Data {
	pub fn vertices_in(&self, region: Bounds) -> Vec<VertexId> {
		self
			.vertices
			.items
			.iter()
			.enumerate()
			.filter(|&(_, v)| region.contains(v.position))
			.map(|(i, _)| i)
			.collect::<Vec<_>>()
	}
	pub fn closest_vertex_to(&self, position: &Vector) -> Option<(VertexId, Number)> {
		// DESIGN: put cutoff distance as param here?
		let (i, d) = self
			.vertices
			.items
			.iter()
			.map(|v| (v.position.clone() - position.clone()).length())
			.enumerate()
			.min_by(|(_, a), (_, b)| (*a).total_cmp(b))?;
		Some((i, d))
	}
	pub fn closest_connection_to(&self, position: &Vector) -> Option<(VertexId, VertexId, Number)> {
		// DESIGN: put cutoff distance as param here?
		let (start, end, distance) = self
			.connections()
			.flat_map(|(start, end, conn)| {
				let distance = match conn {
					ConnectionKind::Arc(arc) => arc.distance_to(*position),
					ConnectionKind::Unreachable => {
						let start = self.vertices.items.get(start)?.position;
						let end = self.vertices.items.get(end)?.position;

						let start_end = end - start;
						let start_point = *position - start;
						let end_point = *position - end;

						let distance = start_point.length().min(end_point.length());

						if start_end.is_ahead(&start_point) && start_end.is_behind(&end_point) {
							distance.min(start_point.rejection_on(&start_end).length())
						} else {
							distance
						}
					},
				};
				Some((start, end, distance))
			})
			.min_by(|(_, _, a), (_, _, b)| (*a).total_cmp(b))?;
		Some((start, end, distance))
	}
	pub fn add_vertex(&mut self, vertex: Vertex) -> VertexId {
		self.vertices.add(vertex);
		self.connections.resize(self.vertices.len());
		return self.vertices.len() - 1;
	}
	pub fn add_connection(
		&mut self,
		start: VertexId,
		end: VertexId,
		orientation: ConnectionOrientation,
		size: SizeId,
	) -> Result<(), ()> {
		let entry = self.connections.entry_mut(start, end)?;
		*entry = Some(Connection::new(orientation, size));
		Ok(())
	}
	pub fn remove_vertex(&mut self, id: VertexId) -> Result<(), ()> {
		self.connections.shrink_by_vertex(id)?;
		self.vertices.remove(id);
		Ok(())
	}
	pub fn remove_connection(&mut self, start: VertexId, end: VertexId) -> Result<(), ()> {
		let Ok(entry) = self.connections.entry_mut(start, end) else { return Err(()) };
		*entry = None;
		Ok(())
	}
	/// Duplicates the specified subgraph.
	///
	/// That includes all the specified vertices and all the connections bewtween exclusively those vertices.
	/// The cloned vertices are appendend at the end.
	///
	/// - vertices: the vertices that define the subgraph (SORTED!)
	///
	/// Returns the index of the first clone.
	pub fn duplicate_subgraph(&mut self, vertices: Vec<VertexId>) -> (VertexId, VertexId) {
		let first = self.vertices.len();

		let clones =
			vertices.iter().map(|id| Some((*id, self.vertices.items.get(*id)?.clone()))).flatten().collect::<Vec<_>>();
		let count = clones.len();
		let old_ids = clones
			.into_iter()
			.map(|(old_id, clone)| {
				self.vertices.add(clone);
				old_id
			})
			.collect::<Vec<_>>();
		self.connections.resize(self.vertices.len());

		for (start_offset, start_old) in old_ids.iter().enumerate() {
			let start_new = first + start_offset;
			for (end_offset, end_old) in old_ids.iter().enumerate() {
				let end_new = first + end_offset;

				let Some(connection) = self.connections.get(*start_old, *end_old) else { continue };
				let clone = connection.clone();
				let Ok(entry) = self.connections.entry_mut(start_new, end_new) else {
					panic!("duplicate_subgraph(): trying to access non-existant connections [ALGORITHMIC ERROR]")
				};
				*entry = Some(clone);
			}
		}

		(first, count)
	}
	pub fn label_vertex(&mut self, vertex: VertexId, label: String) {
		if let Some(vertex) = self.vertices.items.get_mut(vertex) {
			vertex.label = Some(label);
		}
	}
	pub fn connections_subset<'a>(
		&'a self,
		vertices: &'a Vec<VertexId>,
	) -> impl Iterator<Item = (VertexId, VertexId, ConnectionKind)> + 'a {
		let arcs = self
			.connections
			.fast_iter()
			.filter(|(a, b, _)| vertices.binary_search(a).is_ok() && vertices.binary_search(b).is_ok())
			.flat_map(|(start, end, connection)| {
				let conn = match Arc::construct(start, end, &self.vertices, connection.as_ref()?, &self.classes) {
					Ok(arc) => ConnectionKind::Arc(arc),
					Err(()) => ConnectionKind::Unreachable,
				};
				Some((start, end, conn))
			});
		arcs
	}
	fn connections(&self) -> impl Iterator<Item = (VertexId, VertexId, ConnectionKind)> + '_ {
		let arcs = self.connections.fast_iter().flat_map(|(start, end, connection)| {
			let conn = match Arc::construct(start, end, &self.vertices, connection.as_ref()?, &self.classes) {
				Ok(arc) => ConnectionKind::Arc(arc),
				Err(()) => ConnectionKind::Unreachable,
			};
			Some((start, end, conn))
		});
		arcs
	}
	pub fn arcs(&self) -> impl Iterator<Item = (VertexId, VertexId, Arc)> + '_ {
		self.connections().flat_map(|(start, end, connection)| match connection {
			ConnectionKind::Arc(arc) => Some((start, end, arc)),
			ConnectionKind::Unreachable => None,
		})
	}
	pub fn conflicts(
		&self,
		arcs: Vec<(VertexId, VertexId, Arc)>,
	) -> Vec<((VertexId, VertexId), (VertexId, VertexId), Vector)> {
		let mut conflicts = Vec::new();

		for (i, (a, b, arc)) in arcs.iter().enumerate() {
			for (j, (c, d, other)) in arcs.iter().enumerate() {
				if i == j {
					continue;
				}

				let collision_is_connection = |collision_is_right: bool| {
					let Some(connection) = (match (c, d) {
						(c, d) if (c == a && d == b) || (c == b && d == a) => return true,
						(c, d) if c == a || d == a => self.vertices.items.get(*a),
						(c, d) if c == b || d == b => self.vertices.items.get(*b),
						_ => None,
					}) else {
						return false;
					};

					let center_to_other = other.center.clone() - arc.center.clone();
					let center_to_connection = connection.position.clone() - arc.center.clone();
					let connection_is_right = center_to_other.is_right(&center_to_connection);

					let result = !(connection_is_right ^ collision_is_right);
					return result;
				};

				match arc.intersection_with(&other) {
					ArcIntersection::None => (),
					ArcIntersection::One(intersection) => {
						if !(a == c || a == d || b == c || b == d) {
							conflicts.push(((*a, *b), (*c, *d), intersection));
						}
					},
					ArcIntersection::Two(vector1, vector2) => match (vector1, vector2) {
						(None, None) => (),
						(Some(intersection1), Some(intersection2)) => {
							if !collision_is_connection(false) {
								conflicts.push(((*a, *b), (*c, *d), intersection1));
							}
							if !collision_is_connection(true) {
								conflicts.push(((*a, *b), (*c, *d), intersection2));
							}
						},
						(Some(intersection), None) => {
							if !collision_is_connection(false) {
								conflicts.push(((*a, *b), (*c, *d), intersection));
							}
						},
						(None, Some(intersection)) => {
							if !collision_is_connection(true) {
								conflicts.push(((*a, *b), (*c, *d), intersection));
							}
						},
					},
					ArcIntersection::Concentric(_, _) => (),
				};
			}
		}

		return conflicts;
	}
	pub fn edge_mut(&mut self, from: &VertexId, to: &VertexId) -> Option<&mut Connection> {
		self.connections.get_mut(*from, *to)
	}
	pub fn connection(&self, from: &VertexId, to: &VertexId) -> Option<ConnectionKind> {
		let connection = self.connections.get(*from, *to)?;
		let arc = Arc::construct(*from, *to, &self.vertices, connection, &self.classes);
		Some(match arc {
			Ok(arc) => ConnectionKind::Arc(arc),
			Err(()) => ConnectionKind::Unreachable,
		})
	}
	pub fn render_to(&self, renderer: &mut impl RenderTarget) {
		let connections = self.connections().collect::<Vec<_>>();
		connections.iter().for_each(|(start, end, connection)| match connection {
			ConnectionKind::Arc(arc) => renderer.draw_connection_arc(
				arc.center.clone(),
				arc.radius,
				arc.rotation,
				arc.angle,
				self.classes.get_color(self.connections.get(*start, *end).map_or(usize::MAX, |c| c.size)),
				false,
			),
			ConnectionKind::Unreachable => match (self.vertices.items.get(*start), self.vertices.items.get(*end)) {
				(Some(start), Some(end)) => renderer.draw_connection_invalid(start.position, end.position, false),
				_ => (),
			},
		});
		self.vertices.render(renderer);

		let arcs = connections
			.into_iter()
			.flat_map(|(start, end, connection)| match connection {
				ConnectionKind::Arc(arc) => Some((start, end, arc)),
				ConnectionKind::Unreachable => None,
			})
			.collect::<Vec<_>>();

		for conflict in self.conflicts(arcs) {
			renderer.draw_conflict(conflict.2, "orange", false);
		}
		// for (i, (a, b, arc)) in arcs.iter().enumerate() {
		// 	for (j, (c, d, other)) in arcs.iter().enumerate() {
		// 		if i == j {
		// 			continue;
		// 		}
		//
		// 		let collision_is_connection = |collision_is_right: bool| {
		// 			let Some(connection) = (match (c, d) {
		// 				(c, d) if (c == a && d == b) || (c == b && d == a) => return true,
		// 				(c, d) if c == a || d == a => self.vertices.items.get(*a),
		// 				(c, d) if c == b || d == b => self.vertices.items.get(*b),
		// 				_ => None,
		// 			}) else {
		// 				return false;
		// 			};
		//
		// 			let center_to_other = other.center.clone() - arc.center.clone();
		// 			let center_to_connection = connection.position.clone() - arc.center.clone();
		// 			let connection_is_right = center_to_other.is_right(&center_to_connection);
		//
		// 			let result = !(connection_is_right ^ collision_is_right);
		// 			return result;
		// 		};
		//
		// 		match arc.intersection_with(&other) {
		// 			ArcIntersection::None => (),
		// 			ArcIntersection::One(vector) => {
		// 				renderer.draw_conflict(vector, "orange", false);
		// 			},
		// 			ArcIntersection::Two(vector1, vector2) => match (vector1, vector2) {
		// 				(None, None) => (),
		// 				(Some(c1), Some(c2)) => {
		// 					if !collision_is_connection(false) {
		// 						renderer.draw_conflict(c1, "orange", false);
		// 					}
		// 					if !collision_is_connection(true) {
		// 						renderer.draw_conflict(c2, "orange", false);
		// 					}
		// 				},
		// 				(Some(c), None) => {
		// 					if !collision_is_connection(false) {
		// 						renderer.draw_conflict(c, "orange", false);
		// 					}
		// 				},
		// 				(None, Some(c)) => {
		// 					if !collision_is_connection(true) {
		// 						renderer.draw_conflict(c, "orange", false);
		// 					}
		// 				},
		// 			},
		// 			ArcIntersection::Concentric(_, _) => (),
		// 		};
		// 	}
		// }
	}

	/// Renders all specified vertices and all between only! those vertices
	pub fn render_subset_to(&self, renderer: &mut impl RenderTarget, vertices: &Vec<VertexId>) {
		let connections = self.connections_subset(vertices).collect::<Vec<_>>();
		connections.iter().for_each(|(start, end, connection)| match connection {
			ConnectionKind::Arc(arc) => renderer.draw_connection_arc(
				arc.center.clone(),
				arc.radius,
				arc.rotation,
				arc.angle,
				self.classes.get_color(self.connections.get(*start, *end).map_or(usize::MAX, |c| c.size)),
				false,
			),
			ConnectionKind::Unreachable => match (self.vertices.items.get(*start), self.vertices.items.get(*end)) {
				(Some(start), Some(end)) => renderer.draw_connection_invalid(start.position, end.position, false),
				_ => (),
			},
		});
		self.vertices.render_subset(renderer, vertices);

		let arcs = connections
			.into_iter()
			.flat_map(|(start, end, connection)| match connection {
				ConnectionKind::Arc(arc) => Some((start, end, arc)),
				ConnectionKind::Unreachable => None,
			})
			.collect::<Vec<_>>();

		for (i, (a, b, arc)) in arcs.iter().enumerate() {
			for (j, (c, d, other)) in arcs.iter().enumerate() {
				if i == j {
					continue;
				}

				let collision_is_connection = |collision_is_right: bool| {
					let Some(connection) = (match (c, d) {
						(c, d) if (c == a && d == b) || (c == b && d == a) => return true,
						(c, d) if c == a || d == a => self.vertices.items.get(*a),
						(c, d) if c == b || d == b => self.vertices.items.get(*b),
						_ => None,
					}) else {
						return false;
					};

					let center_to_other = other.center.clone() - arc.center.clone();
					let center_to_connection = connection.position.clone() - arc.center.clone();
					let connection_is_right = center_to_other.is_right(&center_to_connection);

					let result = !(connection_is_right ^ collision_is_right);
					return result;
				};

				match arc.intersection_with(&other) {
					ArcIntersection::None => (),
					ArcIntersection::One(vector) => {
						renderer.draw_conflict(vector, "orange", false);
					},
					ArcIntersection::Two(vector1, vector2) => match (vector1, vector2) {
						(None, None) => (),
						(Some(c1), Some(c2)) => {
							if !collision_is_connection(false) {
								renderer.draw_conflict(c1, "orange", false);
							}
							if !collision_is_connection(true) {
								renderer.draw_conflict(c2, "orange", false);
							}
						},
						(Some(c), None) => {
							if !collision_is_connection(false) {
								renderer.draw_conflict(c, "orange", false);
							}
						},
						(None, Some(c)) => {
							if !collision_is_connection(true) {
								renderer.draw_conflict(c, "orange", false);
							}
						},
					},
					ArcIntersection::Concentric(_, _) => (),
				};
			}
		}
	}
	// pub fn render_vertex_to(&self, renderer: &mut impl RenderTarget, vertex: VertexId) {
	// 	if let Some(vertex) = self.vertices.vertices.get(vertex) {
	// 		renderer.draw_point(vertex.position, "green", true);
	// 	}
	// }
	// pub fn render_connection_to(&self, renderer: &mut impl RenderTarget, start: VertexId, end: VertexId) {
	// 	let Some(connenction) = self.connections.get(start, end) else { return };
	// 	let arc = Arc::construct(start, end, &self.vertices, connenction);
	// 	match arc {
	// 		Ok(arc) => renderer.draw_connection_arc(arc.center.clone(), arc.radius, arc.rotation, arc.angle, "purple", true),
	// 		Err(()) => {
	// 			let Some(start) = self.vertices.vertices.get(start) else { return };
	// 			let Some(end) = self.vertices.vertices.get(end) else { return };
	// 			renderer.draw_connection_invalid(start.position, end.position, true)
	// 		},
	// 	}
	// }
}

#[derive(Serialize, Deserialize)]
struct DataRepresentation {
	vertices: Vertices,
	connections: Vec<(VertexId, VertexId, Connection)>,
	sizes: Classes,
}
impl From<DataRepresentation> for Data {
	fn from(value: DataRepresentation) -> Self {
		let mut connections = Connections::new(value.vertices.len());
		for (a, b, connection) in value.connections {
			let Ok(entry) = connections.entry_mut(a, b) else { continue };
			*entry = Some(connection);
		}
		Data { vertices: value.vertices, connections, classes: value.sizes }
	}
}
impl From<Data> for DataRepresentation {
	fn from(value: Data) -> Self {
		let mut connections = Vec::new();
		value.connections.foreach(|a, b, connection| {
			connections.push((a, b, connection.clone()));
		});
		DataRepresentation { vertices: value.vertices, connections, sizes: value.classes }
	}
}

impl ToString for Data {
	fn to_string(&self) -> String {
		let mut result = String::new();
		result.push_str(&self.classes.to_string());
		result.push_str("\n\n");
		result.push_str(&self.vertices.to_string());
		result.push_str("\n\n");
		result.push_str(&self.connections.to_string());
		result
	}
}
impl FromStr for Data {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let sections = s.split("\n\n").collect::<Vec<_>>();
		let (Some(&sizes), Some(&vertices), Some(&connections)) =
			(sections.get(0), sections.get(1), sections.get(2))
		else {
			return Err(());
		};
		let (Ok(sizes), Ok(vertices), Ok(connections)) = (sizes.parse(), vertices.parse(), connections.parse())
		else {
			return Err(());
		};
		let mut this = Self { vertices, connections, classes: sizes };
		this.connections.resize(this.vertices.len());
		Ok(this)
	}
}

pub enum ConnectionKind {
	Arc(Arc),
	Unreachable,
	// BUG: handle self-referential connections (a-a)
}
