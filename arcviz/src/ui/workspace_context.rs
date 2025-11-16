use std::{str::FromStr, vec};

use result_or_err::ResultOrErr;
use serde::{Deserialize, Serialize};

use crate::{
	common::{Bounds, Number, Vector},
	model::{ConnectionKind, ConnectionOrientation, Data, Settings, SizeId, Vertex, VertexId},
	render::{RenderTarget, Stage},
	ui::{CrossRenderer, GridRenderer},
};

#[derive(Copy, Clone)]
pub enum Mode {
	/// TODO:
	Hand,
	/// select rect on rdrag
	Select,
	/// add vertices on rclick
	Vertex,
	/// add edged on rclick
	Edge,
	/// vertex or edge mode depending on context
	Edit,
}

pub enum Selection {
	Vertex(VertexId),
	Edge(VertexId, VertexId),
	/// all indices in a certain area (sorted)
	Area(Vec<VertexId>),
}

pub struct WorkspaceContext<S: Stage<Settings>> {
	data: Data,
	pub resources: Settings,
	pub stage: S,
	pub mode: Mode,
	pub orientation: ConnectionOrientation,
	pub size: SizeId,
	pub selection: Option<Selection>,
	pub label: String,
	fine_grid: GridRenderer,
	coarse_grid: GridRenderer,
	cross: CrossRenderer,
	snap_grid_spacing: Number,
}

impl<S: Stage<Settings>> WorkspaceContext<S> {
	pub fn new(stage: S, resources: Settings) -> Self {
		Self {
			data: Data::default(),
			resources,
			stage,
			mode: Mode::Edit,
			orientation: ConnectionOrientation::InnerRight,
			size: 2,
			selection: None,
			label: "".to_owned(),
			fine_grid: GridRenderer::new(50.0, "grey".to_owned(), 0.1),
			coarse_grid: GridRenderer::new(250.0, "grey".to_owned(), 1.0),
			cross: CrossRenderer::new("grey".to_owned(), 2.0),
			snap_grid_spacing: 50.0,
		}
	}

	pub fn conflicts_representation(&self) -> Vec<(String, Vector)> {
		let arcs = self.data.arcs().collect();
		let conflicts = self.data.conflicts(arcs);
		let conflicts = conflicts
			.into_iter()
			.map(|((a, b), (c, d), position)| {
				let a = self.data.vertices.items.get(a).map(|v| v.label.clone()).flatten().unwrap_or(format!("{}", a));
				let b = self.data.vertices.items.get(b).map(|v| v.label.clone()).flatten().unwrap_or(format!("{}", b));
				let c = self.data.vertices.items.get(c).map(|v| v.label.clone()).flatten().unwrap_or(format!("{}", c));
				let d = self.data.vertices.items.get(d).map(|v| v.label.clone()).flatten().unwrap_or(format!("{}", d));
				(format!("({}) ({}) - ({}) ({}) at [{:.2}; {:.2}]", a, b, c, d, position.x, position.y), position)
			})
			.collect::<Vec<_>>();
		return conflicts;
	}

	pub fn adjust_vertex(&mut self, vertex: &VertexId) {
		if self.resources.snap_to_grid {
			let Some(vertex) = self.data.vertices.items.get_mut(*vertex) else { return };
			let quantized = vertex.position / self.snap_grid_spacing;
			let snapped = Vector::new(quantized.x.round(), quantized.y.round());
			vertex.position = self.snap_grid_spacing * snapped;
		}
	}

	pub fn adjust_selection(&mut self) {
		if let Some(Selection::Vertex(vertex)) = self.selection {
			self.adjust_vertex(&vertex);
		}
	}

	pub fn rerender(&mut self) {
		self.stage.target_background(&self.resources).reset();
		self.fine_grid.render_to(&mut self.stage, &self.resources);
		self.coarse_grid.render_to(&mut self.stage, &self.resources);
		self.cross.render_to(&mut self.stage, &self.resources);

		let target = &mut self.stage.target_canvas(&self.resources);

		target.reset();
		self.data.render_to(target);
		// match self.selection {
		// 	None => (),
		// 	Some(Selection::Vertex(id)) => {
		// 		self.data.render_vertex_to(target, id);
		// 	},
		// 	Some(Selection::Edge(start, end)) => {
		// 		self.data.render_connection_to(target, start, end);
		// 	},
		// 	Some(Selection::Area(_)) => todo!(),
		// }
	}

	pub fn add_vertex_at(&mut self, position: Vector) -> VertexId {
		let position = self.stage.pose().transform_point(&position.into()).into();
		let vertex = Vertex::new(position);
		self.add_vertex(vertex)
	}

