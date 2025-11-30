use std::str::FromStr;

use result_or_err::ResultOrErr;

use crate::{
	common::Vector,
	io::dto::{ClassDto, ConnectionDto, ConnectionOrientationDto, DataDto, VertexDto},
	model::{SizeId, VertexId},
};

impl ToString for DataDto {
	fn to_string(&self) -> String {
		let lines = self.classes.iter().map(ToString::to_string);
		let lines = lines.chain([String::new()]);
		let lines = lines.chain(self.vertices.iter().map(ToString::to_string));
		let lines = lines.chain([String::new()]);
		let lines = lines.chain(self.connections.iter().map(ToString::to_string));
		lines.collect::<Vec<_>>().join("\n")
	}
}
impl FromStr for DataDto {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let lines: Vec<&str> = s.split("\n").collect();
		let sections: Vec<&[&str]> = lines.split(|l| l.trim().is_empty()).collect();

		let classes = sections.get(0).ok_or(())?;
		let vertices = sections.get(1).ok_or(())?;
		let connections = sections.get(2).ok_or(())?;

		let classes = classes.into_iter().map(|l| ClassDto::from_str(l)).flat_map(Result::ok).collect();
		let vertices = vertices.into_iter().map(|l| VertexDto::from_str(l)).flat_map(Result::ok).collect();
		let connections =
			connections.into_iter().map(|l| ConnectionDto::from_str(l)).flat_map(Result::ok).collect();

		Ok(Self { classes, vertices, connections })
	}
}

impl ToString for ClassDto {
	fn to_string(&self) -> String {
		format!("{} {}", self.size, self.color)
	}
}
impl FromStr for ClassDto {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (size, color) = match s.split_once(" ") {
			Some((size, color)) => (size.parse().or_err(())?, color.to_string()),
			None => (s.parse().or_err(())?, ClassDto::generate_color()),
			_ => return Err(()),
		};
		Ok(Self { size, color })
	}
}

impl ToString for VertexDto {
	fn to_string(&self) -> String {
		match &self.label {
			None => format!("{} {}", self.position.x, self.position.y),
			Some(label) => format!("{} {} {}", self.position.x, self.position.y, label),
		}
	}
}
impl FromStr for VertexDto {
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

impl ToString for ConnectionDto {
	fn to_string(&self) -> String {
		let orientation = match &self.orientation {
			ConnectionOrientationDto::InnerRight => "right",
			ConnectionOrientationDto::InnerLeft => "left",
			ConnectionOrientationDto::OuterRight => "Right",
			ConnectionOrientationDto::OuterLeft => "Left",
		};
		format!("{} {} {} {}", self.start + 1, self.end + 1, orientation, self.size + 1)
	}
}
impl FromStr for ConnectionDto {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts = s.split_whitespace().collect::<Vec<_>>();
		let (Some(&start), Some(&end), Some(&orientation), Some(&size)) =
			(parts.get(0), parts.get(1), parts.get(2), parts.get(3))
		else {
			return Err(());
		};
		let (Ok(start), Ok(end), orientation, Ok(size)) =
			(start.parse::<VertexId>(), end.parse::<VertexId>(), orientation, size.parse::<SizeId>())
		else {
			return Err(());
		};
		let orientation = match orientation {
			"right" => ConnectionOrientationDto::InnerRight,
			"left" => ConnectionOrientationDto::InnerLeft,
			"Right" => ConnectionOrientationDto::OuterRight,
			"Left" => ConnectionOrientationDto::OuterLeft,
			_ => return Err(()),
		};
		Ok(Self { start: start - 1, end: end - 1, orientation, size: size - 1 })
	}
}
