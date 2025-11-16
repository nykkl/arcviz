use std::collections::HashMap;

use js_sys::Array;
use ncollide2d::na::Affine2;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, OffscreenCanvas};
// use image;

use crate::{
	common::{Number, Vector},
	model::Settings,
	render::RenderTarget,
};

pub struct CanvasTarget<'a> {
	resources: &'a Settings,
	cache: HashMap<String, OffscreenCanvas>,
	canvas: &'a mut CanvasRenderingContext2d,
	pose: &'a mut Affine2<Number>,
	size: Vector,
	selection_color: &'a str,
}

impl<'a> CanvasTarget<'a> {
	pub fn new(
		canvas: &'a mut CanvasRenderingContext2d,
		resources: &'a Settings,
		pose: &'a mut Affine2<Number>,
		size: Vector,
	) -> Self {
		Self { resources, cache: HashMap::new(), canvas, pose, size, selection_color: "grey" }
	}

	fn draw_line(&mut self, start: &Vector, end: &Vector, width: Number, color: &str) {
		// begin new path
		self.canvas.begin_path();

		// define path shape
		self.canvas.move_to(start.x, start.y);
		self.canvas.line_to(end.x, end.y);

		// define path settings
		self.canvas.set_line_width(width);
		self.canvas.set_stroke_style_str(color);
		self.canvas.set_line_join("round");
		self.canvas.set_line_cap("round");

		// stroke the current path
		self.canvas.stroke();
	}
}

