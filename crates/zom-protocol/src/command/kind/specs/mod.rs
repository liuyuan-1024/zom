//! 命令规范声明的聚合入口。

mod editor;
mod workspace;

use crate::command::kind::CommandKindSpec;

/// 汇总 editor/workspace 领域的全部命令规范。
pub(super) fn collect_specs() -> Vec<CommandKindSpec> {
    let mut specs = Vec::new();
    specs.extend_from_slice(editor::SPECS);
    specs.extend_from_slice(workspace::pane::SPECS);
    specs.extend_from_slice(workspace::panels::SPECS);
    specs.extend_from_slice(workspace::overlays::SPECS);
    specs.extend_from_slice(workspace::actions::SPECS);
    specs.extend_from_slice(workspace::file_tree::SPECS);
    specs.extend_from_slice(workspace::tab::SPECS);
    specs.extend_from_slice(workspace::notification::SPECS);
    specs
}
