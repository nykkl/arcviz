use crate::model::LayeredMatrix;

use super::{Connection, VertexId};

#[derive(Clone)]
pub struct Connections {
	matrix: LayeredMatrix<Option<Connection>>,
}

impl Connections {
	pub fn new(size: VertexId) -> Self {
		Self { matrix: LayeredMatrix::new(size) }
	}

	/// Gets the specified connection entry if the entry exists.
	pub fn entry(&self, start_vertex: VertexId, end_vertex: VertexId) -> Result<&Option<Connection>, ()> {
		self.matrix.entry(start_vertex, end_vertex)
	}
	/// Gets the specified connection entry if the entry exists.
	pub fn entry_mut(
		&mut self,
		start_vertex: VertexId,
		end_vertex: VertexId,
	) -> Result<&mut Option<Connection>, ()> {
		self.matrix.entry_mut(start_vertex, end_vertex)
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
		self.matrix.resize(size);
	}
	/// Removes the row and column that contains the specified vertex.
	/// Thus reducing the size of the connection matrix by 1.
	pub fn shrink_by_vertex(&mut self, vertex: VertexId) -> Result<(), ()> {
		self.matrix.remove_row_and_column(vertex)
	}

	pub fn foreach(&self, mut action: impl FnMut(VertexId, VertexId, &Connection)) {
		let action = |start: VertexId, end: VertexId, connection: &Option<Connection>| {
			let Some(connection) = connection else {return;};
			action(start, end, connection);
		};
		self.matrix.foreach(action);
	}

	pub fn fast_iter(&self) -> super::LinearLayeredMatrixIterator<Option<Connection>> {
		self.matrix.linear_iter()
	}
}
