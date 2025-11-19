use ncollide2d::na::{self, Affine2, Scale2, Translation2};

use crate::{
	common::{Number, Vector},
	render::RenderTarget,
};

const FILE_TEMPLATE: &str = include_str!("./export_template.ipe");
const VERTEX_TEMPLATE: &str = include_str!("./vertex_template.ipe");
const ARC_TEMPLATE: &str = include_str!("./arc_template.ipe");
fn fill_number(original: &str, placeholder: &str, value: Number) -> String {
	original.replace(placeholder, format!("{:.4}", value).as_str())
}
fn fill_str(original: &str, placeholder: &str, value: &str) -> String {
	original.replace(placeholder, value)
}

/// A [RenderTarget] used to export the data to a representation in the .ipe format used by the ['Ipe extensible drawing editor'](https://ipe.otfried.org/).
/// To export render the [crate::model::Data] to an [IpeExporter] and then use the [IpeExporter::to_string] method to get the file.
/// (Since Ipe is a general purpose drawing editor that supports lots of constructs that have nothing to do with this application, there is no meaningful way to import from that format again.)
pub struct IpeExporter {
	content: String,
	pose: Affine2<Number>,
}
impl Default for IpeExporter {
	fn default() -> Self {
		let pose: Affine2<Number> = na::convert(Scale2::new(3.125, -3.125));
		let pose = pose * Translation2::new(0.0, -832.0);
		Self { content: Default::default(), pose }
	}
}
impl IpeExporter {
	fn transform_point(&self, point: Vector) -> Vector {
		self.pose.inverse_transform_point(&point.into()).into()
	}
}
impl ToString for IpeExporter {
	fn to_string(&self) -> String {
		FILE_TEMPLATE.replace("{{{content}}}", &self.content)
	}
}

impl RenderTarget for IpeExporter {
	fn draw_vertex(&mut self, center: crate::common::Vector, color: &str, selected: bool) {
		let center = self.transform_point(center);
		let vertex_string = fill_number(VERTEX_TEMPLATE, "{{{x}}}", center.x);
		let vertex_string = fill_number(&vertex_string, "{{{y}}}", center.y);
		let vertex_string = fill_str(&vertex_string, "{{{color}}}", color);
		self.content.push_str(&vertex_string);
	}
	fn draw_connection_arc(
		&mut self,
		center: crate::common::Vector,
		radius: crate::common::Number,
		rotation: crate::common::Number,
		angle: crate::common::Number,
		color: &str,
		selected: bool,
	) {
		let start = center + radius * Vector::unit_from_angle(rotation);
		let end = center + radius * Vector::unit_from_angle(rotation + angle);
		let (start, end) = match angle < 0.0 {
			true => (start, end),
			false => (end, start),
		};

		let center = self.transform_point(center);
		let start = self.transform_point(start);
		let end = self.transform_point(end);
		let radius = (start - center).length();

		let arc_string = fill_number(ARC_TEMPLATE, "{{{start_x}}}", start.x);
		let arc_string = fill_number(&arc_string, "{{{start_y}}}", start.y);
		let arc_string = fill_number(&arc_string, "{{{radius}}}", radius);
		let arc_string = fill_number(&arc_string, "{{{center_x}}}", center.x);
		let arc_string = fill_number(&arc_string, "{{{center_y}}}", center.y);
		let arc_string = fill_number(&arc_string, "{{{end_x}}}", end.x);
		let arc_string = fill_number(&arc_string, "{{{end_y}}}", end.y);
		let arc_string = fill_str(&arc_string, "{{{color}}}", color);

		self.content.push_str(&arc_string);
	}

	fn draw_grid_line(
		&mut self,
		start: &crate::common::Vector,
		end: &crate::common::Vector,
		width: crate::common::Number,
		color: &str,
	) {
	}
	fn draw_label(&mut self, anchor: crate::common::Vector, text: &str) {}
	fn draw_conflict(&mut self, center: crate::common::Vector, color: &str, selected: bool) {}
	fn draw_connection_invalid(
		&mut self,
		start: crate::common::Vector,
		end: crate::common::Vector,
		selected: bool,
	) {
	}
	fn draw_aid_line(&mut self, from: crate::common::Vector, to: crate::common::Vector) {}
	fn draw_aid_box(&mut self, from: crate::common::Vector, to: crate::common::Vector) {}
	fn clear_region(&mut self, start: &crate::common::Vector, size: &crate::common::Vector) {}
	fn clear(&mut self) {
		self.content = String::new();
	}
	fn reset(&mut self) {}
}
