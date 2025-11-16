use ncollide2d::na::Affine2;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
	adapters::ncollide::ToCss,
	common::{Number, Vector},
	model::Settings,
	render::Stage,
	ui::CanvasTarget,
};

pub struct CanvasStage {
	background: HtmlCanvasElement,
	canvas: HtmlCanvasElement,
	overlay: HtmlCanvasElement,
	hover: HtmlCanvasElement,

	background_context: CanvasRenderingContext2d,
	canvas_context: CanvasRenderingContext2d,
	overlay_context: CanvasRenderingContext2d,
	hover_context: CanvasRenderingContext2d,

	pose: Affine2<Number>,
	identity: Affine2<Number>,
	transformation: Affine2<Number>,
}

impl CanvasStage {
	pub fn new(
		background: HtmlCanvasElement,
		canvas: HtmlCanvasElement,
		overlay: HtmlCanvasElement,
		hover: HtmlCanvasElement,
	) -> Self {
		let background_context =
			background.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
		let canvas_context =
			canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
		let overlay_context =
			overlay.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
		let hover_context =
			hover.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();

		Self {
			background,
			canvas,
			overlay,
			hover,

			background_context,
			canvas_context,
			overlay_context,
			hover_context,

			pose: Affine2::identity(),
			identity: Affine2::identity(),
			transformation: Affine2::identity(),
		}
	}

	pub fn base_pose(&self) -> Affine2<Number> {
		self.pose
	}

	pub fn transform(&mut self, transformation: Affine2<Number>) {
		self.transformation *= transformation;

		self.reset_element(&self.background);
		self.reset_element(&self.canvas);
		self.reset_element(&self.overlay);
	}

	pub fn integrate_transformation(&mut self) {
		self.pose *= self.transformation;
		self.transformation = Affine2::identity();

		self.reset_element(&self.background);
		self.reset_element(&self.canvas);
		self.reset_element(&self.overlay);
	}

	fn reset_element(&self, element: &HtmlCanvasElement) {
		element.style().set_property("transform", self.transformation.inverse().to_css().as_str());
	}
}

impl Stage<Settings> for CanvasStage {
	type Target<'a> = CanvasTarget<'a>;

	fn pose(&self) -> Affine2<Number> {
		self.pose * self.transformation
	}
	fn base_pose_mut(&mut self) -> &mut Affine2<Number> {
		&mut self.pose
	}

	fn size(&self) -> Vector {
		Vector::new(self.canvas.client_width() as Number, self.canvas.client_height() as Number)
	}

	fn target_background<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Settings,
	) -> Self::Target<'borrow_time> {
		self.integrate_transformation();
		let size = self.size();
		CanvasTarget::new(&mut self.background_context, resources, &mut self.pose, size)
	}
	fn target_canvas<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Settings,
	) -> Self::Target<'borrow_time> {
		self.integrate_transformation();
		let size = self.size();
		CanvasTarget::new(&mut self.canvas_context, resources, &mut self.pose, size)
	}
	fn target_overlay<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Settings,
	) -> Self::Target<'borrow_time> {
		self.integrate_transformation();
		let size = self.size();
		CanvasTarget::new(&mut self.canvas_context, resources, &mut self.pose, size)
	}
	fn target_hover<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Settings,
	) -> Self::Target<'borrow_time> {
		self.integrate_transformation();
		let size = self.size();
		CanvasTarget::new(&mut self.hover_context, resources, &mut self.identity, size)
	}
}
