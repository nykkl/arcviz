use std::rc::Rc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_derive_state::{DeserializeState, SerializeState};

use crate::io::state::IOState;

use super::{Resource, ResourceInstance, ResourceProvider};


#[derive(Clone)]
#[derive(SerializeState, DeserializeState)]
#[serde(serialize_state = "IOState", deserialize_state = "IOState")]
pub struct ResourceHandle {
	#[serde(serialize_state_with = "serialize_inner")]
	#[serde(deserialize_state_with = "deserialize_inner")]
	reference: Rc<ResourceInstance>,
}
impl ResourceHandle {
	pub fn new(resource: Rc<ResourceInstance>) -> Self {
		Self {
			reference: resource,
		}
	}
	pub fn id(&self) -> &str {
		self.reference.id()
	}
	pub fn raw(&self) -> Result<&Vec<u8>,()> {
		self.reference.data()
	}
	// pub fn set_to(&mut self, other: &Self) {
	// 	self.reference = Rc::clone(&other.reference);
	// }
}
impl From<Rc<ResourceInstance>> for ResourceHandle {
    fn from(value: Rc<ResourceInstance>) -> Self {
        Self::new(value)
    }
}
impl From<ResourceHandle> for Rc<ResourceInstance> {
    fn from(value: ResourceHandle) -> Self {
        value.reference
    }
}

fn serialize_inner<S: Serializer>(self_: &Rc<ResourceInstance>, serializer: S, seed: &IOState) -> Result<S::Ok, S::Error> {
	self_.id().serialize(serializer)
}
fn deserialize_inner<'de, D: Deserializer<'de>>(seed: &mut IOState, deserializer: D) -> Result<Rc<ResourceInstance>, D::Error> {
	String::deserialize(deserializer).map(|id| seed.resources.get_or_insert_reference(id.as_str()).into())
}
