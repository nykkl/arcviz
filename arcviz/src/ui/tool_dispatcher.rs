use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, PointerEvent};

use crate::{
	common::{Bounds, Vector},
	ui::{Mode, SelectionFrame},
};

use super::AppContext;

#[derive(Clone, Copy, PartialEq)]
enum Button {
	None = 0,
	Left = 1,
	Right = 2,
	Middle = 4,
	Invalid,
}
impl From<Button> for u16 {
	fn from(value: Button) -> Self {
		value as u16
	}
}
impl From<u16> for Button {
	fn from(value: u16) -> Self {
		match value {
			0 => Button::None,
			1 => Button::Left,
			2 => Button::Right,
			4 => Button::Middle,
			_ => Button::Invalid,
		}
	}
}

#[derive(Clone, Copy)]
enum Control {
	Primary,
	Secondary,
	Tertiary,
	Invalid,
}

pub struct ToolDispatcher {
	context: AppContext,
	element: HtmlElement,
	pointer: Option<(i32, Vector, Control)>,
	selection_frame: Rc<SelectionFrame>,
	primary_button: Button,
	secondary_button: Button,
	start_position: Vector,
	show_selection_frame: bool,
}
impl ToolDispatcher {
	pub fn new(context: AppContext, selection_frame: Rc<SelectionFrame>, element: HtmlElement) -> Self {
		Self {
			context: context,
			element,
			pointer: None,
			selection_frame,
			primary_button: Button::Right,
			secondary_button: Button::Left,
			start_position: Vector::zero(),
			show_selection_frame: false,
		}
	}

	pub fn register_down(&mut self, event: PointerEvent) -> Result<bool, ()> {
		if self.pointer.is_some() {
			return Ok(false);
		}; //don't switch pointers
		event.target().unwrap().dyn_into::<HtmlElement>().unwrap().set_pointer_capture(event.pointer_id());
		self.reset();
		let position = self.relative_position(&event);
		self.start_position = position;
		let buttons = self.control(Button::from(event.buttons()));
		self.pointer = Some((event.pointer_id(), position.clone(), buttons));
		let mut context = self.context.access_mut_or(())?;

		self.selection_frame.close();
		context.select_at(position);

		match (buttons, context.selection.is_some(), &context.mode) {
			(Control::Primary, _, Mode::Vertex) => {
				let id = context.add_vertex_at(position);
				context.adjust_vertex(&id);
				context.rerender();
			},
			(Control::Primary, false, Mode::Edit) => {
				let id = context.add_vertex_at(position);
				context.adjust_vertex(&id);
				context.rerender();
			},
			(Control::Primary, _, Mode::Select) => {
				self.show_selection_frame = true;
			},
			_ => (),
		}

		Ok(true)
	}

	pub fn reset(&mut self) {
		self.show_selection_frame = false;
	}

	pub fn offer_move(&mut self, event: PointerEvent) -> Result<bool, ()> {
		let position = self.relative_position(&event);
		let Some(state) = &mut self.pointer else { return Ok(false) };
		if event.pointer_id() != state.0 {
			return Ok(false);
		};
		let movement = position - state.1;
		state.1 = position;
		let buttons = state.2;
		let mut context = self.context.access_mut_or(())?;

		match (buttons, &context.mode) {
			(Control::Secondary, _) => context.move_selection(movement),
			_ => (),
		}

		context.reset_aids();
		match (&context.mode, buttons) {
			(_, Control::Secondary) => {
				context.draw_selection_aid(self.start_position, position);
			},
			(Mode::Edge | Mode::Edit, Control::Primary) => context.draw_connection_aid(position),
			_ => (),
		}

		Ok(true)
	}

	pub fn offer_up(&mut self, event: PointerEvent) -> Result<bool, ()> {
		let Some(state) = &mut self.pointer else { return Ok(false) };
		if event.pointer_id() != state.0 {
			return Ok(false);
		};
		let buttons = state.2;
		let mut context = self.context.access_mut_or(())?;
		let position = self.relative_position(&event);

		match (buttons, context.selection.is_some(), &context.mode) {
			(Control::Primary, true, Mode::Edge | Mode::Edit) => {
				context.connect_at(position);
				self.show_selection_frame = false;
			},
			(Control::Secondary, false, _) => {
				let area = Bounds::new(self.start_position, position - self.start_position);
				context.select_in(area);
				self.show_selection_frame = true;
			},
			(Control::Secondary, true, _) => {
				context.adjust_selection();
				context.rerender();
			},
			_ => (),
		}

		context.reset_aids();

		if context.selection.is_some() && self.show_selection_frame {
			let auto_open_context_menu = context.resources.auto_open_context_menu;
			drop(context);
			self.selection_frame.open();
			if auto_open_context_menu {
				self.selection_frame.open_menu();
			}
		}

		self.pointer = None;
		Ok(true)
	}

	pub fn relative_position(&self, event: &PointerEvent) -> Vector {
		let position = Vector::new(event.client_x() as f64, event.client_y() as f64);
		let start = self.element.get_bounding_client_rect();
		position - Vector::new(start.left() as f64, start.top() as f64)
	}

	fn control(&self, button: Button) -> Control {
		if button == self.primary_button {
			return Control::Primary;
		};
		if button == self.secondary_button {
			return Control::Secondary;
		};
		if button == Button::Middle {
			return Control::Tertiary;
		};
		return Control::Invalid;
	}
}
