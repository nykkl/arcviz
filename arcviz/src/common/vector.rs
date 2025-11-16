use std::ops::{self, Neg};

use ncollide2d::na::{Point2, Vector2};
use serde::{Deserialize, Serialize};

use super::Number;

/// A 2D-vector (/point/position/direction/..).
///
/// This is a domain-specific type.
/// Its' implementation is a reflection of the concrete usage of coordinates in this application, not an attempt to reimplement an algbraic primitive.
///
/// That is also why it's not generic.
///
/// Goals:
/// - keep data in our own type, not a library
/// - provide methods to make common/simple calculations easy
/// - integration of vectors accross the application (all coordinates are [Numbers](Number))
/// - common type that library types can be converted into/from
///
/// Non-Goals:
/// - correct mathematical interpretation (diffentiating between vector/point/direction/angle/..)
/// - universal applicability
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 2]", into = "[f64; 2]")]
pub struct Vector {
	pub x: Number,
	pub y: Number,
}

impl Vector {
	pub fn new(x: Number, y: Number) -> Self {
		Self { x, y }
	}

	/// Creates a new Vector where x and y have the same value.
	///
	/// # Parameters
	/// - xy: The value for both x and y.
	pub fn new_square(xy: Number) -> Self {
		Self::new(xy, xy)
	}

	pub fn zero() -> Self {
		Self::new(0.0, 0.0)
	}
	pub fn unit_x() -> Self {
		Self::new(1.0, 0.0)
	}
	pub fn unit_y() -> Self {
		Self::new(0.0, 1.0)
	}
	pub fn unit_from_angle(angle: Number) -> Self {
		Self::new(angle.cos(), angle.sin())
	}

	/// The length of the vector (l2 norm).
	pub fn length(&self) -> Number {
		self.norm_2()
	}
	/// The l1 norm of the vector (Manhattan distance).
	pub fn norm_1(&self) -> Number {
		self.x.abs() + self.y.abs()
	}
	/// The l2 norm of the vector (Euclidean distance).
	pub fn norm_2(&self) -> Number {
		Number::sqrt(self.x.powi(2) + self.y.powi(2))
	}
	/// The l-infinity norm of the vector (maximum of the absolute values of both coordinates).
	pub fn norm_infinity(&self) -> Number {
		Number::max(self.x.abs(), self.y.abs())
	}
	pub fn abs(&self) -> Vector {
		Self { x: self.x.abs(), y: self.y.abs() }
	}

	/// The normal vector to this vector.
	///
	/// I.e. this vector rotated by +90degrees (that is counter-clockwise or "to the left").
	/// The returned vector is not normalized. It has the same length as the original vector.
	pub fn normal(&self) -> Self {
		Self::new(-self.y, self.x)
	}
	/// A unit vector with the same orientation as this one.
	pub fn unit(&self) -> Self {
		self.clone() / self.length()
	}
	/// The normalized normal vector to this vector.
	///
	/// I.e. a vector normal (+90degrees) to this one of length 1.
	pub fn normal_unit(&self) -> Self {
		self.normal().unit()
	}

	/// The angle of this vector (that is between this vector and the x-axis).
	pub fn angle(&self) -> Number {
		self.y.atan2(self.x)
	}

	/// The dot-product with the other vector.
	pub fn dot(&self, other: &Vector) -> Number {
		self.x * other.x + self.y * other.y
	}
	/// The cross-product with the other vector.
	///
	/// That is, if we extend both vectors to the 3rd dimension by setting the 3rd coordinate 0,
	/// this function returns the 3rd coordinate (the only one that is non-0) of the cross-product of those 2 vectors.
	///
	/// - This happens to be the same as the dot product between other and the normal vector of self.
	/// - a cross b = - (b cross a)
	pub fn cross(&self, other: &Vector) -> Number {
		self.normal().dot(other)
	}
	/// The projection of self onto other:
	/// - It's the purely parallel (to other) component of self.
	/// - Equal to the vector self minus its rejection on other.
	pub fn projection_on(&self, other: &Vector) -> Vector {
		self.dot(&other.unit()) * other.unit()
	}
	/// The rejection of self from other:
	/// - It's the purely orthogonal (to other) component of self.
	/// - Equal to the vector self minus its projection on other.
	pub fn rejection_on(&self, other: &Vector) -> Vector {
		*self - self.projection_on(other)
	}

