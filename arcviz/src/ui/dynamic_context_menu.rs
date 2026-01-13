use anyhow::Result;
use result_or_err::ResultOrErr;
use web_sys::HtmlDivElement;

use webbit::{ComponentContent, DynamicComponent, elements::*, events::BubbleStopper};

pub struct MenuAction {
	element: HtmlDivElement,
	content: Box<dyn DynamicComponent>,
}
impl MenuAction {
	pub fn new(content: impl DynamicComponent + 'static) -> Self {
		let element = div();
		content.mount_in(&element);
		Self { element, content: Box::new(content) }
	}
}
impl ComponentContent for MenuAction {
	fn element(&self) -> &web_sys::Element {
		&self.element
	}
}

pub struct DynamicContextMenu {
	element: HtmlDivElement,
	quick_action_div: HtmlDivElement,
	action_div: HtmlDivElement,

	quick_actions: Vec<MenuAction>,
	actions: Vec<MenuAction>,
}
impl DynamicContextMenu {
	pub fn new(css: &'static str) -> Self {
		let element = styled(div(), ["context-menu", css].join(" ").as_str());
		BubbleStopper::new(element.clone().into(), "click");
		BubbleStopper::new(element.clone().into(), "pointerdown");
		BubbleStopper::new(element.clone().into(), "pointermove");
		BubbleStopper::new(element.clone().into(), "pointerup");
		BubbleStopper::new(element.clone().into(), "contextmenu");

		let quick_action_div = on(&element, styled(div(), "context-menu-quick-section"));
		let action_div = on(&element, styled(div(), "context-menu-section"));

		Self {
			element,
			quick_actions: Default::default(),
			actions: Default::default(),
			quick_action_div,
			action_div,
		}
	}
	pub fn add_quick_action(&mut self, action: impl DynamicComponent + 'static) -> Result<(), ()> {
		let action = MenuAction::new(action);
		action.mount_in(&self.quick_action_div).or_err(())?;
		self.quick_actions.push(action);
		Ok(())
	}
	pub fn add_action(&mut self, action: impl DynamicComponent + 'static) -> Result<(), ()> {
		let action = MenuAction::new(action);
		action.mount_in(&self.action_div).or_err(())?;
		self.actions.push(action);
		Ok(())
	}
}
impl ComponentContent for DynamicContextMenu {
	fn element(&self) -> &web_sys::Element {
		&self.element
	}
}
