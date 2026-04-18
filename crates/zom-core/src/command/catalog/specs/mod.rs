mod editor;
mod workspace_actions;
mod workspace_file_tree;
mod workspace_panels;
mod workspace_tab;

use crate::command::catalog::CommandSpec;

pub(super) fn collect_specs() -> Vec<CommandSpec> {
    let mut specs = Vec::new();
    specs.extend_from_slice(editor::SPECS);
    specs.extend_from_slice(workspace_panels::SPECS);
    specs.extend_from_slice(workspace_actions::SPECS);
    specs.extend_from_slice(workspace_file_tree::SPECS);
    specs.extend_from_slice(workspace_tab::SPECS);
    specs
}
