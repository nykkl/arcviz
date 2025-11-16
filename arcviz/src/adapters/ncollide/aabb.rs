use ncollide2d::bounding_volume::AABB;

use crate::common::{Bounds, Number};

impl From<AABB<Number>> for Bounds {
	fn from(value: AABB<Number>) -> Self {
		Bounds::new(value.mins.into(), (value.maxs - value.mins).into())
	}
}
impl From<Bounds> for AABB<Number> {
	fn from(value: Bounds) -> Self {
		AABB::new(value.start().into(), (value.start() + value.size()).into())
	}
}
