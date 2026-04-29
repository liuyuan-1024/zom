//! 命令规范声明的聚合入口。

mod editor;
mod workspace;

use crate::command::kind::CommandKindSpec;

/// 汇总 editor/workspace 领域的全部命令规范。
pub(super) fn collect_specs() -> Vec<CommandKindSpec> {
    let mut specs = Vec::new();

    // Editor 语义目录。
    specs.extend_from_slice(editor::SPECS);

    // Workspace 顶层动作与聚焦语义目录。
    specs.extend_from_slice(workspace::actions::SPECS);
    specs.extend_from_slice(workspace::pane::SPECS);
    specs.extend_from_slice(workspace::panels::SPECS);
    specs.extend_from_slice(workspace::overlays::SPECS);

    // Workspace 面板内域语义目录。
    specs.extend_from_slice(workspace::file_tree::SPECS);
    specs.extend_from_slice(workspace::tab::SPECS);
    specs
}
