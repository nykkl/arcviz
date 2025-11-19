use std::rc::Rc;

use ncollide2d::na::Affine2;
use webbit::{
	components::{Button, GroupContainer, SideBar},
	elements::{div, on, styled, text},
	errors::TracksEnvironment,
	events::SharedEventListener,
	Component, ComponentContent,
};
use wasm_bindgen::JsValue;
use web_sys::{console, Element, Event, HtmlDivElement, HtmlInputElement};

use crate::{
	model::ConnectionOrientation,
	render::Stage,
	ui::{BarChoice, BarChoiceFactory, ConflictView, Picker, SettingsView},
};

use super::{Mode, Workspace};

struct ModePicker;
impl Picker for ModePicker {
	type Choice = Mode;
	fn pick(choice: Self::Choice, workspace: Rc<Workspace>) {
		workspace.set_mode(choice);
	}
}
struct OrientationPicker;
impl Picker for OrientationPicker {
	type Choice = ConnectionOrientation;
	fn pick(choice: Self::Choice, workspace: Rc<Workspace>) {
		workspace.set_orientation(choice);
	}
}
struct SizePicker;
impl Picker for SizePicker {
	type Choice = usize;
	fn pick(choice: Self::Choice, workspace: Rc<Workspace>) {
		workspace.set_size(choice);
	}
}

pub struct Bar {
	workspace: Rc<Workspace>,
	dock: Rc<SideBar>,
	element: HtmlDivElement,
	open: Component<Button>,
	save_text: Component<Button>,
	save: Component<Button>,
	export_ipe: Component<Button>,
	mode_selector: Component<BarChoice<ModePicker>>,
	orientation_selector: Component<BarChoice<OrientationPicker>>,
	size_selector: Component<BarChoice<SizePicker>>,
	label: HtmlInputElement,
	label_listener: SharedEventListener<Event>,
	label_button: Component<Button>,
	reset: Component<Button>,
	refresh: Component<Button>,
	settings: Component<Button>,
	conflicts: Component<Button>,
}

impl Bar {
	pub fn new(workspace: Rc<Workspace>, dock: Rc<SideBar>) -> Self {
		let element = styled(div(), "bar");

		let file_group = on(&element, styled(div(), "bar-group"));
		let open = Component::make(Button::new_with_handler(Some("open"), "bar-accent-button", {
			let workspace = workspace.clone();
			move |_| {
				workspace.load();
			}
		}));
		open.mount_in(&file_group);
		let save_text = Component::make(Button::new_with_handler(Some("save"), "bar-accent-button", {
			let workspace = workspace.clone();
			move |_| {
				workspace.save_text();
			}
		}));
		save_text.mount_in(&file_group);
		let save = Component::make(Button::new_with_handler(Some("save json"), "bar-accent-button", {
			let workspace = workspace.clone();
			move |_| {
				workspace.save();
			}
		}));
		save.mount_in(&file_group);
		let export_ipe = Component::make(Button::new_with_handler(Some("export ipe"), "bar-accent-button", {
			let workspace = workspace.clone();
			move |_| {
				workspace.export_ipe();
			}
		}));
		export_ipe.mount_in(&file_group);

		let mut mode_selector =
			GroupContainer::new("bar-group", BarChoiceFactory::new("bar-button", workspace.clone()));
		mode_selector.add(("Pointer".to_owned(), Mode::Select, true));
		mode_selector.add(("Edit".to_owned(), Mode::Edit, false));
		mode_selector.add(("Hand".to_owned(), Mode::Hand, false));
		mode_selector.mount_in(&element);
		let mut orientation_selector =
			GroupContainer::new("bar-group", BarChoiceFactory::new("bar-button", workspace.clone()));
		orientation_selector.add(("C".to_owned(), ConnectionOrientation::OuterLeft, false));
		orientation_selector.add(("(".to_owned(), ConnectionOrientation::InnerLeft, false));
		orientation_selector.add((")".to_owned(), ConnectionOrientation::InnerRight, true));
		orientation_selector.add(("D".to_owned(), ConnectionOrientation::OuterRight, false));
		orientation_selector.mount_in(&element);
		let mut size_selector =
			GroupContainer::new("bar-group", BarChoiceFactory::new("bar-button", workspace.clone()));
		size_selector.add(("1".to_owned(), 0, false));
		size_selector.add(("2".to_owned(), 1, false));
		size_selector.add(("3".to_owned(), 2, true));
		size_selector.add(("4".to_owned(), 3, false));
		size_selector.mount_in(&element);

		let label_group = on(&element, styled(div(), "bar-group"));
		let label = on(&label_group, styled(text(""), "bar-text"));
		let label_listener = SharedEventListener::new(label.clone().into(), "change").with_handler({
			let workspace = workspace.clone();
			let label = label.clone();
			move |_| {
				if let Some(mut context) = workspace.context.access_mut() {
					context.label = label.value();
				}
			}
		});
		let label_button = Component::make(Button::new_with_handler(Some("tag"), "bar-button", {
			let workspace = workspace.clone();
			move |_| {
				if let Some(mut context) = workspace.context.access_mut() {
					context.label_selection();
				}
				workspace.update();
			}
		}));
		label_button.mount_in(&label_group);

		let view_group = on(&element, styled(div(), "bar-group"));
		let reset = Component::make(Button::new_with_handler(Some("[1]"), "bar-accent-button", {
			let workspace = workspace.clone();
			move |_| {
				if let Some(mut context) = workspace.context.access_mut() {
					*context.stage.base_pose_mut() = Affine2::identity();
				}
				workspace.update();
			}
		}));
		reset.mount_in(&view_group);
		let refresh = Component::make(Button::new_with_handler(Some(" ? "), "bar-accent-button", {
			let workspace = workspace.clone();
			move |_| {
				console::log_1(&JsValue::from("refresh"));
				workspace.update();
			}
		}));
		refresh.mount_in(&view_group);
		let settings = Component::make(Button::new_with_handler(Some("settings"), "bar-accent-button", {
			let dock = dock.clone();
			let workspace = workspace.clone();
			move |_| {
				dock.set_title(Some("Settings"));
				dock.open(SettingsView::new(workspace.clone()));
			}
		}));
		settings.mount_in(&view_group);
		let conflicts = Component::make(Button::new_with_handler(Some("conflicts"), "bar-accent-button", {
			let dock = dock.clone();
			let workspace = workspace.clone();
			move |_| {
				dock.set_title(Some("Conflicts"));
				dock.open(ConflictView::new(workspace.context.clone_for("ContextView")));
			}
		}));
		conflicts.mount_in(&view_group);

		Self {
			workspace,
			dock,
			element,
			open,
			save_text,
			save,
			export_ipe,
			mode_selector: Component::make(mode_selector),
			orientation_selector: Component::make(orientation_selector),
			size_selector: Component::make(size_selector),
			label,
			label_listener,
			label_button,
			reset,
			refresh,
			settings,
			conflicts,
		}
	}
}

impl ComponentContent for Bar {
	fn element(&self) -> &Element {
		&self.element
	}
}
