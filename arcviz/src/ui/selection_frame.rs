use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use ncollide2d::na::Translation2;
use web_sys::HtmlElement;
use webbit::{
	Component, ComponentContent,
	common::{Bounds, Vector},
	components::{Button, SelectionFrame as Frame},
	errors::{ErrorHandling, TracksEnvironment},
};

use crate::{
	model::Settings,
	render::{RenderTarget, Stage},
	ui::{AppContext, CanvasStage, DynamicContextMenu},
};

pub struct SelectionFrame {
	context: AppContext,
	frame: Component<Frame>,
	menu: RefCell<Option<Component<DynamicContextMenu>>>,
}

impl SelectionFrame {
	pub fn new(parent: HtmlElement, context: AppContext) -> Rc<Self> {
		let frame = Component::make_sharable(Frame::new(
			parent,
			{
				let context = context.clone_for("()compute_bounds");
				move || {
					let context = context.access()?;
					let bounds = context.get_selection_bounds()?;
					Some(Bounds::new(
						Vector::new(bounds.start().x, bounds.start().y),
						Vector::new(bounds.size().x, bounds.size().y),
					))
				}
			},
			{
				let context = context.clone_for("()integrate_transformation");
				move |transform, frame| {
					{
						let mut context = context.access_mut_or(())?;
						let translation = transform.transform_point(&Vector::zero().into()).into();
						context.move_selection(translation);
					}
					frame.reset();
					frame.rerender(); // Q: probably rerender here, right?
					Ok(())
				}
			},
		));
		let mut stage =
			CanvasStage::new(frame.canvas.clone(), frame.canvas.clone(), frame.canvas.clone(), frame.canvas.clone());

		let this = Rc::new(Self { context: context.clone_for("this"), frame, menu: RefCell::new(None) });

		this.frame.on_context.set_handler({
			let this = this.clone();
			move |_| {
				this.toggle_menu();
			}
		});
		this.frame.on_render.set_handler({
			let context = context.clone_for("/on_render");
			move |()| {
				let Some(context) = context.access() else { return };
				let Some(bounds) = context.get_selection_bounds() else { return };
				let translation = Translation2::new(bounds.min().x, bounds.min().y);
				*stage.base_pose_mut() = context.stage.pose() * translation;
				let r = Settings::default();
				let mut target = stage.target_canvas(&r);
				target.reset();
				context.render_selection_to(&mut target);
			}
		});

		this
	}

	pub fn set_integrate_on_move(&self, value: bool) {
		self.frame.set_integrate_on_move(value);
	}

