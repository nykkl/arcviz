use std::rc::Rc;

use web_sys::HtmlDivElement;
use webbit::{
	Component, ComponentContent,
	components::Button,
	elements::{div, styled},
	events::CustomRefEventListener,
};

pub struct Form<T: ComponentContent + 'static> {
	element: HtmlDivElement,
	content: Component<T>,
	submit: Component<Button>,
	pub on_submit: CustomRefEventListener<T>,
}
impl<T: ComponentContent> Form<T> {
	pub fn new(content: T, submit_text: &str, css: &str) -> Rc<Self> {
		let element = styled(div(), css);

		let submit = Component::make(Button::new(Some(submit_text), ""));
		content.mount_in(&element);
		let content = Component::make(content);
		submit.mount_in(&element);
		let on_submit = CustomRefEventListener::new();

		let this = Rc::new(Self { element, content, submit, on_submit });

		this.submit.on_click.set_handler({
			let this = this.clone();
			move |_| {
				this.on_submit.fire(&this.content);
			}
		});

		this
	}
}
impl<T: ComponentContent> ComponentContent for Form<T> {
	fn element(&self) -> &web_sys::Element {
		&self.element
	}
}
