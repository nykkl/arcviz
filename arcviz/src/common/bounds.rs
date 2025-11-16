use serde::{Deserialize, Serialize};

use super::Vector;

#[derive(Clone, Serialize, Deserialize)]
pub struct Bounds {
	start: Vector,
	size: Vector,
}

impl Bounds {
	pub fn new(start: Vector, size: Vector) -> Self {
		Self { start, size }
	}
	pub fn new_with_end(start: Vector, end: Vector) -> Self {
		Self::new(start, end - start)
	}

	pub fn start(&self) -> Vector {
		self.start.clone()
	}
	pub fn size(&self) -> Vector {
		self.size.clone()
	}
	pub fn end(&self) -> Vector {
		self.start.clone() + self.size.clone()
	}
	pub fn min(&self) -> Vector {
		self.end().min(&self.start)
	}
	pub fn max(&self) -> Vector {
		self.end().max(&self.start)
	}

	pub fn contains(&self, point: Vector) -> bool {
		point.x >= self.min().x && point.y >= self.min().y && point.x <= self.max().x && point.y <= self.max().y
	}

	pub fn combined_with(mut self, other: &Self) -> Self {
		// TODO: rename to merged_with
		self.merge(other);
		self
	}

	pub fn merge(&mut self, other: &Self) {
		let new_end = self.end().max(&other.end());
		self.start.set_to_min_with(&other.start());
		self.size = new_end - self.start()
	}
	pub fn try_merge(&mut self, other: &Option<Self>) {
		if let Some(other) = other {
			self.merge(other);
		}
	}

	pub fn expand(&self, by: Vector) -> Self {
		let start = self.start() - by;
		let end = self.end() + by;
		Self::new_with_end(start, end)
	}
	pub fn shrink(&self, by: Vector) -> Self {
		let start = self.start() + by;
		let end = self.end() - by;
		Self::new_with_end(start, end)
	}

	pub fn merged(a: &Option<Self>, b: &Option<Self>) -> Option<Self> {
		let Some(mut a) = a.clone() else { return b.clone() };
		a.try_merge(b);
		Some(a)
	}
}

impl From<Vector> for Bounds {
	fn from(value: Vector) -> Self {
		Self::new(value, Vector::zero())
	}
}
