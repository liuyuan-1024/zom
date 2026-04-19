mod editor;
mod workspace;

use crate::command::kind::CommandKindSpec;

pub(super) fn collect_specs() -> Vec<CommandKindSpec> {
    let mut specs = Vec::new();
    specs.extend_from_slice(editor::SPECS);
    specs.extend_from_slice(workspace::panels::SPECS);
    specs.extend_from_slice(workspace::overlays::SPECS);
    specs.extend_from_slice(workspace::actions::SPECS);
    specs.extend_from_slice(workspace::file_tree::SPECS);
    specs.extend_from_slice(workspace::tab::SPECS);
    specs
}
