use std::{marker::PhantomData, rc::Rc};

use webbit::{
	components::{Button, GroupContainer, GroupContainerElementFactory},
	events::Group,
	Component, ComponentContent,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::ui::Workspace;

pub trait Picker {
	type Choice: Copy;
	fn pick(choice: Self::Choice, workspace: Rc<Workspace>);
}
pub struct BarChoiceFactory<C: Picker> {
	workspace: Rc<Workspace>,
	button_css: &'static str,
	chooser: PhantomData<C>,
}
impl<C: Picker> BarChoiceFactory<C> {
	pub fn new(button_css: &'static str, workspace: Rc<Workspace>) -> Self {
		Self { workspace, button_css, chooser: PhantomData::default() }
	}
}
impl<C: Picker + 'static> GroupContainerElementFactory for BarChoiceFactory<C> {
	type ConstructionArgs = (String, C::Choice, bool);
	type Element = Button;
	fn make_new(&mut self, group: &mut Group<()>, mut args: Self::ConstructionArgs) -> Component<Self::Element> {
		let button = Button::new(Some(args.0.as_str()), self.button_css);
		let token = group.register({
			let button = button.element().clone();
			move |()| {
				let Ok(element) = button.clone().dyn_into::<HtmlElement>() else { return };
				element.style().remove_property("border-color");
			}
		});
		if let Ok(token) = token {
			if args.2 {
				if let Ok(element) = button.element().clone().dyn_into::<HtmlElement>() {
					C::pick(args.1, self.workspace.clone());
					token.notify(());
					element.style().set_property("border-color", "#10578c");
				};
			}
			button.on_click.set_handler({
				let workspace = self.workspace.clone();
				let button = button.element().clone();
				let chooser = self.chooser;
				move |_| {
					C::pick(args.1, workspace.clone());
					token.notify(());
					let Ok(element) = button.clone().dyn_into::<HtmlElement>() else { return };
					element.style().set_property("border-color", "#10578c");
				}
			});
		}
		Component::make(button)
	}
}

pub type BarChoice<T> = GroupContainer<BarChoiceFactory<T>>;
