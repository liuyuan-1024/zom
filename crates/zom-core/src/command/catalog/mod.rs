//! 命令目录（Command Catalog）。
//! 单一声明源：所有命令元信息集中在 `CommandSpec` 分片声明并汇总投影。

use std::{collections::HashMap, sync::LazyLock};

mod specs;
mod types;

pub use types::{
    CommandId, CommandKey, CommandMeta, CommandShortcut, CommandSpec, DefaultShortcutBinding,
    ShortcutScope,
};

use super::{Command, EditorCommand};

impl Command {
    /// 返回命令的稳定 Key。
    pub fn key(&self) -> CommandKey {
        command_key(self)
    }

    /// 返回命令的稳定字符串 ID。
    pub fn id(&self) -> CommandId {
        command_id(self)
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

/// 读取单一声明源中的所有命令定义。
pub fn command_specs() -> &'static [CommandSpec] {
    COMMAND_SPECS.as_slice()
}

/// 从命令语义解析稳定 Key。
pub fn command_key(command: &Command) -> CommandKey {
    if let Some(key) = dynamic_command_key(command) {
        return key;
    }

    *COMMAND_KEY_LOOKUP
        .get(command)
        .expect("all static commands must be declared in CommandSpec slices with factory")
}

/// 读取命令的稳定字符串 ID。
pub fn command_id(command: &Command) -> CommandId {
    command_spec(command).meta.id
}

/// 读取命令元信息。
pub fn command_meta(command: &Command) -> CommandMeta {
    command_spec(command).meta
}

/// 通过 Key 查询命令声明。
pub fn command_spec_by_key(key: CommandKey) -> Option<&'static CommandSpec> {
    command_specs().iter().find(|spec| spec.key == key)
}

/// 通过 ID 查询命令声明。
pub fn command_spec_by_id(id: CommandId) -> Option<&'static CommandSpec> {
    command_specs().iter().find(|spec| spec.meta.id == id)
}

/// 通过命令查询命令声明。
pub fn command_spec(command: &Command) -> &'static CommandSpec {
    let key = command_key(command);
    command_spec_by_key(key).expect("all command keys must be declared in CommandSpec slices")
}

/// 读取命令默认快捷键。
pub fn default_shortcuts(command: &Command) -> Vec<CommandShortcut> {
    command_spec(command).default_shortcuts.to_vec()
}

/// 根据 Key 反向构造可执行命令。
/// 注意：`EditorInsertText` 需要动态 payload，无法在此函数中构造。
pub fn command_from_key(key: CommandKey) -> Option<Command> {
    command_spec_by_key(key).and_then(|spec| (spec.factory)())
}

/// 从统一命令声明自动投影默认快捷键绑定（输入层可直接消费）。
pub fn default_shortcut_bindings() -> Vec<DefaultShortcutBinding> {
    let mut bindings = Vec::new();

    for spec in command_specs() {
        if spec.default_shortcuts.is_empty() {
            continue;
        }
        let Some(command) = command_from_key(spec.key) else {
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

static COMMAND_SPECS: LazyLock<Vec<CommandSpec>> = LazyLock::new(specs::collect_specs);
static COMMAND_KEY_LOOKUP: LazyLock<HashMap<Command, CommandKey>> =
    LazyLock::new(build_command_key_lookup);

fn dynamic_command_key(command: &Command) -> Option<CommandKey> {
    match command {
        Command::Editor(EditorCommand::InsertText(_)) => Some(CommandKey::EditorInsertText),
        _ => None,
    }
}

fn build_command_key_lookup() -> HashMap<Command, CommandKey> {
    let mut lookup = HashMap::new();

    for spec in command_specs() {
        let Some(command) = (spec.factory)() else {
            continue;
        };
        lookup.insert(command, spec.key);
    }

    lookup
}

#[cfg(test)]
mod tests {
    use crate::{Command, FocusTarget, command::WorkspaceCommand};

    use super::{
        CommandId, CommandKey, command_from_key, command_meta, command_spec_by_id,
        command_spec_by_key, default_shortcut_bindings,
    };

    #[test]
    fn command_meta_provides_stable_ids_for_parameterized_commands() {
        let command = Command::from(WorkspaceCommand::FocusPanel(FocusTarget::FileTreePanel));
        let meta = command_meta(&command);

        assert_eq!(meta.id, CommandId("workspace.focus_panel.file_tree"));
        assert_eq!(meta.title, "Focus File Tree Panel");
    }

    #[test]
    fn command_specs_are_queryable_by_key() {
        let spec = command_spec_by_key(CommandKey::WorkspaceOpenProjectPicker)
            .expect("open project picker should be declared");

        assert_eq!(spec.meta.id, CommandId("workspace.open_project_picker"));
    }

    #[test]
    fn command_specs_are_queryable_by_id() {
        let spec = command_spec_by_id(CommandId("workspace.open_settings"))
            .expect("open settings should be declared");

        assert_eq!(spec.key, CommandKey::WorkspaceOpenSettings);
    }

    #[test]
    fn default_shortcut_bindings_are_emitted_from_catalog() {
        let bindings = default_shortcut_bindings();
        let has_project_picker = bindings
            .iter()
            .any(|binding| binding.command == Command::from(WorkspaceCommand::OpenProjectPicker));
        let has_settings = bindings
            .iter()
            .any(|binding| binding.command == Command::from(WorkspaceCommand::OpenSettings));

        assert!(has_project_picker);
        assert!(has_settings);
    }

    #[test]
    fn editor_insert_text_declares_empty_default_shortcuts() {
        let spec = command_spec_by_key(CommandKey::EditorInsertText)
            .expect("editor.insert_text should be declared");

        assert!(spec.default_shortcuts.is_empty());
        assert_eq!(spec.meta.id, CommandId("editor.insert_text"));
    }

    #[test]
    fn command_from_key_returns_none_for_dynamic_payload_commands() {
        assert!(command_from_key(CommandKey::EditorInsertText).is_none());
        assert!(command_from_key(CommandKey::WorkspaceOpenSettings).is_some());
    }
}
