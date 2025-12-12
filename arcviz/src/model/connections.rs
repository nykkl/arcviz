use wasm_bindgen::JsValue;
use web_sys::console;

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
		todo!("a")
	}
	/// Gets the specified connection entry if the entry exists.
	pub fn entry_mut(
		&mut self,
		start_vertex: VertexId,
		end_vertex: VertexId,
	) -> Result<&mut Option<Connection>, ()> {
		todo!("b")
	}
	/// Gets the specified connection if the connection exists.
	pub fn get(&self, start_vertex: VertexId, end_vertex: VertexId) -> Option<&Connection> {
		todo!("c")
	}
	/// Gets the specified connection if the connection exists.
	pub fn get_mut(&mut self, start_vertex: VertexId, end_vertex: VertexId) -> Option<&mut Connection> {
		todo!("d")
	}
	/// Adds or removes rows and columns at the end to meet the specified size.
	pub fn resize(&mut self, size: VertexId) {
		console::log_1(&JsValue::from("A"));
		self.matrix.resize(size);
		console::log_1(&JsValue::from("B"));
	}
	/// Removes the row and column that contains the specified vertex.
	/// Thus reducing the size of the connection matrix by 1.
	pub fn shrink_by_vertex(&mut self, vertex: VertexId) -> Result<(), ()> {
		todo!("f")
	}

	pub fn foreach(&self, mut action: impl FnMut(VertexId, VertexId, &Connection)) {
		todo!("g")
	}

	pub fn fast_iter(&self) -> super::LinearLayeredMatrixIterator<Option<Connection>> {
		self.matrix.linear_iter()
	}
}
