use ncollide2d::na::Affine2;

use crate::common::Number;

pub trait ToCss {
	fn to_css(&self) -> String;
}

impl ToCss for Affine2<Number> {
	fn to_css(&self) -> String {
		format!(
			"matrix({},{},{},{},{},{})",
			self[(0, 0)],
			self[(1, 0)],
			self[(0, 1)],
			self[(1, 1)],
			self[(0, 2)],
			self[(1, 2)],
		)
	}
}
