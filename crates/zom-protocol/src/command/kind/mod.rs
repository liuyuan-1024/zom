//! 命令语义族目录（Command Kind Catalog）。
//! 单一声明源：所有命令元信息集中在 `CommandKindSpec` 分片声明并汇总。

use std::{collections::HashMap, sync::LazyLock};

mod command_kind;
mod mapping;
mod meta;
mod spec;
mod specs;

pub use command_kind::CommandKind;
pub use meta::{CommandKindId, CommandMeta};
pub use spec::CommandKindSpec;

use super::{CommandInvocation, EditorInvocation, FindReplaceAction};

impl CommandInvocation {
    /// 返回运行时调用所属的稳定语义族。
    pub fn kind(&self) -> CommandKind {
        command_kind(self)
    }

    /// 返回运行时调用所属语义族的稳定字符串 ID。
    pub fn kind_id(&self) -> CommandKindId {
        command_kind_id(self)
    }

    /// 返回命令元信息。
    pub fn meta(&self) -> CommandMeta {
        command_meta(self)
    }
}

/// 读取单一声明源中的所有命令语义族定义。
pub fn command_kind_specs() -> &'static [CommandKindSpec] {
    COMMAND_KIND_SPECS.as_slice()
}

/// 从运行时调用解析稳定语义族。
pub fn command_kind(command: &CommandInvocation) -> CommandKind {
    // 动态载荷命令先走专门分流，避免把 payload 纳入静态查表键。
    if let Some(kind) = dynamic_command_kind(command) {
        return kind;
    }

    *COMMAND_KIND_LOOKUP
        .get(command)
        .expect("所有静态命令调用都必须在 CommandKindSpec 切片中声明。")
}

/// 读取命令语义族稳定字符串 ID。
pub fn command_kind_id(command: &CommandInvocation) -> CommandKindId {
    command_kind_spec(command).meta.id
}

/// 读取命令元信息。
pub fn command_meta(command: &CommandInvocation) -> CommandMeta {
    command_kind_spec(command).meta
}

/// 通过语义族查询命令声明。
pub fn command_kind_spec_by_kind(kind: CommandKind) -> Option<&'static CommandKindSpec> {
    command_kind_specs().iter().find(|spec| spec.kind == kind)
}

/// 通过语义族 ID 查询命令声明。
pub fn command_kind_spec_by_id(id: CommandKindId) -> Option<&'static CommandKindSpec> {
    command_kind_specs().iter().find(|spec| spec.meta.id == id)
}

/// 通过运行时调用查询命令声明。
pub fn command_kind_spec(command: &CommandInvocation) -> &'static CommandKindSpec {
    let kind = command_kind(command);
    command_kind_spec_by_kind(kind).expect("必须声明所有命令类型。")
}

static COMMAND_KIND_SPECS: LazyLock<Vec<CommandKindSpec>> = LazyLock::new(specs::collect_specs);
static COMMAND_KIND_LOOKUP: LazyLock<HashMap<CommandInvocation, CommandKind>> =
    LazyLock::new(build_command_kind_lookup);

/// 提取需要动态 payload 参与判定的命令类型；找不到时返回 `None`。
fn dynamic_command_kind(command: &CommandInvocation) -> Option<CommandKind> {
    match command {
        CommandInvocation::Editor(EditorInvocation::InsertText { .. }) => {
            Some(CommandKind::EditorInsertText)
        }
        CommandInvocation::Editor(EditorInvocation::FindReplace { request }) => {
            Some(match request.action {
                FindReplaceAction::FindNext => CommandKind::EditorFindNext,
                FindReplaceAction::FindPrev => CommandKind::EditorFindPrev,
                FindReplaceAction::ReplaceNext => CommandKind::EditorReplaceNext,
                FindReplaceAction::ReplaceAll => CommandKind::EditorReplaceAll,
            })
        }
        _ => None,
    }
}

/// 构建“静态调用 -> CommandKind”查找表。
///
/// 仅收录可静态构造的命令；动态命令由 `dynamic_command_kind` 兜底。
fn build_command_kind_lookup() -> HashMap<CommandInvocation, CommandKind> {
    let mut lookup = HashMap::new();

    for spec in command_kind_specs() {
        let Some(invocation) = mapping::invocation_for_kind(spec.kind) else {
            continue;
        };
        lookup.insert(invocation, spec.kind);
    }

    lookup
}

#[cfg(test)]
mod tests {
    use crate::{
        CommandInvocation, EditorInvocation, FindReplaceAction, FindReplaceRequest, FocusTarget,
        OverlayTarget, WorkspaceAction,
    };

    use super::{
        CommandKind, CommandKindId, command_kind, command_kind_spec_by_id,
        command_kind_spec_by_kind, command_meta,
    };

    #[test]
    fn command_meta_provides_stable_ids_for_parameterized_commands() {
        let command =
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::FileTreePanel));
        let meta = command_meta(&command);

        assert_eq!(meta.id, CommandKindId("workspace.focus_panel.file_tree"));
        assert!(!meta.title.is_empty());
    }

    #[test]
    fn command_specs_are_queryable_by_kind() {
        let spec = command_kind_spec_by_kind(CommandKind::WorkspaceOpenProjectPicker)
            .expect("open project picker should be declared");

        assert_eq!(spec.meta.id, CommandKindId("workspace.open_project_picker"));
    }

    #[test]
    fn command_specs_are_queryable_by_id() {
        let spec = command_kind_spec_by_id(CommandKindId("workspace.focus_overlay.settings"))
            .expect("focus settings overlay should be declared");

        assert_eq!(
            spec.kind,
            CommandKind::WorkspaceFocusOverlay(OverlayTarget::Settings)
        );
    }

    #[test]
    fn command_kind_maps_static_invocations() {
        let command = CommandInvocation::from(WorkspaceAction::OpenProjectPicker);
        assert_eq!(
            command_kind(&command),
            CommandKind::WorkspaceOpenProjectPicker
        );
    }

    #[test]
    fn editor_insert_text_uses_dynamic_kind_mapping() {
        let command = CommandInvocation::from(EditorInvocation::insert_text("hello"));
        assert_eq!(command_kind(&command), CommandKind::EditorInsertText);
    }

    #[test]
    fn editor_find_replace_uses_dynamic_kind_mapping() {
        let command =
            CommandInvocation::from(EditorInvocation::find_replace(FindReplaceRequest::new(
                "hello",
                "world",
                FindReplaceAction::ReplaceAll,
                true,
                false,
                false,
            )));
        assert_eq!(command_kind(&command), CommandKind::EditorReplaceAll);
    }
}
