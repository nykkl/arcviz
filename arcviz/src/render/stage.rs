use ncollide2d::na::Affine2;

use crate::common::{Number, Vector};

use super::RenderTarget;

pub trait Stage<Resources> {
	type Target<'a>: RenderTarget
	where
		Self: 'a,
		Resources: 'a;

	/// The combined, complete pose of the stage.
	/// Can not me accessed mutably because this may be a composite value.
	/// That is the base pose plus some implementation dependent transformation.
	///
	/// Given in the form of the transformation from this stage's coordinate system to the parent's coordinate system.
	fn pose(&self) -> Affine2<Number>;
	/// The base pose of the stage.
	/// This is not garanteed to be the complete pose (see [pose]). Transformations may have been applied on top of this.
	///
	/// Given in the form of the transformation from this stage's coordinate system to the parent's coordinate system.
	fn base_pose_mut(&mut self) -> &mut Affine2<Number>;
	/// The size of the stage.
	///
	/// Given as a vector in this stage's coordinate system.
	fn size(&self) -> Vector;

	// fn test<'borrow_time>(some: &'borrow_time Settings) -> Self::Target<'borrow_time>;

	fn target_background<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Resources,
	) -> Self::Target<'borrow_time>;
	/// Used to borrow a target from this stage that you can render to.
	///
	/// The idea is that you borrow this for 1 render and then let go of it and borrow it again next time.
	/// This way we keep the Resources seperate from the Stage without having to pass them into every call on the render target separately.
	///
	/// Pseudocode:
	///	`render_to(stage.render_target(resources)); // render_to(RenderTarget) being your render function`
	fn target_canvas<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Resources,
	) -> Self::Target<'borrow_time>;
	fn target_overlay<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Resources,
	) -> Self::Target<'borrow_time>;
	fn target_hover<'borrow_time>(
		&'borrow_time mut self,
		resources: &'borrow_time Resources,
	) -> Self::Target<'borrow_time>;
}
