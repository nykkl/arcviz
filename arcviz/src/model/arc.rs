use std::{f64::consts::PI, ops::Div};

use crate::{
	common::{Bounds, Number, Vector},
	model::Classes,
};

use super::{Connection, ConnectionOrientation, VertexId, Vertices};

pub struct Arc {
	pub center: Vector,
	pub radius: Number,
	pub rotation: Number,
	pub angle: Number,
	start: Vector,
	end: Vector,
}

impl Arc {
	pub fn construct(
		start: VertexId,
		end: VertexId,
		vertices: &Vertices,
		connection: &Connection,
		sizes: &Classes,
	) -> Result<Self, ()> {
		let start = vertices.items.get(start).ok_or(())?.position.clone();
		let end = vertices.items.get(end).ok_or(())?.position.clone();
		let radius = sizes.get_size(connection.size);

		let straight = end.clone() - start.clone();
		let middle = (start.clone() + end.clone()) / 2.0;

		let distance_middle_to_center = (radius.powi(2) - straight.length().div(2.0).powi(2)).sqrt();
		if distance_middle_to_center.is_nan() {
			return Err(());
		}; // arc radius is to small to span from start to end -> (end-start) was bigger than (2*radius) -> sqrt was imaginary (or start or end was nan to begin with)
		let middle_to_center = distance_middle_to_center * straight.normal_unit();

		let center =
			middle.clone() + middle_to_center * if connection.orientation.center_is_left() { 1.0 } else { -1.0 };

		let rotation = (start - center.clone()).angle();
		let alpha = Number::asin(straight.length().div(2.0) / radius);
		let angle = 2.0
			* match connection.orientation {
				ConnectionOrientation::InnerRight => -alpha,
				ConnectionOrientation::OuterRight => -PI + alpha,
				ConnectionOrientation::InnerLeft => alpha,
				ConnectionOrientation::OuterLeft => PI - alpha,
			};

		Ok(Self { center, radius: radius, rotation, angle, start, end })
	}

	fn normalize_angle_to_360(angle: Number) -> Number {
		let angle = angle % (2.0 * PI);
		match angle {
			a if a.is_sign_negative() => a + 2.0 * PI,
			a => a,
		}
	}

	/// Checks if the given angle (in rad) is part of the arc.
	/// I.e. whether the point on the circle the arc lies on that is specified by the angle is part of the arc.
	///
	/// Returns true if it is, false otherwise.
	pub fn contains_angle(&self, angle: Number) -> bool {
		let angle = Self::normalize_angle_to_360(angle);

		let start = Self::normalize_angle_to_360(match self.angle.is_sign_negative() {
			false => self.rotation,
			true => self.rotation + self.angle,
		});
		let end = start + self.angle.abs();

		if angle >= start && angle <= end {
			return true;
		} else if angle + 2.0 * PI >= start && angle + 2.0 * PI <= end {
			return true;
		}
		return false;
	}

	/// Computes the distance between the arc line and the given point.
	pub fn distance_to(&self, point: Vector) -> Number {
		let start = self.center + self.radius * Vector::unit_from_angle(self.rotation);
		let end = self.center + self.radius * Vector::unit_from_angle(self.rotation + self.angle);

		let from_start = point - start;
		let from_end = point - end;
		let from_center = point - self.center;

		let distance = from_start.length().min(from_end.length());
		if self.contains_angle(from_center.angle()) {
			distance.min((from_center.length() - self.radius).abs())
		} else {
			distance
		}
	}

	/// Computes the intersection points with the given Arc regarding only their angles.
	/// I.e. it pretends that the center and radius the Arcs were the same.
	fn angular_intersection(&self, other: &Self) -> (Option<(Number, Number)>, Option<(Number, Number)>) {
		let start1 = self.rotation;
		let end1 = (start1 + self.angle) % (2.0 * PI);
		let start2 = other.rotation;
		let end2 = (start2 + other.angle) % (2.0 * PI);

		let (range1, range2) = match (
			self.contains_angle(start2),
			self.contains_angle(end2),
			other.contains_angle(start1),
			other.contains_angle(end1),
		) {
			(true, true, true, _) => (Some((start1, end2 - start1)), Some((start2, end1 - start2))),
			(true, true, false, _) => (Some((start2, end2 - start2)), None),
			(true, false, _, _) => (Some((start2, end1 - start2)), None),
			(false, true, _, _) => (Some((start1, end2 - start1)), None),
			(false, false, true, _) => (Some((start1, end1 - start1)), None),
			(false, false, false, _) => (None, None),
		};

		if let Some(mut range1) = range1 {
			if range1.1 < 0.0 {
				range1.1 += 2.0 * PI;
			}
		}
		if let Some(mut range2) = range2 {
			if range2.1 < 0.0 {
				range2.1 += 2.0 * PI;
			}
		}

		return (range1, range2);
	}

