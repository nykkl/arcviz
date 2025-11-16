use crate::{
	common::{Bounds, Number, Vector},
	model::Settings,
	render::{RenderTarget, Stage},
};

pub struct GridRenderer {
	spacing: Number,
	color: String,
	thickness: Number,
}
impl GridRenderer {
	pub fn new(spacing: Number, color: String, thickness: Number) -> Self {
		Self { spacing, color, thickness }
	}
	pub fn render_to(&self, stage: &mut impl Stage<Settings>, settings: &Settings) {
		let start: Vector = stage.pose().transform_point(&Vector::zero().into()).into();
		let end: Vector = stage.pose().transform_point(&stage.size().into()).into();
		let bounds = Bounds::new_with_end(start, end);

		let quantized_start = start / self.spacing;
		let quantized_end = end / self.spacing;
		let (xmin, ymin) = (quantized_start.x.ceil() as isize, quantized_start.y.ceil() as isize);
		let (xmax, ymax) = (quantized_end.x.ceil() as isize, quantized_end.y.ceil() as isize);

		let mut target = stage.target_background(settings);

		for x in xmin..xmax {
			self.draw_vertical_line_at(x as Number * self.spacing, &bounds, &mut target);
		}
		for y in ymin..ymax {
			self.draw_horizontal_line_at(y as Number * self.spacing, &bounds, &mut target);
		}
	}
	fn draw_vertical_line_at(&self, x: Number, bounds: &Bounds, target: &mut impl RenderTarget) {
		target.draw_grid_line(
			&Vector::new(x, bounds.start().y),
			&Vector::new(x, bounds.end().y),
			self.thickness,
			&self.color,
		);
	}
	fn draw_horizontal_line_at(&self, y: Number, bounds: &Bounds, target: &mut impl RenderTarget) {
		target.draw_grid_line(
			&Vector::new(bounds.start().x, y),
			&Vector::new(bounds.end().x, y),
			self.thickness,
			&self.color,
		);
	}
}

pub struct CrossRenderer {
	color: String,
	thickness: Number,
}
impl CrossRenderer {
	pub fn new(color: String, thickness: Number) -> Self {
		Self { color, thickness }
	}
	pub fn render_to(&self, stage: &mut impl Stage<Settings>, settings: &Settings) {
		let start: Vector = stage.pose().transform_point(&Vector::zero().into()).into();
		let end: Vector = stage.pose().transform_point(&stage.size().into()).into();
		let bounds = Bounds::new_with_end(start, end);

		let mut target = stage.target_background(settings);

		self.draw_vertical_line_at(0.0, &bounds, &mut target);
		self.draw_horizontal_line_at(0.0, &bounds, &mut target);
	}
	fn draw_vertical_line_at(&self, x: Number, bounds: &Bounds, target: &mut impl RenderTarget) {
		target.draw_grid_line(
			&Vector::new(x, bounds.start().y),
			&Vector::new(x, bounds.end().y),
			self.thickness,
			&self.color,
		);
	}
	fn draw_horizontal_line_at(&self, y: Number, bounds: &Bounds, target: &mut impl RenderTarget) {
		target.draw_grid_line(
			&Vector::new(bounds.start().x, y),
			&Vector::new(bounds.end().x, y),
			self.thickness,
			&self.color,
		);
	}
}