	fn add_vertex(&mut self, vertex: Vertex) -> VertexId {
		let id = self.data.add_vertex(vertex);
		self.rerender();
		return id;
	}

	fn add_connection(&mut self, start: VertexId, end: VertexId) {
		if start == end {
			return;
		}
		self.data.add_connection(start, end, self.orientation, self.size);
		self.rerender();
	}

	pub fn get_vertex_at(&mut self, position: Vector) -> Option<VertexId> {
		let position = self.stage.pose().transform_point(&position.into()).into();
		let (id, distance) = self.data.closest_vertex_to(&position)?;

		if distance <= 10.0 {
			return Some(id);
		} else {
			return None;
		}
	}

	fn get_connection_at(&mut self, position: Vector) -> Option<(VertexId, VertexId)> {
		let position = self.stage.pose().transform_point(&position.into()).into();
		let (start, end, distance) = self.data.closest_connection_to(&position)?;

		if distance <= 5.0 {
			return Some((start, end));
		} else {
			return None;
		}
	}

	pub fn select_at(&mut self, position: Vector) {
		self.select_vertex_at(position);
		if self.selection.is_none() {
			self.select_connection_at(position);
		}
	}

	pub fn select_in(&mut self, area: Bounds) {
		let start = self.stage.pose().transform_point(&area.start().into()).into();
		let size = self.stage.pose().transform_vector(&area.size().into()).into();

		let indices = self.data.vertices_in(Bounds::new(start, size));
		self.selection = match indices.is_empty() {
			true => None,
			false => Some(Selection::Area(indices)),
		}
	}

	pub fn select_vertex_at(&mut self, position: Vector) {
		self.selection = self.get_vertex_at(position).map(|v| Selection::Vertex(v));
	}

	pub fn select_connection_at(&mut self, position: Vector) {
		self.selection = self.get_connection_at(position).map(|(s, e)| Selection::Edge(s, e));
	}

	pub fn connect_to(&mut self, end: VertexId) {
		let Some(Selection::Vertex(selection)) = self.selection else { return };
		self.data.remove_connection(end, selection); // no double connections (by choice)
		self.add_connection(selection, end);
	}

	pub fn connect_at(&mut self, end: Vector) {
		let Some(Selection::Vertex(start)) = self.selection else { return };
		let Some(end) = self.get_vertex_at(end) else { return };
		self.data.remove_connection(end, start); // no double connections (by choice)
		self.add_connection(start, end);
	}

	pub fn move_selection(&mut self, by: Vector) {
		let Some(selection) = &self.selection else { return };
		let by = self.stage.pose().transform_vector(&by.into()).into();

		let ids = match selection {
			Selection::Vertex(id) => &vec![*id],
			Selection::Edge(a, b) => &vec![*a, *b],
			Selection::Area(items) => items,
		};
		for &id in ids {
			let Some(vertex) = self.data.vertices.items.get_mut(id) else { return };
			vertex.position = vertex.position + by;
		}
		self.rerender();
	}

	pub fn get_selection_bounds(&self) -> Option<Bounds> {
		let bounds = match self.selection.as_ref()? {
			Selection::Vertex(id) => {
				let vertex = self.data.vertices.items.get(*id)?;
				let position = vertex.position;
				Bounds::new(position, Vector::zero())
			},
			Selection::Edge(a, b) => {
				let va = self.data.vertices.items.get(*a)?;
				let vb = self.data.vertices.items.get(*b)?;
				let mut bounds = Into::<Bounds>::into(va.position).combined_with(&vb.position.into());
				if let Some(ConnectionKind::Arc(arc)) = self.data.connection(a, b) {
					bounds = bounds.combined_with(&arc.bounds());
				}
				bounds
			},
			Selection::Area(ids) => {
				let mut bounds: Option<Bounds> = None;
				for &id in ids {
					let Some(vertex) = self.data.vertices.items.get(id) else { continue };
					bounds = Bounds::merged(&bounds, &Some(vertex.position.into()))
				}
				for (_, _, connection) in self.data.connections_subset(ids) {
					let ConnectionKind::Arc(arc) = connection else { continue };
					bounds = Bounds::merged(&bounds, &Some(arc.bounds()));
				}
				bounds?
			},
		}
		.expand(Vector::new_square(8.0)); // WARN: includes padding

		let transform = self.stage.pose().inverse();
		let start = transform.transform_point(&bounds.start().into()).into();
		let size = transform.transform_vector(&bounds.size().into()).into();

		Some(Bounds::new(start, size))
	}

	pub fn delete_selection(&mut self) {
		let Some(selection) = self.selection.take() else { return };
		match selection {
			Selection::Vertex(v) => {
				self.data.remove_vertex(v);
			},
			Selection::Edge(a, b) => {
				self.data.remove_connection(a, b);
			},
			Selection::Area(items) => {
				for v in items.into_iter().rev() {
					self.data.remove_vertex(v);
				}
			},
		}
	}

