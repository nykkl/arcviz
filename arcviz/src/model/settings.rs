#[derive(Clone)]
pub struct Settings {
	pub show_grid: bool,
	pub show_conflicts: bool,
	pub show_labels: bool,
	pub snap_to_grid: bool,
	pub integrate_on_move: bool,
	pub auto_open_context_menu: bool,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			show_grid: true,
			show_conflicts: true,
			show_labels: true,
			snap_to_grid: false,
			integrate_on_move: true,
			auto_open_context_menu: false,
		}
	}
}
