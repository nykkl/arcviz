use crate::common::Vector;

#[derive(Clone)]
pub struct Vertex {
	pub position: Vector,
	pub label: Option<String>,
}

impl Vertex {
	pub fn new(position: Vector) -> Self {
		Self { position, label: None }
	}

	pub fn set_label(&mut self, label: String) {
		self.label = Some(label);
	}
	pub fn remove_label(&mut self) {
		self.label = None;
	}
}