	/// Computes the intersection points with the given Arc.
	pub fn intersection_with(&self, other: &Self) -> ArcIntersection {
		let self_to_other = other.center.clone() - self.center.clone();
		let distance = self_to_other.length();

		// 0: arcs are concentric
		if !(distance > 0.0) {
			if self.radius != other.radius {
				return ArcIntersection::None;
			}
			let ranges = self.angular_intersection(other);
			let Some(range1) = ranges.0 else { return ArcIntersection::None };
			return ArcIntersection::Concentric(range1, ranges.1);
		}

		// 1: too far apart to intersect
		if distance > self.radius + other.radius {
			return ArcIntersection::None;
		}
		// 2: too close => one fully contains the other
		if self.radius > distance + other.radius {
			return ArcIntersection::None;
		}
		if other.radius > distance + self.radius {
			return ArcIntersection::None;
		}

		// 3: (possibly) just touching
		if distance == self.radius + other.radius {
			// from outside
			if self.contains_angle(self_to_other.angle()) && other.contains_angle((-self_to_other.clone()).angle()) {
				let self_to_intersection = self.radius * self_to_other.unit();
				return ArcIntersection::One(self.center.clone() + self_to_intersection);
			} else {
				return ArcIntersection::None;
			}
		}
		if self.radius == distance + other.radius {
			// other is in self
			if self.contains_angle(self_to_other.angle()) && other.contains_angle(self_to_other.angle()) {
				let self_to_intersection = self.radius * self_to_other.unit();
				return ArcIntersection::One(self.center.clone() + self_to_intersection);
			} else {
				return ArcIntersection::None;
			}
		}
		if other.radius == distance + self.radius {
			// self is in other
			if self.contains_angle((-self_to_other.clone()).angle())
				&& other.contains_angle((-self_to_other.clone()).angle())
			{
				let self_to_intersection = -self.radius * self_to_other.unit();
				return ArcIntersection::One(self.center.clone() + self_to_intersection);
			} else {
				return ArcIntersection::None;
			}
		}

		// 4: (possible) intersecting in 2 places
		// we define the "collision" point to be the point where the straight line through self.center and other.center intersects with the straight line connecting the 2 possible intersection points
		// that is the midpoint of the chord connecting the 2 possible intersection points
		let self_to_collision =
			self_to_other.clone() / 2.0 * (1.0 + ((self.radius.powi(2) - other.radius.powi(2)) / distance.powi(2)));
		let collision_to_intersection_distance = (self.radius.powi(2) - self_to_collision.length().powi(2)).sqrt();
		let collision_to_intersection = collision_to_intersection_distance * self_to_other.normal().unit();
		let left_intersection = self.center.clone() + self_to_collision.clone() + collision_to_intersection.clone();
		let right_intersection =
			self.center.clone() + self_to_collision.clone() - collision_to_intersection.clone();

		let self_to_left = left_intersection.clone() - self.center.clone();
		let self_to_right = right_intersection.clone() - self.center.clone();
		let other_to_left = left_intersection.clone() - other.center.clone();
		let other_to_right = right_intersection.clone() - other.center.clone();

		let mut left = None;
		if self.contains_angle(self_to_left.angle()) && other.contains_angle(other_to_left.angle()) {
			left = Some(left_intersection);
		}
		let mut right = None;
		if self.contains_angle(self_to_right.angle()) && other.contains_angle(other_to_right.angle()) {
			right = Some(right_intersection);
		}

		return ArcIntersection::Two(left, right);
	}

	pub fn bounds(&self) -> Bounds {
		let mut bounds = Bounds::from(self.start);
		bounds.merge(&Bounds::from(self.end));

		let angles = vec![0.0, PI / 2.0, PI, -PI / 2.0];
		let directions = vec![Vector::unit_x(), Vector::unit_y(), -Vector::unit_x(), -Vector::unit_y()];
		let extrema_bounds = angles
			.iter()
			.zip(directions)
			.filter(|(&a, _)| self.contains_angle(a))
			.map(|(_, d)| self.center.clone() + d * self.radius)
			.map(Bounds::from)
			.reduce(|acc, b| acc.combined_with(&b));

		bounds.try_merge(&extrema_bounds);
		bounds
	}
}

/// The intersection of 2 Arcs.
pub enum ArcIntersection {
	/// The Arcs don't intersect.
	None,
	/// The Arcs intersect once in the given point AND could only itersect in one point based on their distance.
	One(Vector),
	/// The Arcs could intersect twice and do intersect in the points given.
	/// The first point is the left, the second point is the right point.
	/// That means that if we trace a directed line from the center of the Arc that intersection was called on to the other,
	/// the first lies on the left on that line.
	Two(Option<Vector>, Option<Vector>),
	Concentric((Number, Number), Option<(Number, Number)>),
}
