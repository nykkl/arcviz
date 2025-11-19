use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use ncollide2d::na::{convert, Affine2, Scale2, Translation2};
use result_or_err::ResultOrErr;
use wasm_bindgen::JsValue;
use web_sys::{console, HtmlDivElement, PointerEvent, WheelEvent};
use webbit::{
	components::ResizeCanvas,
	elements::{div, styled},
	errors::{IgnoreErr, TracksEnvironment},
	events::{BubbleStopper, SharedEventListener},
	io::FileIOHandler,
	Component, ComponentContent, Context,
};

use crate::{
	common::{Number, Vector},
	model::{ConnectionOrientation, Settings, SizeId},
	render::{resources::ResourceProvider, RenderTarget, Stage},
	ui::Mode,
};

use super::{CanvasStage, HandDispatcher, SelectionFrame, ToolDispatcher, WorkspaceContext};

pub type AppContext = Context<WorkspaceContext<CanvasStage>, IgnoreErr>;

pub struct Workspace {
	element: HtmlDivElement,
	pub context: AppContext,
	canvas1: Component<ResizeCanvas>,
	canvas2: Component<ResizeCanvas>,
	canvas3: Component<ResizeCanvas>,
	canvas4: Component<ResizeCanvas>,
	hand_dispatcher: RefCell<HandDispatcher>,
	pub tool_dispatcher: RefCell<ToolDispatcher>,
	down_listener: SharedEventListener<PointerEvent>,
	move_listener: SharedEventListener<PointerEvent>,
	up_listener: SharedEventListener<PointerEvent>,
	wheel_listener: SharedEventListener<WheelEvent>,
	pub selection_frame: Component<SelectionFrame>,
	io: FileIOHandler,
}

