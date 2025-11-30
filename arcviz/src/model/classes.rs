use crate::common::Number;

pub type SizeId = usize;

#[derive(Clone)]
pub struct Class {
	size: Number,
	color: String,
}
impl Class {
	pub fn new(size: Number, color: String) -> Self {
		Self { size, color }
	}
	pub fn size(&self) -> &Number {
		&self.size
	}
	pub fn color(&self) -> &str {
		&self.color
	}
}
pub fn generate_color() -> String {
	"white".to_string()
}

#[derive(Clone)]
pub struct Classes {
	items: Vec<Class>,
	default: Class,
}
impl Default for Classes {
	fn default() -> Self {
		Self {
			items: vec![
				Class::new(100.0, "red".to_string()),
				Class::new(200.0, "blue".to_string()),
				Class::new(300.0, "purple".to_string()),
				Class::new(400.0, "yellow".to_string()),
			],
			default: Class::new(300.0, "white".to_string()),
		}
	}
}
impl Classes {
	pub fn new(items: Vec<Class>, default: Class) -> Self {
		Self { items, default }
	}
	pub fn items(&self) -> &[Class] {
		&self.items
	}
	pub fn get_size(&self, id: SizeId) -> Number {
		self.items.get(id).unwrap_or(&self.default).size.clone()
	}
	pub fn get_color(&self, id: SizeId) -> &str {
		&self.items.get(id).unwrap_or(&self.default).color
	}
	pub fn previous(&self, id: SizeId) -> SizeId {
		if id <= 0 {
			return id;
		}
		return id - 1;
	}
	pub fn next(&self, id: SizeId) -> SizeId {
		if id >= self.items.len() - 1 {
			return id;
		}
		return id + 1;
	}
}
