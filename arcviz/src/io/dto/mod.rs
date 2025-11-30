//! 'DataTransferObjects'
//! 'Plain old data' representations of the [crate::model] data structures.
//! These are meant as an intermediary between the runtime data structures and the different file formats.
//! As such they are meant to be plain data structures with public members and without much behavior that directly translate into what is stored on disk.

use serde::{Deserialize, Serialize};

use crate::{
	common::{Number, Vector},
	model::{
		Class, Classes, Connection, ConnectionOrientation, Connections, Data, SizeId, Vertex, VertexId, Vertices,
	},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct DataDto {
	pub classes: Vec<ClassDto>,
	pub vertices: Vec<VertexDto>,
	pub connections: Vec<ConnectionDto>,
}
impl From<&Data> for DataDto {
	fn from(value: &Data) -> Self {
		DataDto {
			classes: (&value.classes).into(),
			vertices: (&value.vertices).into(),
			connections: value.get_connections().into(),
		}
	}
}
impl From<&DataDto> for Data {
	fn from(value: &DataDto) -> Self {
		let mut data = Data::new((&value.classes).into(), (&value.vertices).into());
		for c in &value.connections {
			data.add_connection(c.start, c.end, (&c.orientation).into(), c.size);
		}
		data
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClassDto {
	pub size: Number,
	#[serde(default = "ClassDto::generate_color")]
	pub color: String,
}
impl ClassDto {
	pub fn generate_color() -> String {
		"white".to_string()
	}
}
impl From<&Class> for ClassDto {
	fn from(value: &Class) -> Self {
		Self { size: value.size().clone(), color: value.color().to_owned() }
	}
}
impl From<&Classes> for Vec<ClassDto> {
	fn from(value: &Classes) -> Self {
		value.items().iter().map(ClassDto::from).collect()
	}
}
impl From<&ClassDto> for Class {
	fn from(value: &ClassDto) -> Self {
		Self::new(value.size, value.color.clone())
	}
}
impl From<&Vec<ClassDto>> for Classes {
	fn from(value: &Vec<ClassDto>) -> Self {
		Self::new(value.iter().map(Class::from).collect(), Class::new(300.0, "white".to_string()))
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VertexDto {
	pub position: Vector,
	pub label: Option<String>,
}
impl From<&Vertex> for VertexDto {
	fn from(value: &Vertex) -> Self {
		Self { position: value.position, label: value.label.clone() }
	}
}
impl From<&VertexDto> for Vertex {
	fn from(value: &VertexDto) -> Self {
		Self { position: value.position, label: value.label.clone() }
	}
}
impl From<&Vertices> for Vec<VertexDto> {
	fn from(value: &Vertices) -> Self {
		value.items.iter().map(VertexDto::from).collect()
	}
}
impl From<&Vec<VertexDto>> for Vertices {
	fn from(value: &Vec<VertexDto>) -> Self {
		Self { items: value.iter().map(Vertex::from).collect() }
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionDto {
	pub start: VertexId,
	pub end: VertexId,
	pub orientation: ConnectionOrientationDto,
	pub size: SizeId,
}
impl From<&ConnectionDto> for Connection {
	fn from(value: &ConnectionDto) -> Self {
		Self { orientation: (&value.orientation).into(), size: value.size.into() }
	}
}
impl From<&Connections> for Vec<ConnectionDto> {
	fn from(value: &Connections) -> Self {
		let mut connections = Vec::new();
		value.foreach(|start, end, connection| {
			connections.push(ConnectionDto {
				start,
				end,
				orientation: (&connection.orientation).into(),
				size: connection.size,
			});
		});
		connections
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ConnectionOrientationDto {
	InnerRight,
	InnerLeft,
	OuterRight,
	OuterLeft,
}
impl From<&ConnectionOrientation> for ConnectionOrientationDto {
	fn from(value: &ConnectionOrientation) -> Self {
		match value {
			ConnectionOrientation::InnerRight => ConnectionOrientationDto::InnerRight,
			ConnectionOrientation::InnerLeft => ConnectionOrientationDto::InnerLeft,
			ConnectionOrientation::OuterRight => ConnectionOrientationDto::OuterRight,
			ConnectionOrientation::OuterLeft => ConnectionOrientationDto::OuterLeft,
		}
	}
}
impl From<&ConnectionOrientationDto> for ConnectionOrientation {
	fn from(value: &ConnectionOrientationDto) -> Self {
		match value {
			ConnectionOrientationDto::InnerRight => ConnectionOrientation::InnerRight,
			ConnectionOrientationDto::InnerLeft => ConnectionOrientation::InnerLeft,
			ConnectionOrientationDto::OuterRight => ConnectionOrientation::OuterRight,
			ConnectionOrientationDto::OuterLeft => ConnectionOrientation::OuterLeft,
		}
	}
}
