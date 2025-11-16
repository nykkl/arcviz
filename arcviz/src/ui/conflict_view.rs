use webbit::{
	elements::{div, label, on, styled},
	ComponentContent,
};
use web_sys::HtmlDivElement;

use crate::ui::AppContext;

pub struct ConflictView {
	root: HtmlDivElement,
	context: AppContext,
}
impl ConflictView {
	pub fn new(context: AppContext) -> Self {
		let root = styled(div(), "conflict-view");

		let this = Self { root, context };

		this.refresh();

		this
	}
	pub fn clear(&self) {
		while let Some(child) = self.root.last_child() {
			self.root.remove_child(&child);
		}
	}
	pub fn refresh(&self) {
		self.clear();

		let Some(context) = self.context.access() else { return };

		for conflict in context.conflicts_representation() {
			on(&self.root, styled(label(conflict.0.as_str()), "conflict-element"));
		}
	}
}
impl ComponentContent for ConflictView {
	fn element(&self) -> &web_sys::Element {
		&self.root
	}
	fn update(&self) -> anyhow::Result<()> {
		self.refresh();
		Ok(())
	}
}
