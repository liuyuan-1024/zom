mod editor;
mod workspace;

use crate::command::catalog::CommandSpec;

pub(super) fn collect_specs() -> Vec<CommandSpec> {
    let mut specs = Vec::new();
    specs.extend_from_slice(editor::SPECS);
    specs.extend_from_slice(workspace::panels::SPECS);
    specs.extend_from_slice(workspace::actions::SPECS);
    specs.extend_from_slice(workspace::file_tree::SPECS);
    specs.extend_from_slice(workspace::tab::SPECS);
    specs
}
