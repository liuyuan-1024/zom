//! 命令语义族目录（Command Kind Catalog）。
//! 单一声明源：所有命令元信息集中在 `CommandKindSpec` 分片声明并汇总投影。

use std::{collections::HashMap, sync::LazyLock};

mod specs;
mod types;

pub use types::{
    Buildability, CommandKind, CommandKindId, CommandKindSpec, CommandMeta, CommandShortcut,
    DefaultShortcutBinding, ShortcutScope,
};

use super::{CommandInvocation, EditorInvocation};

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

    /// 返回命令默认快捷键。
    pub fn default_shortcuts(&self) -> Vec<CommandShortcut> {
        default_shortcuts(self)
    }
}

/// 读取单一声明源中的所有命令语义族定义。
pub fn command_kind_specs() -> &'static [CommandKindSpec] {
    COMMAND_KIND_SPECS.as_slice()
}

/// 从运行时调用解析稳定语义族。
pub fn command_kind(command: &CommandInvocation) -> CommandKind {
    if let Some(kind) = dynamic_command_kind(command) {
        return kind;
    }

    *COMMAND_KIND_LOOKUP
        .get(command)
        .expect("all static command invocations must be declared in CommandKindSpec slices")
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
    command_kind_spec_by_kind(kind).expect("all command kinds must be declared")
}

/// 读取命令默认快捷键。
pub fn default_shortcuts(command: &CommandInvocation) -> Vec<CommandShortcut> {
    command_kind_spec(command).default_shortcuts.to_vec()
}

/// 根据语义族反向构造运行时调用。
/// 注意：`EditorInsertText` 需要动态 payload，无法在此函数中构造。
pub fn invocation_from_kind(kind: CommandKind) -> Option<CommandInvocation> {
    let spec = command_kind_spec_by_kind(kind)?;
    match spec.buildability {
        Buildability::Static(builder) => Some(builder()),
        Buildability::RequiresArgs => None,
    }
}

/// 从统一命令声明自动投影默认快捷键绑定（输入层可直接消费）。
pub fn default_shortcut_bindings() -> Vec<DefaultShortcutBinding> {
    let mut bindings = Vec::new();

    for spec in command_kind_specs() {
        if spec.default_shortcuts.is_empty() {
            continue;
        }
        let Some(command) = invocation_from_kind(spec.kind) else {
            continue;
        };

        for shortcut in spec.default_shortcuts {
            bindings.push(DefaultShortcutBinding {
                command: command.clone(),
                shortcut: *shortcut,
            });
        }
    }

    bindings
}

static COMMAND_KIND_SPECS: LazyLock<Vec<CommandKindSpec>> = LazyLock::new(specs::collect_specs);
static COMMAND_KIND_LOOKUP: LazyLock<HashMap<CommandInvocation, CommandKind>> =
    LazyLock::new(build_command_kind_lookup);

fn dynamic_command_kind(command: &CommandInvocation) -> Option<CommandKind> {
    match command {
        CommandInvocation::Editor(EditorInvocation::InsertText { .. }) => {
            Some(CommandKind::EditorInsertText)
        }
        _ => None,
    }
}

fn build_command_kind_lookup() -> HashMap<CommandInvocation, CommandKind> {
    let mut lookup = HashMap::new();

    for spec in command_kind_specs() {
        let Buildability::Static(builder) = spec.buildability else {
            continue;
        };
        lookup.insert(builder(), spec.kind);
    }

    lookup
}

#[cfg(test)]
mod tests {
    use crate::{CommandInvocation, FocusTarget, WorkspaceAction};

    use super::{
        CommandKind, CommandKindId, command_kind_spec_by_id, command_kind_spec_by_kind,
        command_meta, default_shortcut_bindings, invocation_from_kind,
    };

    #[test]
    fn command_meta_provides_stable_ids_for_parameterized_commands() {
        let command =
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::FileTreePanel));
        let meta = command_meta(&command);

        assert_eq!(meta.id, CommandKindId("workspace.focus_panel.file_tree"));
        assert_eq!(meta.title, "Focus File Tree Panel");
    }

    #[test]
    fn command_specs_are_queryable_by_kind() {
        let spec = command_kind_spec_by_kind(CommandKind::WorkspaceOpenProjectPicker)
            .expect("open project picker should be declared");

        assert_eq!(spec.meta.id, CommandKindId("workspace.open_project_picker"));
    }

    #[test]
    fn command_specs_are_queryable_by_id() {
        let spec = command_kind_spec_by_id(CommandKindId("workspace.open_settings"))
            .expect("open settings should be declared");

        assert_eq!(spec.kind, CommandKind::WorkspaceOpenSettings);
    }

    #[test]
    fn default_shortcut_bindings_are_emitted_from_catalog() {
        let bindings = default_shortcut_bindings();
        let has_project_picker = bindings.iter().any(|binding| {
            binding.command == CommandInvocation::from(WorkspaceAction::OpenProjectPicker)
        });
        let has_settings = bindings.iter().any(|binding| {
            binding.command == CommandInvocation::from(WorkspaceAction::OpenSettings)
        });

        assert!(has_project_picker);
        assert!(has_settings);
    }

    #[test]
    fn editor_insert_text_declares_empty_default_shortcuts() {
        let spec = command_kind_spec_by_kind(CommandKind::EditorInsertText)
            .expect("editor.insert_text should be declared");

        assert!(spec.default_shortcuts.is_empty());
        assert_eq!(spec.meta.id, CommandKindId("editor.insert_text"));
    }

    #[test]
    fn invocation_from_kind_returns_none_for_dynamic_payload_commands() {
        assert!(invocation_from_kind(CommandKind::EditorInsertText).is_none());
        assert!(invocation_from_kind(CommandKind::WorkspaceOpenSettings).is_some());
    }
}
