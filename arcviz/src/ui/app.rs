use web_sys::HtmlDivElement;

use webbit::{
	components::SideBar,
	elements::{div, on, styled},
	io::FileIOHandler,
	Component, ComponentContent,
};

use super::{Bar, Workspace};

pub struct App {
	element: HtmlDivElement,
	bar: Component<Bar>,
	workspace: Component<Workspace>,
	dock: Component<SideBar>,
}

impl App {
	pub fn new(io: FileIOHandler) -> Self {
		let element = styled(div(), "app");
		let main = on(&element, styled(div(), "main"));

		let workspace = Workspace::new(io);
		let dock = SideBar::new(
			element.clone().into(),
			"sidebar",
			"sidebar-resize-handle",
			"sidebar-internal",
			"sidebar-controls",
			"sidebar-control-button",
			"sidebar-control-title",
			"sidebar-content",
		);

		let bar = Component::make(Bar::new(workspace.clone(), dock.clone()));
		bar.mount_in(&main);
		let workspace = Component::make_sharable(workspace);
		workspace.mount_in(&main);

		Self { element, bar, workspace, dock: Component::make_sharable(dock) }
	}
}

impl ComponentContent for App {
	fn element(&self) -> &web_sys::Element {
		&self.element
	}
}
