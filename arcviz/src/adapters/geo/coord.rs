use geo::Coord;

use crate::common::{Number, Vector};

impl From<Vector> for Coord<Number> {
	fn from(value: Vector) -> Self {
		Coord { x: value.x, y: value.y }
	}
}

impl From<Coord<Number>> for Vector {
	fn from(value: Coord<Number>) -> Self {
		Vector::new(value.x, value.y)
	}
}
