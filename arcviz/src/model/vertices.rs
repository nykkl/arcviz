use crate::render::RenderTarget;

use super::{Vertex, VertexId};

#[derive(Default)]
pub struct Vertices {
	pub items: Vec<Vertex>,
}

impl Vertices {
	pub fn add(&mut self, vertex: Vertex) -> VertexId {
		self.items.push(vertex);
		self.items.len() - 1
	}
	pub fn remove(&mut self, vertex: VertexId) -> Option<Vertex> {
		if vertex >= self.items.len() {
			return None;
		}
		Some(self.items.remove(vertex))
	}
	pub fn len(&self) -> VertexId {
		self.items.len()
	}

	pub fn render(&self, renderer: &mut impl RenderTarget) {
		for vertex in &self.items {
			renderer.draw_vertex(vertex.position.clone(), "green", false);
			if let Some(label) = &vertex.label {
				renderer.draw_label(vertex.position.clone(), label);
			}
		}
	}
	pub fn render_subset(&self, renderer: &mut impl RenderTarget, vertices: &Vec<VertexId>) {
		for id in vertices {
			let Some(vertex) = self.items.get(*id) else { continue };
			renderer.draw_vertex(vertex.position.clone(), "green", false);
		}
	}
}
