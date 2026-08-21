fn main() {
	let css = webbit_style::StylesheetBuilder::default().build("style/app.sass").unwrap();
	std::fs::write(format!("{}/app.css", std::env::var("OUT_DIR").unwrap()), css).unwrap();
	println!("cargo:rerun-if-changed=style");
}
