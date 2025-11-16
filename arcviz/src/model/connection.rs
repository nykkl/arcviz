use serde::{Deserialize, Serialize};

use crate::model::SizeId;

/// With which arc to form the connection.
///
/// Left and right here refer to whether the connections curves left(+) or right(-) in a standard cartesian coordinate system.
/// On a screen this is likely reversed.
/// Meaning left or right would determine whether the arc is on the left or right of the straight line connecting the points.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ConnectionOrientation {
	InnerRight,
	InnerLeft,
	OuterRight,
	OuterLeft,
}
impl ConnectionOrientation {
	/// Returns the inverse of the given orientation.
	///
	/// The inverse for the purposes of this functions is considered that orientation,
	/// which would complete the circle. I.e. that which yields the circle arc that is currently missing.
	pub fn inverse(self) -> Self {
		match self {
			ConnectionOrientation::InnerRight => ConnectionOrientation::OuterLeft,
			ConnectionOrientation::InnerLeft => ConnectionOrientation::OuterRight,
			ConnectionOrientation::OuterRight => ConnectionOrientation::InnerLeft,
			ConnectionOrientation::OuterLeft => ConnectionOrientation::InnerRight,
		}
	}
	pub fn flipped(self) -> Self {
		match self {
			ConnectionOrientation::InnerRight => ConnectionOrientation::InnerLeft,
			ConnectionOrientation::InnerLeft => ConnectionOrientation::InnerRight,
			ConnectionOrientation::OuterRight => ConnectionOrientation::OuterLeft,
			ConnectionOrientation::OuterLeft => ConnectionOrientation::OuterRight,
		}
	}
	pub fn everted(self) -> Self {
		match self {
			ConnectionOrientation::InnerRight => ConnectionOrientation::OuterRight,
			ConnectionOrientation::InnerLeft => ConnectionOrientation::OuterLeft,
			ConnectionOrientation::OuterRight => ConnectionOrientation::InnerRight,
			ConnectionOrientation::OuterLeft => ConnectionOrientation::InnerLeft,
		}
	}

	pub fn is_left(&self) -> bool {
		match self {
			ConnectionOrientation::InnerLeft | ConnectionOrientation::OuterLeft => true,
			ConnectionOrientation::InnerRight | ConnectionOrientation::OuterRight => false,
		}
	}
	pub fn is_inner(&self) -> bool {
		match self {
			ConnectionOrientation::InnerRight | ConnectionOrientation::InnerLeft => true,
			ConnectionOrientation::OuterRight | ConnectionOrientation::OuterLeft => false,
		}
	}
	/// Whether the center of the arc is to the left of the straight line connecting the points.
	pub fn center_is_left(&self) -> bool {
		match self {
			ConnectionOrientation::InnerLeft | ConnectionOrientation::OuterRight => true,
			ConnectionOrientation::InnerRight | ConnectionOrientation::OuterLeft => false,
		}
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Connection {
	pub orientation: ConnectionOrientation,
	// pub radius: Number, // TODO: make this a reference to a ConnectionSize and store that as Resources in the Stage
	//                     // ISSUE: actually don't store it in the stage cause radius is an important part of our data -> store in Data
	//                     // DESIGN: do we even need the indirection
	pub size: SizeId,
}

impl Connection {
	pub fn new(orientation: ConnectionOrientation, size: SizeId) -> Self {
		// pub fn new(orientation: ConnectionOrientation, radius: Number) -> Self {
		// Self { orientation, radius }
		Self { orientation, size }
	}
	pub fn invert(&mut self) {
		self.orientation = self.orientation.inverse();
	}
	pub fn flip(&mut self) {
		self.orientation = self.orientation.flipped();
	}
	pub fn evert(&mut self) {
		self.orientation = self.orientation.everted();
	}
}
