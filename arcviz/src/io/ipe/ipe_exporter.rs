const FILE_TEMPLATE: &str = include_str!("./export_template.ipe");
const VERTEX_TEMPLATE: &str = include_str!("./vertex_template.ipe");
const ARC_TEMPLATE: &str = include_str!("./arc_template.ipe");

/// Exports the data to a representation in the .ipe format used by the ['Ipe extensible drawing editor'](https://ipe.otfried.org/).
/// (Since Ipe is a general purpose drawing editor that supports lots of constructs that have nothing to do with this application, there is no meaningful way to import from that format again.)
pub struct IpeExporter {
}