	pub fn toggle_menu(self: &Rc<Self>) -> Result<(), ()> {
		let Ok(mut menu) = self.menu.try_borrow_mut() else { return Err(()) };
		*menu = match menu.as_ref() {
			Some(_) => None,
			None => {
				let menu = self.make_menu();
				menu.mount_in(&self.frame.control_knob);
				Some(menu)
			},
		};
		Ok(())
	}
	pub fn close_menu(&self) -> Result<(), ()> {
		let Ok(mut menu) = self.menu.try_borrow_mut() else { return Err(()) };
		*menu = None;
		Ok(())
	}
	pub fn open_menu(self: &Rc<Self>) -> Result<(), ()> {
		let Ok(mut menu) = self.menu.try_borrow_mut() else { return Err(()) };
		let component = self.make_menu();
		component.mount_in(&self.frame.control_knob);
		*menu = Some(component);
		Ok(())
	}
	fn make_menu(self: &Rc<Self>) -> Component<DynamicContextMenu> {
		let delete = {
			let selection_frame = self.clone();
			move |context: AppContext| {
				move |_| {
					selection_frame.close();
					let Some(mut context) = context.access_mut() else { return };
					context.delete_selection();
					context.rerender();
				}
			}
		};
		let quick_delete = Button::new_with_handler(
			Some("X"),
			"context-menu-quick-button",
			delete.clone()(self.context.clone_for("menu.quick.()delete")),
		);
		let delete = Button::new_with_handler(
			Some("Delete"),
			"context-menu-button",
			delete(self.context.clone_for("menu.()delete")),
		);
		let duplicate = {
			let selection_frame = self.clone();
			move |context: AppContext| {
				move |_| {
					{
						let Some(mut context) = context.access_mut() else { return };
						context.duplicate_selection();
						context.rerender();
					}
					selection_frame.rerender();
				}
			}
		};
		let quick_duplicate = Button::new_with_handler(
			Some("#"),
			"context-menu-quick-button",
			duplicate.clone()(self.context.clone_for("menu.quick.()duplicate")),
		);
		let duplicate = Button::new_with_handler(
			Some("Duplicate"),
			"context-menu-button",
			duplicate(self.context.clone_for("menu.()duplicate")),
		);
		let invert = Button::new_with_handler(Some("Invert"), "context-menu-button", {
			let context = self.context.clone_for("menu.()invert");
			let selection_frame = self.clone();
			move |_| {
				{
					let Some(mut context) = context.access_mut() else { return };
					context.change_orientation_of_selection(|o| o.inverse());
					context.rerender();
				}
				selection_frame.reset();
				selection_frame.rerender();
			}
		});
		let flip = Button::new_with_handler(Some("Flip"), "context-menu-button", {
			let context = self.context.clone_for("menu.()flip");
			let selection_frame = self.clone();
			move |_| {
				{
					let Some(mut context) = context.access_mut() else { return };
					context.change_orientation_of_selection(|o| o.flipped());
					context.rerender();
				}
				selection_frame.reset();
				selection_frame.rerender();
			}
		});
		let evert = Button::new_with_handler(Some("Evert"), "context-menu-button", {
			let context = self.context.clone_for("menu.()evert");
			let selection_frame = self.clone();
			move |_| {
				{
					let Some(mut context) = context.access_mut() else { return };
					context.change_orientation_of_selection(|o| o.everted());
					context.rerender();
				}
				selection_frame.reset();
				selection_frame.rerender();
			}
		});
		let tag = Button::new_with_handler(Some("Tag"), "context-menu-button", {
			let context = self.context.clone_for("menu.()tag");
			move |_| {
				let Some(mut context) = context.access_mut() else { return };
				context.label_selection();
				context.rerender();
			}
		});
		let increase = Button::new_with_handler(Some("+"), "context-menu-quick-button", {
			let context = self.context.clone_for("menu.()increase");
			let selection_frame = self.clone();
			move |_| {
				let Some(mut context) = context.access_mut() else { return };
				context.change_selection_class(true);
				context.rerender();
				drop(context);
				selection_frame.reset();
				selection_frame.rerender();
			}
		});
		let decrease = Button::new_with_handler(Some("-"), "context-menu-quick-button", {
			let context = self.context.clone_for("menu.()decrease");
			let selection_frame = self.clone();
			move |_| {
				let Some(mut context) = context.access_mut() else { return };
				context.change_selection_class(false);
				context.rerender();
				drop(context);
				selection_frame.reset();
				selection_frame.rerender();
			}
		});
		let mut menu = DynamicContextMenu::new("");
		menu.add_quick_action(Component::make(quick_delete)).handle(&self.context);
		menu.add_quick_action(Component::make(decrease)).handle(&self.context);
		menu.add_quick_action(Component::make(increase)).handle(&self.context);
		menu.add_quick_action(Component::make(quick_duplicate)).handle(&self.context);
		menu.add_action(Component::make(flip)).handle(&self.context);
		menu.add_action(Component::make(evert)).handle(&self.context);
		menu.add_action(Component::make(invert)).handle(&self.context);
		menu.add_action(Component::make(tag)).handle(&self.context);
		menu.add_action(Component::make(duplicate)).handle(&self.context);
		menu.add_action(Component::make(delete)).handle(&self.context);

		Component::make(menu)
	}
	pub fn open(&self) -> Result<(), ()> {
		self.frame.open()
	}
	pub fn close(&self) -> Result<(), ()> {
		self.close_menu()?;
		self.frame.close()
	}
	pub fn reset(&self) -> Result<(), ()> {
		self.frame.reset()
	}
	pub fn rerender(&self) -> Result<(), ()> {
		self.frame.rerender()
	}
}

impl ComponentContent for SelectionFrame {
	fn element(&self) -> &web_sys::Element {
		self.frame.element()
	}

	fn update(&self) -> anyhow::Result<()> {
		self.frame.update()?;
		let Ok(menu) = self.menu.try_borrow() else { return Err(anyhow!("can't access data")) };
		if let Some(menu) = menu.as_ref() {
			menu.update()?;
		}
		Ok(())
	}
}