	pub fn change_selection_class(&mut self, increase: bool) {
		match self.selection {
			Some(Selection::Edge(a, b)) => {
				let mut class = 0;
				if let Some(edge) = self.data.edge_mut(&a, &b) {
					class = edge.size;
				}
				match increase {
					true => class = self.data.classes.next(class),
					false => class = self.data.classes.previous(class),
				}
				if let Some(edge) = self.data.edge_mut(&a, &b) {
					edge.size = class;
				}
			},
			_ => (),
		}
	}

	pub fn duplicate_selection(&mut self) {
		let Some(selection) = self.selection.take() else { return };
		match selection {
			Selection::Vertex(v) => {
				let (start, _) = self.data.duplicate_subgraph(vec![v]);
				self.selection = Some(Selection::Vertex(start));
			},
			Selection::Edge(a, b) => {
				let (start, _) = self.data.duplicate_subgraph(vec![a, b]);
				self.selection = Some(Selection::Edge(start, start + 1));
			},
			Selection::Area(items) => {
				let (start, count) = self.data.duplicate_subgraph(items);
				self.selection = Some(Selection::Area((start..start + count).into_iter().collect()));
			},
		}
		self.rerender(); // DESIGN: rerender or not? make this consistent
	}

	pub fn label_selection(&mut self) {
		let Some(selection) = &self.selection else { return };
		match selection {
			Selection::Vertex(v) => self.data.label_vertex(*v, self.label.clone()),
			Selection::Edge(a, b) => (),
			Selection::Area(items) => {
				for v in items {
					self.data.label_vertex(*v, self.label.clone());
				}
			},
		}
		self.rerender();
	}

	pub fn change_orientation_of_selection(
		&mut self,
		mut transformation: impl FnMut(ConnectionOrientation) -> ConnectionOrientation,
	) {
		let Some(selection) = &self.selection else { return };
		match selection {
			Selection::Vertex(v) => (),
			Selection::Edge(a, b) => {
				if let Some(connection) = self.data.edge_mut(a, b) {
					connection.orientation = transformation(connection.orientation);
				}
			},
			Selection::Area(items) => {
				let connections =
					self.data.connections_subset(&items).map(|(start, end, _)| (start, end)).collect::<Vec<_>>();
				for (start, end) in connections {
					if let Some(connection) = self.data.edge_mut(&start, &end) {
						connection.orientation = transformation(connection.orientation);
					}
				}
			},
		}
		self.rerender();
	}

	pub fn render_selection_to(&self, target: &mut impl RenderTarget) {
		let Some(selection) = &self.selection else { return };
		let ids = match selection {
			Selection::Vertex(id) => &vec![*id],
			Selection::Edge(a, b) => &vec![*a, *b],
			Selection::Area(items) => items,
		};

		self.data.render_subset_to(target, ids);
	}

	pub fn serialize(&self) -> Result<Vec<u8>, ()> {
		let mut buffer = Vec::new();
		let mut serializer = serde_json::Serializer::new(&mut buffer);
		self.data.serialize(&mut serializer).or_err(())?;
		Ok(buffer)
	}

	pub fn load(&mut self, data: &[u8]) -> Result<(), ()> {
		let mut deserializer = serde_json::Deserializer::from_slice(data);
		self.data = Data::deserialize(&mut deserializer).or_err(())?;
		Ok(())
	}

	pub fn load_text(&mut self, data: &[u8]) -> Result<(), ()> {
		let text = String::from_utf8(data.to_vec()).or_err(())?;
		self.data = Data::from_str(&text)?;
		Ok(())
	}

	pub fn to_text(&self) -> Vec<u8> {
		self.data.to_string().as_bytes().to_vec()
	}

	pub fn draw_selection_aid(&mut self, from: Vector, to: Vector) {
		if self.selection.is_some() {
			return;
		}

		let size = to - from;
		let mut target = self.stage.target_hover(&self.resources);
		target.draw_aid_box(from, size);
	}
	pub fn draw_connection_aid(&mut self, to: Vector) {
		let Some(Selection::Vertex(id)) = &self.selection else { return };
		let Some(vertex) = self.data.vertices.items.get(*id) else { return };
		let from = vertex.position;

		let from: Vector = self.stage.pose().inverse_transform_point(&from.into()).into();
		let mut target = self.stage.target_hover(&self.resources);
		target.draw_aid_line(from, to);
	}
	pub fn reset_aids(&mut self) {
		let mut target = self.stage.target_hover(&self.resources);
		target.reset();
	}
}
