use std::{iter::once, str::FromStr};

use result_or_err::ResultOrErr;
use serde::{Deserialize, Serialize};

use crate::common::Number;

pub type SizeId = usize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Class {
	size: Number,
	#[serde(default = "generate_color")]
	color: String,
}
impl Class {
	pub fn new(size: Number, color: String) -> Self {
		Self { size, color }
	}
}
impl ToString for Class {
	fn to_string(&self) -> String {
		format!("{} {}", self.size, self.color)
	}
}
impl FromStr for Class {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (size, color) = match s.split_once(" ") {
			Some((size, color)) => (size.parse().or_err(())?, color.to_string()),
			None => (s.parse().or_err(())?, generate_color()),
			_ => return Err(()),
		};
		Ok(Self::new(size, color))
	}
}
pub fn generate_color() -> String {
	"white".to_string()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Classes {
	items: Vec<Class>,
	default: Class,
}
impl Default for Classes {
	fn default() -> Self {
		Self {
			items: vec![
				Class::new(100.0, "red".to_string()),
				Class::new(200.0, "blue".to_string()),
				Class::new(300.0, "purple".to_string()),
				Class::new(400.0, "yellow".to_string()),
			],
			default: Class::new(300.0, "white".to_string()),
		}
	}
}
impl Classes {
	pub fn get_size(&self, id: SizeId) -> Number {
		self.items.get(id).unwrap_or(&self.default).size.clone()
	}
	pub fn get_color(&self, id: SizeId) -> &str {
		&self.items.get(id).unwrap_or(&self.default).color
	}
	pub fn previous(&self, id: SizeId) -> SizeId {
		if id <= 0 {
			return id;
		}
		return id - 1;
	}
	pub fn next(&self, id: SizeId) -> SizeId {
		if id >= self.items.len() - 1 {
			return id;
		}
		return id + 1;
	}
}
impl ToString for Classes {
	fn to_string(&self) -> String {
		self.items.iter().chain(once(&self.default)).map(ToString::to_string).collect::<Vec<_>>().join("\n")
	}
}
impl FromStr for Classes {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut sizes = s.split("\n").map(|p| p.parse::<Class>()).collect::<Result<Vec<_>, _>>().or_err(())?;
		let default = sizes.pop().ok_or(())?;
		Ok(Self { items: sizes, default })
	}
}
