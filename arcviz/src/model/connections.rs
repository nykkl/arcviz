use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::model::{ConnectionOrientation, SizeId};

use super::{Connection, VertexId};

/// Connections is split into layers.
/// This is because the number of layers corresponds to the number of vertices.
/// And this way adding/removing a vertex corresponds to just adding/removing a layer.
/// So only the end of the vector is modified without shifting the remaining data.
///
/// row/column index:   in-layer index:
/// 0 | 0 | 0 | ...     0 | 2 | 4 | ...
/// --/   |   |         --/   |   |
/// 0   1 | 1           0   1 | 3
/// ------/             ------/
/// 0   1   2           0   1   2
///
/// previous layer size:
/// 0 | 1 | 4 | ...
/// --/   |   |
/// 1   1 | 4
/// ------/
/// 4   4   4
///
/// global index = in_layer_index + previous layer size:
/// 0 | 3 | 8 | ...
/// --/   |   |
/// 1   2 | 7
/// ------/
/// 4   5   6
#[derive(Clone, Serialize, Deserialize)]
pub struct Connections {
	items: Vec<Option<Connection>>,
	size: VertexId,
}

impl Connections {
	pub fn new(size: VertexId) -> Self {
		Self { items: vec![None; size.pow(2)], size }
	}
	fn yx_to_global(y: VertexId, x: VertexId) -> usize {
		let (layer_id, in_layer_index) = match (y, x) {
			// y: start; x: end
			(s, e) if e > s => (e, 2 * e - s),
			(s, e) => (s, e),
		};
		let layer_offset = layer_id.pow(2);
		layer_offset + in_layer_index
	}
	// TODO: implement global_to_yx

	/// Gets the specified connection entry if the entry exists.
	pub fn entry(&self, start_vertex: VertexId, end_vertex: VertexId) -> Result<&Option<Connection>, ()> {
		let global_index = Self::yx_to_global(start_vertex, end_vertex);
		self.items.get(global_index).ok_or(())
	}
	/// Gets the specified connection entry if the entry exists.
	pub fn entry_mut(
		&mut self,
		start_vertex: VertexId,
		end_vertex: VertexId,
	) -> Result<&mut Option<Connection>, ()> {
		let global_index = Self::yx_to_global(start_vertex, end_vertex);
		self.items.get_mut(global_index).ok_or(())
	}
	/// Gets the specified connection if the connection exists.
	pub fn get(&self, start_vertex: VertexId, end_vertex: VertexId) -> Option<&Connection> {
		self.entry(start_vertex, end_vertex).ok()?.as_ref()
	}
	/// Gets the specified connection if the connection exists.
	pub fn get_mut(&mut self, start_vertex: VertexId, end_vertex: VertexId) -> Option<&mut Connection> {
		self.entry_mut(start_vertex, end_vertex).ok()?.as_mut()
	}
	/// Adds or removes rows and columns at the end to meet the specified size.
	pub fn resize(&mut self, size: VertexId) {
		self.size = size;
		let target_length = self.size.pow(2);
		self.items.resize(target_length, None);
	}
	/// Removes the row and column that contains the specified vertex.
	/// Thus reducing the size of the connection matrix by 1.
	pub fn shrink_by_vertex(&mut self, vertex: VertexId) -> Result<(), ()> {
		// OPTIMIZE: this is O(n^2) and could be O(n)
		if vertex >= self.size {
			return Err(());
		}
		for layer in (vertex + 1..self.size).rev() {
			self.remove_row_from_layer(vertex, layer);
			self.remove_col_from_layer(vertex, layer);
		}
		self.remove_layer(vertex);
		self.size -= 1;
		Ok(())
	}
	fn remove_layer(&mut self, layer: VertexId) {
		let layer_offset = layer.pow(2);
		let layer_size = 2 * layer + 1;
		self.items.drain(layer_offset..layer_offset + layer_size);
	}
	fn remove_col_from_layer(&mut self, col: VertexId, layer: VertexId) {
		let layer_offset = layer.pow(2);
		let in_layer_index = col;
		let global_index = layer_offset + in_layer_index;
		self.items.remove(global_index);
	}
	fn remove_row_from_layer(&mut self, row: VertexId, layer: VertexId) {
		let layer_offset = layer.pow(2);
		let in_layer_index = 2 * layer - row;
		let global_index = layer_offset + in_layer_index;
		self.items.remove(global_index);
	}

