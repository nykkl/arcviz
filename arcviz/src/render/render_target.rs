use crate::common::{Number, Vector};

// use super::resources::ResourceHandle;

/// Something that provides certain functionality for renedering objects to it.
pub trait RenderTarget {
	fn draw_grid_line(&mut self, start: &Vector, end: &Vector, width: Number, color: &str);
	fn draw_label(&mut self, anchor: Vector, text: &str);
	fn draw_vertex(&mut self, center: Vector, color: &str, selected: bool);
	fn draw_conflict(&mut self, center: Vector, color: &str, selected: bool);
	fn draw_connection_invalid(&mut self, start: Vector, end: Vector, selected: bool);
	fn draw_connection_arc(
		&self,
		center: Vector,
		radius: Number,
		rotation: Number,
		angle: Number,
		color: &str,
		selected: bool,
	);
	fn draw_aid_line(&mut self, from: Vector, to: Vector);
	fn draw_aid_box(&mut self, from: Vector, to: Vector);

	fn clear_region(&mut self, start: &Vector, size: &Vector);
	fn clear(&mut self);
	fn reset(&mut self);
}
