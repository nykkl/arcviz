use std::rc::Rc;

use webbit::{
	components::Checkbox,
	elements::{div, styled},
	errors::TracksEnvironment,
	Component, ComponentContent,
};
use web_sys::HtmlDivElement;

use crate::{model::Settings, ui::Workspace};

pub struct SettingsView {
	root: HtmlDivElement,

	pub show_grid: Component<Checkbox>,
	pub show_conflicts: Component<Checkbox>,
	pub show_labels: Component<Checkbox>,
	pub snap_to_grid: Component<Checkbox>,
	pub integrate_on_move: Component<Checkbox>,
}
impl SettingsView {
	pub fn new(workspace: Rc<Workspace>) -> Self {
		let root = styled(div(), "settings-view");

		// let context = workspace.context.clone();

		let settings =
			workspace.context.access().map(|c| c.resources.clone()).unwrap_or_else(|| Settings::default());

		let show_grid = Checkbox::new(Some("show grid lines"), settings.show_grid, "settings-checkbox");
		show_grid.on_change.set_handler({
			let context = workspace.context.clone_for("show_grid./on_change");
			move |v| {
				let Some(mut context) = context.access_mut() else { return };
				context.resources.show_grid = v;
				context.rerender();
			}
		});
		show_grid.mount_in(&root);
		let show_conflicts = Checkbox::new(Some("show conflicts"), settings.show_conflicts, "settings-checkbox");
		show_conflicts.on_change.set_handler({
			let context = workspace.context.clone_for("show_conflicts./on_change");
			move |v| {
				let Some(mut context) = context.access_mut() else { return };
				context.resources.show_conflicts = v;
				context.rerender();
			}
		});
		show_conflicts.mount_in(&root);
		let show_labels = Checkbox::new(Some("show labels"), settings.show_labels, "settings-checkbox");
		show_labels.on_change.set_handler({
			let context = workspace.context.clone_for("show_labels./on_change");
			move |v| {
				let Some(mut context) = context.access_mut() else { return };
				context.resources.show_labels = v;
				context.rerender();
			}
		});
		show_labels.mount_in(&root);
		let snap_to_grid = Checkbox::new(Some("snap to grid"), settings.snap_to_grid, "settings-checkbox");
		snap_to_grid.on_change.set_handler({
			let context = workspace.context.clone_for("snap_to_grid./on_change");
			move |v| {
				let Some(mut context) = context.access_mut() else { return };
				context.resources.snap_to_grid = v;
				context.rerender();
			}
		});
		snap_to_grid.mount_in(&root);
		let integrate_on_move = Checkbox::new(
			Some("continuously render multiselection drag (performance intensive!)"),
			settings.integrate_on_move,
			"settings-checkbox",
		);
		integrate_on_move.on_change.set_handler({
			let context = workspace.context.clone_for("integrate_on_move./on_change");
			let workspace = workspace.clone();
			move |v| {
				let Some(mut context) = context.access_mut() else { return };
				context.resources.integrate_on_move = v;
				workspace.selection_frame.set_integrate_on_move(v);
			}
		});
		integrate_on_move.mount_in(&root);
		let auto_open_context_menu =
			Checkbox::new(Some("auto open context menu"), settings.auto_open_context_menu, "settings-checkbox");
		auto_open_context_menu.on_change.set_handler({
			let context = workspace.context.clone_for("auto_open_context_menu./on_change");
			move |v| {
				let Some(mut context) = context.access_mut() else { return };
				context.resources.auto_open_context_menu = v;
			}
		});
		auto_open_context_menu.mount_in(&root);

		Self {
			root,
			show_grid: Component::make_sharable(show_grid),
			show_conflicts: Component::make_sharable(show_conflicts),
			show_labels: Component::make_sharable(show_labels),
			snap_to_grid: Component::make_sharable(snap_to_grid),
			integrate_on_move: Component::make_sharable(integrate_on_move),
		}
	}
}
impl ComponentContent for SettingsView {
	fn element(&self) -> &web_sys::Element {
		&self.root
	}
}
