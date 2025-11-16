use std::str::FromStr;

use result_or_err::ResultOrErr;
use serde::{Deserialize, Serialize};

use crate::common::Vector;

#[derive(Clone, Serialize, Deserialize)]
pub struct Vertex {
	pub position: Vector,
	pub label: Option<String>,
}

impl Vertex {
	pub fn new(position: Vector) -> Self {
		Self { position, label: None }
	}

	pub fn set_label(&mut self, label: String) {
		self.label = Some(label);
	}
	pub fn remove_label(&mut self) {
		self.label = None;
	}
}

impl ToString for Vertex {
	fn to_string(&self) -> String {
		match &self.label {
			None => format!("{} {}", self.position.x, self.position.y),
			Some(label) => format!("{} {} {}", self.position.x, self.position.y, label),
		}
	}
}
impl FromStr for Vertex {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (x, rest) = s.split_once(" ").ok_or(())?;
		if let Some((y, label)) = rest.split_once(" ") {
			Ok(Self {
				position: Vector::new(x.parse().or_err(())?, y.parse().or_err(())?),
				label: Some(label.to_owned()),
			})
		} else {
			Ok(Self { position: Vector::new(x.parse().or_err(())?, rest.parse().or_err(())?), label: None })
		}
	}
}