impl RenderTarget for CanvasTarget<'_> {
	fn draw_grid_line(&mut self, start: &Vector, end: &Vector, width: Number, color: &str) {
		if !self.resources.show_grid {
			return;
		}
		self.draw_line(start, end, width, color);
	}

	fn clear_region(&mut self, start: &Vector, size: &Vector) {
		self.canvas.clear_rect(start.x, start.y, size.x, size.y);
	}

	fn clear(&mut self) {
		let start: Vector = self.pose.transform_point(&Vector::zero().into()).into();
		let size: Vector = self.pose.transform_vector(&self.size.clone().into()).into();
		self.clear_region(&start, &size);
	}

	fn reset(&mut self) {
		let from_parent = self.pose.inverse();
		self.canvas.set_transform(
			from_parent[(0, 0)],
			from_parent[(1, 0)],
			from_parent[(0, 1)],
			from_parent[(1, 1)],
			from_parent[(0, 2)],
			from_parent[(1, 2)],
		);

		self.clear();
	}

	fn draw_vertex(&mut self, position: Vector, color: &str, selected: bool) {
		if selected {
			self.canvas.set_fill_style_str(self.selection_color);
			self.canvas.begin_path();
			self.canvas.ellipse(position.x, position.y, 10.0, 10.0, 0.0, 0.0, 2.0 * std::f64::consts::PI);
			self.canvas.fill();
		}
		self.canvas.set_fill_style_str(color);
		self.canvas.begin_path();
		self.canvas.ellipse(position.x, position.y, 8.0, 8.0, 0.0, 0.0, 2.0 * std::f64::consts::PI);
		self.canvas.fill();
	}

	fn draw_conflict(&mut self, position: Vector, color: &str, selected: bool) {
		if !self.resources.show_conflicts {
			return;
		}

		let mut square = Vector::new_square(4.0);
		let start = position.clone() - square.clone();
		let end = position.clone() + square.clone();
		if selected {
			self.draw_line(&start, &end, 4.0, self.selection_color);
		}
		self.draw_line(&start, &end, 2.0, color);

		square.x = -square.x;
		let start = position.clone() - square.clone();
		let end = position + square.clone();
		if selected {
			self.draw_line(&start, &end, 4.0, self.selection_color);
		}
		self.draw_line(&start, &end, 2.0, color);
	}

	fn draw_connection_invalid(&mut self, start: Vector, end: Vector, selected: bool) {
		// begin new path
		self.canvas.begin_path();
		// define path shape
		self.canvas.move_to(start.x, start.y);
		self.canvas.line_to(end.x, end.y);
		// define path settings
		self.canvas.set_line_join("round");
		self.canvas.set_line_cap("round");
		self
			.canvas
			.set_line_dash(&JsValue::from([15, 10].into_iter().map(|n| JsValue::from(n)).collect::<Array>()));
		if selected {
			self.canvas.set_stroke_style_str(self.selection_color);
			self.canvas.set_line_width(4.0);
			self.canvas.stroke();
		}
		self.canvas.set_stroke_style_str("firebrick");
		self.canvas.set_line_width(2.0);
		// stroke the current path
		self.canvas.stroke();
		self.canvas.set_line_dash(&JsValue::from(Array::new()));
	}

	fn draw_connection_arc(
		&self,
		center: Vector,
		radius: Number,
		rotation: Number,
		angle: Number,
		color: &str,
		selected: bool,
	) {
		// self.canvas.begin_path();
		// self.canvas.set_line_width(1.0);
		// self.canvas.set_stroke_style(&JsString::from("grey"));
		// self.canvas.ellipse_with_anticlockwise(center.x, center.y, radius, radius, rotation, 0.0, 2.0 * std::f64::consts::PI, angle.is_sign_negative());
		// self.canvas.stroke();

		self.canvas.begin_path();
		self.canvas.ellipse_with_anticlockwise(
			center.x,
			center.y,
			radius,
			radius,
			rotation,
			0.0,
			angle,
			angle.is_sign_negative(),
		);
		if selected {
			self.canvas.set_line_width(5.0);
			self.canvas.set_stroke_style_str(self.selection_color);
			self.canvas.stroke();
		}
		self.canvas.set_line_width(3.0);
		self.canvas.set_stroke_style_str(color);
		self.canvas.stroke();
	}

	fn draw_label(&mut self, anchor: Vector, text: &str) {
		if !self.resources.show_labels {
			return;
		}

		self.canvas.set_fill_style_str("white");
		self.canvas.set_font("16px sans");
		let anchor = anchor + Vector::new(8.0, -8.0);
		self.canvas.fill_text(text, anchor.x, anchor.y);
	}

	fn draw_aid_line(&mut self, from: Vector, to: Vector) {
		// begin new path
		self.canvas.begin_path();
		// define path shape
		self.canvas.move_to(from.x, from.y);
		self.canvas.line_to(to.x, to.y);
		// define path settings
		self.canvas.set_line_join("round");
		self.canvas.set_line_cap("round");
		self
			.canvas
			.set_line_dash(&JsValue::from([10, 10].into_iter().map(|n| JsValue::from(n)).collect::<Array>()));
		self.canvas.set_stroke_style_str("grey");
		self.canvas.set_line_width(3.0);
		// stroke the current path
		self.canvas.stroke();
		self.canvas.set_line_dash(&JsValue::from(Array::new()));
	}
	fn draw_aid_box(&mut self, start: Vector, size: Vector) {
		// begin new path
		self.canvas.begin_path();
		// define path shape
		self.canvas.rect(start.x, start.y, size.x, size.y);
		// define path settings
		self.canvas.set_line_join("round");
		self.canvas.set_line_cap("round");
		self.canvas.set_line_dash(&JsValue::from([5, 5].into_iter().map(|n| JsValue::from(n)).collect::<Array>()));
		self.canvas.set_stroke_style_str("grey");
		self.canvas.set_line_width(1.0);
		// stroke the current path
		self.canvas.stroke();
		self.canvas.set_line_dash(&JsValue::from(Array::new()));
	}

	// ISSUE: the printout objects need to know their resources to determine their size
	// but at the moment they only get them when rendering
	// so maybe they should actually get Rc<Resource> instead of &str
	// then they can get their size from there
	//
	// but then how do we deserialize so they all get the same Rc<>?

	// TODO: some way to clear the cache. idk: invalidate() or something. ?: should we reset; should we clear
}
