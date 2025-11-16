use std::{
	collections::HashMap,
	rc::{Rc, Weak},
};

use crate::{
	// adapters::storage::ResourceWriter,
	render::resources::{MissingResource, Resource, ResourceInstance, ResourceProvider}
};

#[derive(Default)]
pub struct ResourceRegistry {
	resources: HashMap<String, ResourceEntry>,
}

impl ResourceRegistry {
	pub fn add(&mut self, resource: ResourceInstance) -> Result<(), ()> {
		if self.resources.contains_key(resource.id()) { return Err(()) };

		self.resources.insert(resource.id().to_owned(), ResourceEntry::new_pinned(resource));
		Ok(())
	}

	pub fn write(&self, writer: &mut ResourceWriter) -> Result<(),()> {
		let mut result = Ok(());
		for (id, entry) in self.resources.iter() {
			let Some(instance) = entry.get() else { continue };

			let kind = &id[..].chars().next();
			result = result.and(match *kind {
				Some('e') => writer.write_embedded(id.to_owned(), &*instance),
				Some('l') => writer.write_linked(id.to_owned(), &*instance),
				_ => Err(()),
			});
		}
		return result
	}

	pub fn list(&self) -> Vec<ResourceInfo> {
		self.resources.iter()
			.map(|(id, entry)| ResourceInfo::new(id.to_owned(), entry.reference_count()))
		.collect()
	}
}
impl ResourceProvider for ResourceRegistry {
	fn get_reference(&self, id: &str) -> Option<ResourceHandle> { // ISSUE: maybe call this aquire handle, to make clear that 1. returns handle 2. this is a serious aquisition that shouldn't be made unneccessarily
		let entry = self.resources.get(id)?;
		let reference = ResourceHandle::new(entry.get()?);
		Some(reference)
	}

	fn get_or_insert_reference(&mut self, id: &str) -> ResourceHandle {
		if let Some(entry) = self.resources.get(id) {
			if let Some(instance) = entry.get() {
				return ResourceHandle::new(instance);
			}
		}

		let instance = Rc::new(MissingResource::new(id.to_owned()).into()); // TODO: resource is now missing. is that ok?
		self.resources.insert(id.to_owned(), ResourceEntry::Pinned(Rc::clone(&instance)));
		ResourceHandle::new(instance)
	}

	/// Exports a subset of the entryes of this instance as a new instance.
	///
	/// All the entries in the new instance will be pinned.
	///
	/// # Parameters
	/// - ids: The ids of the entries to export
    fn export_subset(&self, ids: &[String]) -> Self { // ISSUE: maybe copy the actual resources instead of just the Rc's. (would require ResourceInstance to implment Clone, but that probably ok)
        let entries = ids.into_iter()
        	.flat_map(|id| Some((id, self.resources.get(id)?)))
        	.map(|(id, entry)| (id.clone(), entry.clone()))
        	.flat_map(|(id, entry)| Some((id, entry.into_pinned()?)))
        .collect::<HashMap<_,_>>();
		Self { resources: entries }
    }
}

#[derive(Clone)]
pub enum ResourceEntry {
	Pinned(Rc<ResourceInstance>),
	Unpinned(Weak<ResourceInstance>),
}
impl ResourceEntry {
	pub fn new_pinned(resource: ResourceInstance) -> Self {
		Self::Pinned(Rc::new(resource))
	}

	pub fn new_unpinned(resource: ResourceInstance) -> Self {
		Self::Unpinned(Rc::downgrade(&Rc::new(resource)))
	}

	pub fn get(&self) -> Option<Rc<ResourceInstance>> {
		match self {
			Self::Pinned(r) => Some(Rc::clone(r)),
			Self::Unpinned(r) => r.upgrade(),
		}
	}

	pub fn reference_count(&self) -> usize {
		match self {
			Self::Pinned(r) => Rc::strong_count(r),
			Self::Unpinned(r) => Weak::strong_count(r),
		}
	}

	pub fn into_pinned(self) -> Option<Self> {
		match self {
			Self::Pinned(_) => Some(self),
			Self::Unpinned(r) => r.upgrade().map(|r| Self::Pinned(Rc::clone(&r))),
		}
	}
	pub fn into_unpinned(self) -> Option<Self> {
		let weak = match self {
			Self::Unpinned(r) => r,
			Self::Pinned(r) => Rc::downgrade(&r),
		};

		if Weak::strong_count(&weak) < 1 { return None; }

		Some(Self::Unpinned(weak))
	}
}

#[derive(Clone, Debug)]
pub struct ResourceInfo {
	pub id: String,
	pub reference_count: usize,
}
impl ResourceInfo {
	pub fn new(id: String, reference_count: usize) -> Self {
		Self {
			id,
			reference_count
		}
	}
}