	pub fn foreach(&self, mut action: impl FnMut(VertexId, VertexId, &Connection)) {
		// TODO: check correctness
		let mut layer_id = 0;
		let mut in_layer_index = 0;
		for connection in self.items.iter() {
			let layer_size = 2 * layer_id + 1;
			let (start, end) = match in_layer_index {
				i if i <= layer_id => (layer_id, i),
				i => (layer_size - i - 1, layer_id),
			};

			in_layer_index += 1;
			if in_layer_index >= layer_size {
				layer_id += 1;
				in_layer_index = 0;
			}

			let Some(connection) = connection else {
				continue;
			};
			action(start, end, connection);
		}
	}

	pub fn fast_iter(&self) -> FastConnectionsIterator {
		FastConnectionsIterator::new(&self)
	}
}

#[derive(Clone)]
pub struct FastConnectionsIterator<'a> {
	connections: &'a Connections,
	current: usize,
	layer_id: usize,
	in_layer_index: usize,
}
impl<'a> FastConnectionsIterator<'a> {
	fn new(connections: &'a Connections) -> Self {
		Self { connections, current: 0, layer_id: 0, in_layer_index: 0 }
	}
}
impl<'a> Iterator for FastConnectionsIterator<'a> {
	type Item = (VertexId, VertexId, &'a Option<Connection>);

	fn next(&mut self) -> Option<Self::Item> {
		let connection = self.connections.items.get(self.current)?;

		let layer_size = 2 * self.layer_id + 1;
		let (start, end) = match self.in_layer_index {
			i if i <= self.layer_id => (self.layer_id, i),
			i => (layer_size - i - 1, self.layer_id),
		};

		self.current += 1;
		self.in_layer_index += 1;
		if self.in_layer_index >= layer_size {
			self.layer_id += 1;
			self.in_layer_index = 0;
		}

		return Some((start, end, connection));
	}
}

struct ConnectionRepresentation {
	pub start: VertexId,
	pub end: VertexId,
	pub orientation: ConnectionOrientation,
	pub size: SizeId,
}

impl ToString for ConnectionRepresentation {
	fn to_string(&self) -> String {
		let orientation = match &self.orientation {
			ConnectionOrientation::InnerRight => "right",
			ConnectionOrientation::InnerLeft => "left",
			ConnectionOrientation::OuterRight => "Right",
			ConnectionOrientation::OuterLeft => "Left",
		};
		format!("{} {} {} {}", self.start + 1, self.end + 1, orientation, self.size + 1)
	}
}
impl FromStr for ConnectionRepresentation {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts = s.split_whitespace().collect::<Vec<_>>();
		let (Some(&start), Some(&end), Some(&orientation), Some(&size)) =
			(parts.get(0), parts.get(1), parts.get(2), parts.get(3))
		else {
			return Err(());
		};
		let (Ok(start), Ok(end), orientation, Ok(size)) =
			(start.parse::<VertexId>(), end.parse::<VertexId>(), orientation, size.parse::<SizeId>())
		else {
			return Err(());
		};
		let orientation = match orientation {
			"right" => ConnectionOrientation::InnerRight,
			"left" => ConnectionOrientation::InnerLeft,
			"Right" => ConnectionOrientation::OuterRight,
			"Left" => ConnectionOrientation::OuterLeft,
			_ => return Err(()),
		};
		Ok(Self { start: start - 1, end: end - 1, orientation, size: size - 1 })
	}
}

impl ToString for Connections {
	fn to_string(&self) -> String {
		self
			.fast_iter()
			.map(|(start, end, connection)| {
				Some(ConnectionRepresentation {
					start,
					end,
					orientation: connection.as_ref()?.orientation,
					size: connection.as_ref()?.size,
				})
			})
			.flatten()
			.map(|c| c.to_string())
			.collect::<Vec<_>>()
			.join("\n")
	}
}
impl FromStr for Connections {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut size = 0;
		let items: Vec<ConnectionRepresentation> = s
			.lines()
			.map(|line| {
				let connection = ConnectionRepresentation::from_str(line);
				if let Ok(connection) = &connection {
					size = size.max(connection.start + 1);
					size = size.max(connection.end + 1);
				}
				connection
			})
			.collect::<Result<_, _>>()?;
		let mut connections = Connections::new(size);
		for item in items {
			if let Ok(entry) = connections.entry_mut(item.start, item.end) {
				let connection = Connection { orientation: item.orientation, size: item.size };
				*entry = Some(connection);
			}
		}
		Ok(connections)
	}
}
