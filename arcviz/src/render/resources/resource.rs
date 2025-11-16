use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Resource {
	fn id(&self) -> &str;
	fn data(&self) -> Result<&Vec<u8>, ()>;
}

#[enum_dispatch(Resource)]
// #[derive(Clone)]
pub enum ResourceInstance {
	Missing(MissingResource),
	Embedded(EmbeddedResource),
	// Linked(String),
}

pub struct MissingResource {
	id: String,
}
impl Resource for MissingResource {
	fn id(&self) -> &str {
		&self.id
	}
	fn data(&self) -> Result<&Vec<u8>, ()> {
		Err(())
	}
}
impl MissingResource {
	pub fn new(id: String) -> Self {
		Self { id }
	}
}

pub struct EmbeddedResource {
	id: String,
	data: Vec<u8>,
}
impl Resource for EmbeddedResource {
	fn id(&self) -> &str {
		&self.id
	}
	fn data(&self) -> Result<&Vec<u8>, ()> {
		Ok(&self.data)
	}
}
impl EmbeddedResource {
	pub fn new(id: String, data: Vec<u8>) -> Self {
		Self { id, data }
	}
}