impl Workspace {
	pub fn new(io: FileIOHandler) -> Rc<Self> {
		let element = styled(div(), "workspace");
		BubbleStopper::configure(element.clone().into(), "contextmenu", |b| b.prevent_default());

		let canvas1 = Component::make_sharable(ResizeCanvas::new("canvas1"));
		let canvas2 = Component::make_sharable(ResizeCanvas::new("canvas2"));
		let canvas3 = Component::make_sharable(ResizeCanvas::new("canvas3"));
		let canvas4 = Component::make_sharable(ResizeCanvas::new("canvas4"));
		canvas1.mount_in(&element);
		canvas2.mount_in(&element);
		canvas3.mount_in(&element);
		canvas4.mount_in(&element);
		let down_listener = SharedEventListener::new(element.clone().into(), "pointerdown");
		let move_listener = SharedEventListener::new(element.clone().into(), "pointermove");
		let up_listener = SharedEventListener::new(element.clone().into(), "pointerup");
		let wheel_listener = SharedEventListener::new(element.clone().into(), "wheel");

		let stage = CanvasStage::new(
			canvas1.canvas.clone(),
			canvas2.canvas.clone(),
			canvas3.canvas.clone(),
			canvas4.canvas.clone(),
		);
		let settings = Settings::default();
		let context = Context::make(WorkspaceContext::new(stage, settings.clone()), IgnoreErr::default());
		let frame = SelectionFrame::new(element.clone().into(), context.clone_for("SelectionFrame"));
		frame.set_integrate_on_move(settings.integrate_on_move);
		let frame_component = Component::make_sharable(frame.clone());
		let hand_dispatcher =
			RefCell::new(HandDispatcher::new(context.clone_for("HandDispatcher"), element.clone().into()));
		let tool_dispatcher =
			RefCell::new(ToolDispatcher::new(context.clone_for("ToolDispatcher"), frame, element.clone().into()));

		let this = Rc::new(Self {
			element,
			context,
			canvas1,
			canvas2,
			canvas3,
			canvas4,
			hand_dispatcher,
			tool_dispatcher,
			down_listener,
			move_listener,
			up_listener,
			wheel_listener,
			selection_frame: frame_component,
			io,
		});

		this.update();

		this
			.canvas2
			.on_resize
			.set_handler({
				// ISSUE: this should be done with all canvases to be correct (doing this wrong leads to errors); maybe we should make one resize canvas with multiple members since they are completely congruent anyway
				let this = this.clone();
				move |(w, h)| {
					this.update();
				}
			})
			.or_err(())
			.unwrap();
		this.down_listener.set_handler({
			let this = this.clone();
			move |event: PointerEvent| {
				event.prevent_default();
				event.stop_propagation();

				let Ok(hand_dispatcher) = &mut this.hand_dispatcher.try_borrow_mut() else { return };
				let Ok(tool_dispatcher) = &mut this.tool_dispatcher.try_borrow_mut() else { return };

				// pick exactly ONE dispatch method
				let mut pan = false;
				if let Some(context) = this.context.access() {
					if matches!(context.mode, Mode::Hand) {
						pan = true;
					}
				}
				match (pan, event.pointer_type().as_str(), event.buttons()) {
					(true, _, _) => {
						hand_dispatcher.register_down(event);
					},
					(_, "mouse", 1 | 2) => {
						tool_dispatcher.register_down(event);
					},
					(_, "mouse", 4) => {
						hand_dispatcher.register_down(event);
					},
					(_, "touch", _) => {
						hand_dispatcher.register_down(event);
					},
					(_, "pen", _) => {
						tool_dispatcher.register_down(event);
					},
					_ => (),
				}
			}
		});
		this.move_listener.set_handler({
			let this = this.clone();
			move |event: PointerEvent| {
				event.prevent_default();
				event.stop_propagation();

				if let Ok(hand_dispatcher) = &mut this.hand_dispatcher.try_borrow_mut() {
					hand_dispatcher.offer_move(event.clone());
					this.selection_frame.update();
				}
				if let Ok(tool_dispatcher) = &mut this.tool_dispatcher.try_borrow_mut() {
					tool_dispatcher.offer_move(event.clone());
				}
			}
		});
		this.up_listener.set_handler({
			let this = this.clone();
			move |event: PointerEvent| {
				event.prevent_default();
				event.stop_propagation();

				if let Ok(tool_dispatcher) = &mut this.tool_dispatcher.try_borrow_mut() {
					tool_dispatcher.offer_up(event.clone());
				}
				if let Ok(hand_dispatcher) = &mut this.hand_dispatcher.try_borrow_mut() {
					hand_dispatcher.offer_up(event);
				}

				this.selection_frame.update();
			}
		});
		this.wheel_listener.set_handler({
			let this = this.clone();
			move |event: WheelEvent| {
				event.prevent_default();
				event.stop_propagation();

				let Some(mut context) = this.context.access_mut() else { return };

				let mut zoom = 1.0;
				if event.delta_y() > 0.0 {
					zoom = 1.0 / 0.85;
				} else if event.delta_y() < 0.0 {
					zoom = 0.85;
				}
				let scale: Affine2<Number> = convert(Scale2::new(zoom, zoom));
				let frame_start = this.element.get_bounding_client_rect();
				let offset = Vector::new(event.client_x() as f64, event.client_y() as f64)
					- Vector::new(frame_start.left() as f64, frame_start.top() as f64);
				let t: Affine2<Number> = convert(Translation2::new(offset.x, offset.y));
				if let Some(inv_t) = t.try_inverse() {
					let scale = t * scale * inv_t;
					context.stage.transform(scale);
					let resources = Settings::default();
					let mut target = context.stage.target_canvas(&resources);
					target.reset();
					context.rerender();
				}

				drop(context);
				this.selection_frame.update();
				this.selection_frame.rerender();
			}
		});

		this.io.on_load().set_handler({
			let this = this.clone();
			move |data| {
				let Ok(Some(data)) = data else { return };
				this.selection_frame.close();
				let Some(mut context) = this.context.access_mut() else { return };
				if context.load(&data).is_err() && context.load_text(&data).is_err() {
					console::log_1(&JsValue::from("failed to deserialize"));
				}
				context.rerender();
			}
		});

		this
	}

	pub fn save(&self) {
		if let Some(context) = self.context.access() {
			let Ok(data) = context.serialize() else { return };
			self.io.save(&data);
		}
	}
	pub fn save_text(&self) {
		if let Some(context) = self.context.access() {
			self.io.save(&context.to_text());
		}
	}
	pub fn load(&self) {
		self.io.load();
	}

	pub fn export_ipe(&self) {
		if let Some(context) = self.context.access() {
			self.io.save(&context.export_ipe());
		}
	}

	pub fn set_mode(&self, mode: Mode) {
		if let Some(mut context) = self.context.access_mut() {
			context.mode = mode;
		}
	}
	pub fn set_orientation(&self, orientation: ConnectionOrientation) {
		if let Some(mut context) = self.context.access_mut() {
			context.orientation = orientation;
		}
	}
	pub fn set_size(&self, size: SizeId) {
		if let Some(mut context) = self.context.access_mut() {
			context.size = size;
		}
	}
}

#[derive(Default)]
pub struct DummyResources;
impl ResourceProvider for DummyResources {}

impl ComponentContent for Workspace {
	fn element(&self) -> &web_sys::Element {
		&self.element
	}

	fn update(&self) -> anyhow::Result<()> {
		let mut context = self.context.access_mut_or(anyhow!("can't access data"))?;
		context.rerender();
		Ok(())
	}
}