	/// Whether the other Vector is to the left (+0 .. +180)deg of the self Vector.
	pub fn is_left(&self, other: &Self) -> bool {
		self.normal().is_ahead(other)
	}
	/// Whether the other Vector is to the right (-0 .. -180)deg of the self Vector.
	pub fn is_right(&self, other: &Self) -> bool {
		self.normal().is_behind(other)
	}
	/// Whether the other Vector is in front (-90 .. +90)deg of the self Vector.
	pub fn is_ahead(&self, other: &Self) -> bool {
		self.dot(other) > 0.0
	}
	/// Whether the other Vector is behind (+90 .. -90)deg i.e. (+90 .. +270)deg the self Vector.
	pub fn is_behind(&self, other: &Self) -> bool {
		self.dot(other) < 0.0
	}

	pub fn min(mut self, other: &Vector) -> Vector {
		self.set_to_min_with(other);
		self
	}
	pub fn max(mut self, other: &Vector) -> Vector {
		self.set_to_max_with(other);
		self
	}

	/// Sets each coordinate of this vector to the minimum of itself and the corresponding coordinate of the other vector.
	pub fn set_to_min_with(&mut self, other: &Vector) {
		if other.x < self.x {
			self.x = other.x
		};
		if other.y < self.y {
			self.y = other.y
		};
	}
	/// Sets each coordinate of this vector to the maximum of itself and the corresponding coordinate of the other vector.
	pub fn set_to_max_with(&mut self, other: &Vector) {
		if other.x > self.x {
			self.x = other.x
		};
		if other.y > self.y {
			self.y = other.y
		};
	}
}

// TODO: implement all these operations for Borrows instead of owned values -> doesn't matter for performance; better ergonomics (don't have to clone() everywhere)

impl ops::Add for Vector {
	type Output = Self;

	fn add(self, rhs: Vector) -> Self::Output {
		Self { x: self.x + rhs.x, y: self.y + rhs.y }
	}
}

impl Neg for Vector {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self::Output { x: -self.x, y: -self.y }
	}
}

impl ops::Sub for Vector {
	type Output = Self;

	fn sub(self, rhs: Vector) -> Self::Output {
		Self { x: self.x - rhs.x, y: self.y - rhs.y }
	}
}

impl ops::Mul<Vector> for Number {
	type Output = Vector;

	fn mul(self, rhs: Vector) -> Self::Output {
		Vector { x: self * rhs.x, y: self * rhs.y }
	}
}

impl ops::Mul<Number> for Vector {
	type Output = Self;

	fn mul(self, rhs: Number) -> Self::Output {
		Self { x: self.x * rhs, y: self.y * rhs }
	}
}

impl ops::Mul<Self> for Vector {
	type Output = Number;

	fn mul(self, rhs: Self) -> Self::Output {
		self.dot(&rhs)
	}
}

impl ops::Div<Number> for Vector {
	type Output = Self;

	fn div(self, rhs: Number) -> Self::Output {
		Self { x: self.x / rhs, y: self.y / rhs }
	}
}

impl ops::Index<usize> for Vector {
	type Output = Number;

	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => &self.x,
			1 => &self.y,
			_ => panic!(),
		}
	}
}

impl ops::IndexMut<usize> for Vector {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			_ => panic!(),
		}
	}
}

impl From<[f64; 2]> for Vector {
	fn from(value: [f64; 2]) -> Self {
		Self::new(value[0], value[1])
	}
}

impl From<Vector> for [f64; 2] {
	fn from(value: Vector) -> Self {
		[value.x, value.y]
	}
}

impl From<Vector> for Point2<Number> {
	fn from(value: Vector) -> Self {
		Self::new(value.x, value.y)
	}
}

impl From<Point2<Number>> for Vector {
	fn from(value: Point2<Number>) -> Self {
		Self::new(value[0], value[1])
	}
}

impl From<Vector> for Vector2<Number> {
	fn from(value: Vector) -> Self {
		Self::new(value.x, value.y)
	}
}

impl From<Vector2<Number>> for Vector {
	fn from(value: Vector2<Number>) -> Self {
		Self::new(value[0], value[1])
	}
}
