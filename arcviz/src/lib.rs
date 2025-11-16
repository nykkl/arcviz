use js_sys::Function;
use webbit::Component;
use ui::App;
use wasm_bindgen::prelude::*;
use web_sys::window;

use webbit::io::FileIOHandler;

pub mod adapters;
pub mod common;
pub mod model;
pub mod render;
pub mod ui;

#[wasm_bindgen]
pub struct Arcviz {
	app: Component<App>,
}
#[wasm_bindgen]
impl Arcviz {
	#[wasm_bindgen(constructor)]
	pub fn new(read: Function, write: Function) -> Self {
		Self { app: Component::make(App::new(FileIOHandler::new(read, write))) }
	}
	#[wasm_bindgen]
	pub fn mount(&self) {
		let doc = window().unwrap().document().unwrap();
		let body = doc.body().unwrap();
		self.app.mount_in(&body);
	}
}
