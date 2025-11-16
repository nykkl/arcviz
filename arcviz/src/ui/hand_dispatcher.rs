use std::collections::HashMap;

use ncollide2d::na::{convert, Affine2, Scale2, Translation2};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, PointerEvent};

use crate::common::{Number, Vector};

use super::AppContext;

pub struct HandDispatcher {
	context: AppContext,
	pointers: HashMap<i32, Vector>,
	element: HtmlElement,
}

impl HandDispatcher {
	pub fn new(context: AppContext, element: HtmlElement) -> Self {
		Self { context, pointers: HashMap::new(), element }
	}

	pub fn register_down(&mut self, event: PointerEvent) -> Result<(), ()> {
		event.target().unwrap().dyn_into::<HtmlElement>().unwrap().set_pointer_capture(event.pointer_id());

		let position = self.relative_position(&event);

		self.pointers.insert(event.pointer_id(), position);

		Ok(())
	}

	pub fn offer_move(&mut self, event: PointerEvent) -> Result<bool, ()> {
		let id = event.pointer_id();
		if self.pointers.get(&id).is_none() {
			return Ok(false);
		};

		let position = self.relative_position(&event);
		self.move_to(id, position.clone())?;

		Ok(true)
	}

	pub fn offer_up(&mut self, event: PointerEvent) -> Result<bool, ()> {
		let id = event.pointer_id();
		if self.pointers.get(&id).is_none() {
			return Ok(false);
		};

		let position = self.relative_position(&event);
		self.move_to(id, position.clone())?;
		let mut context = self.context.access_mut_or(())?;
		context.stage.integrate_transformation();
		context.rerender();
		self.pointers.remove(&id);

		Ok(true)
	}

	fn move_to(&mut self, id: i32, position: Vector) -> Result<(), ()> {
		let Some(old_position) = self.pointers.insert(id, position.clone()) else { return Err(()) };

		let other = self.pointers.iter().find(|&(k, _)| k != &id);

		if let Some((_, other)) = other {
			self.zoom_pan((old_position, position), (other.clone(), other.clone()));
		} else {
			self.pan((old_position, position));
		}

		Ok(())
	}

	fn zoom_pan(&mut self, pointer1: (Vector, Vector), pointer2: (Vector, Vector)) -> Result<(), ()> {
		let mid1 = (pointer1.0.clone() + pointer2.0.clone()) / 2.0;
		let mid2 = (pointer1.1.clone() + pointer2.1.clone()) / 2.0;
		let path = mid1 - mid2.clone();
		let translation = Translation2::new(path.x, path.y);

		let dist1 = (pointer2.0 - pointer1.0).length();
		let dist2 = (pointer2.1 - pointer1.1).length();
		let zoom = dist1 / dist2;
		let scale: Affine2<Number> = convert(Scale2::new(zoom, zoom));
		let t: Affine2<Number> = convert(Translation2::new(mid2.x, mid2.y));
		let Some(inv_t) = t.try_inverse() else { return Err(()) };
		let scale = t * scale * inv_t;

		let mut context = self.context.access_mut_or(())?;
		context.stage.transform(translation * scale);

		Ok(())
	}
	fn pan(&mut self, pointer1: (Vector, Vector)) -> Result<(), ()> {
		let path = pointer1.0 - pointer1.1;
		let translation = Translation2::new(path.x, path.y);

		let mut context = self.context.access_mut_or(())?;
		context.stage.transform(convert(translation));

		Ok(())
	}

	pub fn relative_position(&self, event: &PointerEvent) -> Vector {
		let position = Vector::new(event.client_x() as f64, event.client_y() as f64);
		let start = self.element.get_bounding_client_rect();
		position - Vector::new(start.left() as f64, start.top() as f64)
	}
}
