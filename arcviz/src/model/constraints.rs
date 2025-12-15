use crate::model::LayeredMatrix;

use super::VertexId;

#[derive(Clone)]
pub struct Constraints {
	matrix: LayeredMatrix<Option<bool>>,
}

impl Constraints {
	pub fn new(size: VertexId) -> Self {
		Self { matrix: LayeredMatrix::new(size) }
	}

	/// The number of connections to be constrained.
	pub fn size(&self) -> usize {
		self.matrix.size()
	}
	/// The number of entries in the constraints matrix.
	pub fn len(&self) -> usize {
		self.matrix.len()
	}
	/// Gets the specified constraint entry if the entry exists.
	pub fn entry(&self, start_vertex: VertexId, end_vertex: VertexId) -> Result<&Option<bool>, ()> {
		self.matrix.entry(start_vertex, end_vertex)
	}
	/// Gets the specified constraint entry if the entry exists.
	pub fn entry_mut(
		&mut self,
		start_vertex: VertexId,
		end_vertex: VertexId,
	) -> Result<&mut Option<bool>, ()> {
		self.matrix.entry_mut(start_vertex, end_vertex)
	}
	/// Gets the specified constraint if the constraint exists.
	pub fn get(&self, start_vertex: VertexId, end_vertex: VertexId) -> Option<&bool> {
		self.entry(start_vertex, end_vertex).ok()?.as_ref()
	}
	/// Gets the specified constraint if the constraint exists.
	pub fn get_mut(&mut self, start_vertex: VertexId, end_vertex: VertexId) -> Option<&mut bool> {
		self.entry_mut(start_vertex, end_vertex).ok()?.as_mut()
	}
	/// Adds or removes rows and columns at the end to meet the specified size.
	pub fn resize(&mut self, size: VertexId) {
		self.matrix.resize(size);
	}
	/// Removes the row and column that contains the specified vertex.
	/// Thus reducing the size of the constraint matrix by 1.
	pub fn shrink_by_vertex(&mut self, vertex: VertexId) -> Result<(), ()> {
		self.matrix.remove_row_and_column(vertex)
	}

	pub fn foreach(&self, mut action: impl FnMut(VertexId, VertexId, &bool)) {
		let action = |start: VertexId, end: VertexId, constraint: &Option<bool>| {
			let Some(constraint) = constraint else {return;};
			action(start, end, constraint);
		};
		self.matrix.foreach(action);
	}

	pub fn fast_iter(&self) -> super::LinearLayeredMatrixIterator<Option<bool>> {
		self.matrix.linear_iter()
	}
}
