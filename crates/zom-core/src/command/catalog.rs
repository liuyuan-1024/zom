//! 命令目录（Command Catalog）。
//! 统一维护命令语义的标识、描述与默认快捷键元数据。

use crate::{FocusTarget, KeyCode, Keystroke, Modifiers};

use super::{Command, EditorCommand, FileTreeCommand, TabCommand, WorkspaceCommand};

/// 命令的稳定标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    EditorInsertText,
    EditorInsertNewline,
    EditorMoveLeft,
    EditorMoveRight,
    EditorMoveUp,
    EditorMoveDown,
    EditorMoveToStart,
    EditorMoveToEnd,
    EditorMovePageUp,
    EditorMovePageDown,
    EditorDeleteBackward,
    EditorDeleteForward,
    EditorDeleteWordBackward,
    EditorDeleteWordForward,
    EditorUndo,
    EditorRedo,
    EditorSelectAll,

    WorkspaceFocusPanel(FocusTarget),
    WorkspaceTogglePanel(FocusTarget),
    WorkspaceOpenProjectPicker,
    WorkspaceOpenSettings,
    WorkspaceOpenCodeActions,
    WorkspaceStartDebugging,
    WorkspaceFileTreeSelectPrev,
    WorkspaceFileTreeSelectNext,
    WorkspaceFileTreeExpandOrDescend,
    WorkspaceFileTreeCollapseOrAscend,
    WorkspaceFileTreeActivateSelection,
    WorkspaceTabCloseActive,
    WorkspaceTabActivatePrev,
    WorkspaceTabActivateNext,
}

/// 命令目录中的人类可读描述。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandDescriptor {
    /// 稳定 ID，供跨层引用与文档检索。
    pub id: &'static str,
    /// 简短标题。
    pub title: &'static str,
    /// 语义说明。
    pub description: &'static str,
}

impl CommandDescriptor {
    /// 创建一条命令描述。
    pub const fn new(id: &'static str, title: &'static str, description: &'static str) -> Self {
        Self {
            id,
            title,
            description,
        }
    }
}

/// 默认快捷键作用域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    /// 全局快捷键。
    Global,
    /// 仅在指定焦点下生效。
    Focus(FocusTarget),
}

/// 默认快捷键额外触发条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutWhen {
    /// 无额外条件。
    Always,
}

/// 默认快捷键适用平台。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutPlatform {
    /// 所有平台。
    Any,
    /// macOS。
    MacOS,
    /// Windows。
    Windows,
    /// Linux。
    Linux,
}

/// 命令目录里定义的一条默认快捷键。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandShortcut {
    /// 作用域。
    pub scope: ShortcutScope,
    /// 按键定义。
    pub keystroke: Keystroke,
    /// 触发条件。
    pub when: ShortcutWhen,
    /// 适用平台。
    pub platform: ShortcutPlatform,
    /// 优先级（越大越优先）。
    pub priority: u8,
}

impl CommandShortcut {
    /// 创建一条默认快捷键。
    pub fn new(scope: ShortcutScope, keystroke: Keystroke) -> Self {
        Self {
            scope,
            keystroke,
            when: ShortcutWhen::Always,
            platform: ShortcutPlatform::Any,
            priority: 0,
        }
    }

    /// 设置触发条件。
    pub fn with_when(mut self, when: ShortcutWhen) -> Self {
        self.when = when;
        self
    }

    /// 设置平台。
    pub fn with_platform(mut self, platform: ShortcutPlatform) -> Self {
        self.platform = platform;
        self
    }

    /// 设置优先级。
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// 一条“命令 + 默认快捷键”绑定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultShortcutBinding {
    /// 命令语义。
    pub command: Command,
    /// 对应快捷键。
    pub shortcut: CommandShortcut,
}

/// 目录条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCatalogEntry {
    /// 命令 ID。
    pub id: CommandId,
    /// 命令描述。
    pub descriptor: CommandDescriptor,
    /// 默认快捷键集合（可为空）。
    pub default_shortcuts: Vec<CommandShortcut>,
}

impl Command {
    /// 返回命令的稳定 ID。
    pub fn id(&self) -> CommandId {
        command_id(self)
    }

    /// 返回命令描述。
    pub fn descriptor(&self) -> CommandDescriptor {
        command_descriptor(self)
    }

    /// 返回命令默认快捷键。
    pub fn default_shortcuts(&self) -> Vec<CommandShortcut> {
        default_shortcuts(self)
    }
}

/// 读取命令对应的稳定 ID。
pub fn command_id(command: &Command) -> CommandId {
    match command {
        Command::Editor(command) => match command {
            EditorCommand::InsertText(_) => CommandId::EditorInsertText,
            EditorCommand::InsertNewline => CommandId::EditorInsertNewline,
            EditorCommand::MoveLeft => CommandId::EditorMoveLeft,
            EditorCommand::MoveRight => CommandId::EditorMoveRight,
            EditorCommand::MoveUp => CommandId::EditorMoveUp,
            EditorCommand::MoveDown => CommandId::EditorMoveDown,
            EditorCommand::MoveToStart => CommandId::EditorMoveToStart,
            EditorCommand::MoveToEnd => CommandId::EditorMoveToEnd,
            EditorCommand::MovePageUp => CommandId::EditorMovePageUp,
            EditorCommand::MovePageDown => CommandId::EditorMovePageDown,
            EditorCommand::DeleteBackward => CommandId::EditorDeleteBackward,
            EditorCommand::DeleteForward => CommandId::EditorDeleteForward,
            EditorCommand::DeleteWordBackward => CommandId::EditorDeleteWordBackward,
            EditorCommand::DeleteWordForward => CommandId::EditorDeleteWordForward,
            EditorCommand::Undo => CommandId::EditorUndo,
            EditorCommand::Redo => CommandId::EditorRedo,
            EditorCommand::SelectAll => CommandId::EditorSelectAll,
        },
        Command::Workspace(command) => match command {
            WorkspaceCommand::FocusPanel(target) => CommandId::WorkspaceFocusPanel(*target),
            WorkspaceCommand::TogglePanel(target) => CommandId::WorkspaceTogglePanel(*target),
            WorkspaceCommand::OpenProjectPicker => CommandId::WorkspaceOpenProjectPicker,
            WorkspaceCommand::OpenSettings => CommandId::WorkspaceOpenSettings,
            WorkspaceCommand::OpenCodeActions => CommandId::WorkspaceOpenCodeActions,
            WorkspaceCommand::StartDebugging => CommandId::WorkspaceStartDebugging,
            WorkspaceCommand::FileTree(command) => match command {
                FileTreeCommand::SelectPrev => CommandId::WorkspaceFileTreeSelectPrev,
                FileTreeCommand::SelectNext => CommandId::WorkspaceFileTreeSelectNext,
                FileTreeCommand::ExpandOrDescend => CommandId::WorkspaceFileTreeExpandOrDescend,
                FileTreeCommand::CollapseOrAscend => CommandId::WorkspaceFileTreeCollapseOrAscend,
                FileTreeCommand::ActivateSelection => CommandId::WorkspaceFileTreeActivateSelection,
            },
            WorkspaceCommand::Tab(command) => match command {
                TabCommand::CloseActiveTab => CommandId::WorkspaceTabCloseActive,
                TabCommand::ActivatePrevTab => CommandId::WorkspaceTabActivatePrev,
                TabCommand::ActivateNextTab => CommandId::WorkspaceTabActivateNext,
            },
        },
    }
}

/// 读取命令描述。
pub fn command_descriptor(command: &Command) -> CommandDescriptor {
    descriptor_for_id(command_id(command))
}

/// 读取命令默认快捷键。
pub fn default_shortcuts(command: &Command) -> Vec<CommandShortcut> {
    default_shortcuts_for_id(command_id(command))
}

/// 构建命令目录。
pub fn command_catalog() -> Vec<CommandCatalogEntry> {
    all_command_ids()
        .into_iter()
        .map(|id| CommandCatalogEntry {
            id,
            descriptor: descriptor_for_id(id),
            default_shortcuts: default_shortcuts_for_id(id),
        })
        .collect()
}

/// 构建默认快捷键绑定（输入层可直接消费）。
pub fn default_shortcut_bindings() -> Vec<DefaultShortcutBinding> {
    let mut bindings = Vec::new();

    let commands = [
        Command::from(WorkspaceCommand::FocusPanel(FocusTarget::FileTreePanel)),
        Command::from(WorkspaceCommand::FocusPanel(FocusTarget::GitPanel)),
        Command::from(WorkspaceCommand::FocusPanel(FocusTarget::OutlinePanel)),
        Command::from(WorkspaceCommand::FocusPanel(FocusTarget::ProjectSearch)),
        Command::from(WorkspaceCommand::FocusPanel(FocusTarget::Terminal)),
        Command::from(WorkspaceCommand::OpenProjectPicker),
        Command::from(WorkspaceCommand::OpenSettings),
        Command::from(WorkspaceCommand::OpenCodeActions),
        Command::from(WorkspaceCommand::StartDebugging),
        Command::from(WorkspaceCommand::FocusPanel(FocusTarget::Notification)),
        Command::from(FileTreeCommand::SelectPrev),
        Command::from(FileTreeCommand::SelectNext),
        Command::from(FileTreeCommand::ExpandOrDescend),
        Command::from(FileTreeCommand::CollapseOrAscend),
        Command::from(FileTreeCommand::ActivateSelection),
    ];

    for command in commands {
        for shortcut in default_shortcuts(&command) {
            bindings.push(DefaultShortcutBinding {
                command: command.clone(),
                shortcut,
            });
        }
    }

    for panel in FocusTarget::VISIBILITY_MANAGED_PANELS {
        let command = Command::from(WorkspaceCommand::TogglePanel(panel));
        for shortcut in default_shortcuts(&command) {
            bindings.push(DefaultShortcutBinding {
                command: command.clone(),
                shortcut,
            });
        }
    }

    bindings
}

fn all_command_ids() -> Vec<CommandId> {
    let mut ids = vec![
        CommandId::EditorInsertText,
        CommandId::EditorInsertNewline,
        CommandId::EditorMoveLeft,
        CommandId::EditorMoveRight,
        CommandId::EditorMoveUp,
        CommandId::EditorMoveDown,
        CommandId::EditorMoveToStart,
        CommandId::EditorMoveToEnd,
        CommandId::EditorMovePageUp,
        CommandId::EditorMovePageDown,
        CommandId::EditorDeleteBackward,
        CommandId::EditorDeleteForward,
        CommandId::EditorDeleteWordBackward,
        CommandId::EditorDeleteWordForward,
        CommandId::EditorUndo,
        CommandId::EditorRedo,
        CommandId::EditorSelectAll,
        CommandId::WorkspaceOpenProjectPicker,
        CommandId::WorkspaceOpenSettings,
        CommandId::WorkspaceOpenCodeActions,
        CommandId::WorkspaceStartDebugging,
        CommandId::WorkspaceFileTreeSelectPrev,
        CommandId::WorkspaceFileTreeSelectNext,
        CommandId::WorkspaceFileTreeExpandOrDescend,
        CommandId::WorkspaceFileTreeCollapseOrAscend,
        CommandId::WorkspaceFileTreeActivateSelection,
        CommandId::WorkspaceTabCloseActive,
        CommandId::WorkspaceTabActivatePrev,
        CommandId::WorkspaceTabActivateNext,
    ];

    ids.extend(
        FocusTarget::ALL
            .into_iter()
            .map(CommandId::WorkspaceFocusPanel),
    );
    ids.extend(
        FocusTarget::ALL
            .into_iter()
            .map(CommandId::WorkspaceTogglePanel),
    );

    ids
}

fn descriptor_for_id(id: CommandId) -> CommandDescriptor {
    match id {
        CommandId::EditorInsertText => CommandDescriptor::new(
            "editor.insert_text",
            "Insert Text",
            "Insert provided text at the current caret position.",
        ),
        CommandId::EditorInsertNewline => CommandDescriptor::new(
            "editor.insert_newline",
            "Insert Newline",
            "Insert a newline at the current caret position.",
        ),
        CommandId::EditorMoveLeft => CommandDescriptor::new(
            "editor.move_left",
            "Move Left",
            "Move the caret one character to the left.",
        ),
        CommandId::EditorMoveRight => CommandDescriptor::new(
            "editor.move_right",
            "Move Right",
            "Move the caret one character to the right.",
        ),
        CommandId::EditorMoveUp => {
            CommandDescriptor::new("editor.move_up", "Move Up", "Move the caret one line up.")
        }
        CommandId::EditorMoveDown => CommandDescriptor::new(
            "editor.move_down",
            "Move Down",
            "Move the caret one line down.",
        ),
        CommandId::EditorMoveToStart => CommandDescriptor::new(
            "editor.move_to_start",
            "Move To Start",
            "Move the caret to the beginning of the current line.",
        ),
        CommandId::EditorMoveToEnd => CommandDescriptor::new(
            "editor.move_to_end",
            "Move To End",
            "Move the caret to the end of the current line.",
        ),
        CommandId::EditorMovePageUp => {
            CommandDescriptor::new("editor.page_up", "Page Up", "Move the caret one page up.")
        }
        CommandId::EditorMovePageDown => CommandDescriptor::new(
            "editor.page_down",
            "Page Down",
            "Move the caret one page down.",
        ),
        CommandId::EditorDeleteBackward => CommandDescriptor::new(
            "editor.delete_backward",
            "Delete Backward",
            "Delete one character backward.",
        ),
        CommandId::EditorDeleteForward => CommandDescriptor::new(
            "editor.delete_forward",
            "Delete Forward",
            "Delete one character forward.",
        ),
        CommandId::EditorDeleteWordBackward => CommandDescriptor::new(
            "editor.delete_word_backward",
            "Delete Word Backward",
            "Delete one word backward.",
        ),
        CommandId::EditorDeleteWordForward => CommandDescriptor::new(
            "editor.delete_word_forward",
            "Delete Word Forward",
            "Delete one word forward.",
        ),
        CommandId::EditorUndo => {
            CommandDescriptor::new("editor.undo", "Undo", "Undo the most recent edit.")
        }
        CommandId::EditorRedo => {
            CommandDescriptor::new("editor.redo", "Redo", "Redo the most recently undone edit.")
        }
        CommandId::EditorSelectAll => CommandDescriptor::new(
            "editor.select_all",
            "Select All",
            "Select all content in the current editor.",
        ),

        CommandId::WorkspaceFocusPanel(target) => CommandDescriptor::new(
            focus_panel_id(target),
            focus_panel_title(target),
            focus_panel_description(target),
        ),
        CommandId::WorkspaceTogglePanel(target) => CommandDescriptor::new(
            toggle_panel_id(target),
            toggle_panel_title(target),
            toggle_panel_description(target),
        ),
        CommandId::WorkspaceOpenProjectPicker => CommandDescriptor::new(
            "workspace.open_project_picker",
            "Open Project Picker",
            "Open the project folder picker.",
        ),
        CommandId::WorkspaceOpenSettings => CommandDescriptor::new(
            "workspace.open_settings",
            "Open Settings",
            "Open application settings.",
        ),
        CommandId::WorkspaceOpenCodeActions => CommandDescriptor::new(
            "workspace.open_code_actions",
            "Open Code Actions",
            "Open the code actions menu.",
        ),
        CommandId::WorkspaceStartDebugging => CommandDescriptor::new(
            "workspace.start_debugging",
            "Start Debugging",
            "Start or continue debugging.",
        ),
        CommandId::WorkspaceFileTreeSelectPrev => CommandDescriptor::new(
            "workspace.file_tree.select_prev",
            "File Tree Select Previous",
            "Move file-tree selection to the previous visible node.",
        ),
        CommandId::WorkspaceFileTreeSelectNext => CommandDescriptor::new(
            "workspace.file_tree.select_next",
            "File Tree Select Next",
            "Move file-tree selection to the next visible node.",
        ),
        CommandId::WorkspaceFileTreeExpandOrDescend => CommandDescriptor::new(
            "workspace.file_tree.expand_or_descend",
            "File Tree Expand Or Descend",
            "Expand selected folder or descend into its first child.",
        ),
        CommandId::WorkspaceFileTreeCollapseOrAscend => CommandDescriptor::new(
            "workspace.file_tree.collapse_or_ascend",
            "File Tree Collapse Or Ascend",
            "Collapse selected folder or move selection to parent node.",
        ),
        CommandId::WorkspaceFileTreeActivateSelection => CommandDescriptor::new(
            "workspace.file_tree.activate_selection",
            "File Tree Activate Selection",
            "Activate selected file-tree node.",
        ),
        CommandId::WorkspaceTabCloseActive => CommandDescriptor::new(
            "workspace.tab.close_active",
            "Close Active Tab",
            "Close the currently active tab.",
        ),
        CommandId::WorkspaceTabActivatePrev => CommandDescriptor::new(
            "workspace.tab.activate_prev",
            "Activate Previous Tab",
            "Activate the previous tab.",
        ),
        CommandId::WorkspaceTabActivateNext => CommandDescriptor::new(
            "workspace.tab.activate_next",
            "Activate Next Tab",
            "Activate the next tab.",
        ),
    }
}

fn default_shortcuts_for_id(id: CommandId) -> Vec<CommandShortcut> {
    match id {
        CommandId::WorkspaceFocusPanel(FocusTarget::FileTreePanel) => {
            vec![CommandShortcut::new(ShortcutScope::Global, meta_char('b')).with_priority(100)]
        }
        CommandId::WorkspaceFocusPanel(FocusTarget::GitPanel) => {
            vec![
                CommandShortcut::new(ShortcutScope::Global, meta_shift_char('g')).with_priority(80),
            ]
        }
        CommandId::WorkspaceFocusPanel(FocusTarget::OutlinePanel) => {
            vec![
                CommandShortcut::new(ShortcutScope::Global, meta_shift_char('o')).with_priority(80),
            ]
        }
        CommandId::WorkspaceFocusPanel(FocusTarget::ProjectSearch) => {
            vec![
                CommandShortcut::new(ShortcutScope::Global, meta_shift_char('f')).with_priority(80),
            ]
        }
        CommandId::WorkspaceFocusPanel(FocusTarget::Terminal) => {
            vec![CommandShortcut::new(ShortcutScope::Global, ctrl_char('`')).with_priority(80)]
        }
        CommandId::WorkspaceFocusPanel(FocusTarget::Notification) => {
            vec![
                CommandShortcut::new(ShortcutScope::Global, meta_shift_char('n')).with_priority(80),
            ]
        }
        CommandId::WorkspaceOpenProjectPicker => vec![
            CommandShortcut::new(ShortcutScope::Global, meta_shift_char('p')).with_priority(80),
        ],
        CommandId::WorkspaceOpenSettings => {
            vec![CommandShortcut::new(ShortcutScope::Global, meta_char(',')).with_priority(80)]
        }
        CommandId::WorkspaceOpenCodeActions => {
            vec![CommandShortcut::new(ShortcutScope::Global, meta_char('.')).with_priority(80)]
        }
        CommandId::WorkspaceStartDebugging => vec![
            CommandShortcut::new(ShortcutScope::Global, plain(KeyCode::F(5))).with_priority(80),
        ],
        CommandId::WorkspaceFileTreeSelectPrev => vec![
            CommandShortcut::new(
                ShortcutScope::Focus(FocusTarget::FileTreePanel),
                plain(KeyCode::Up),
            )
            .with_priority(110),
        ],
        CommandId::WorkspaceFileTreeSelectNext => vec![
            CommandShortcut::new(
                ShortcutScope::Focus(FocusTarget::FileTreePanel),
                plain(KeyCode::Down),
            )
            .with_priority(110),
        ],
        CommandId::WorkspaceFileTreeExpandOrDescend => vec![
            CommandShortcut::new(
                ShortcutScope::Focus(FocusTarget::FileTreePanel),
                plain(KeyCode::Right),
            )
            .with_priority(110),
        ],
        CommandId::WorkspaceFileTreeCollapseOrAscend => vec![
            CommandShortcut::new(
                ShortcutScope::Focus(FocusTarget::FileTreePanel),
                plain(KeyCode::Left),
            )
            .with_priority(110),
        ],
        CommandId::WorkspaceFileTreeActivateSelection => vec![
            CommandShortcut::new(
                ShortcutScope::Focus(FocusTarget::FileTreePanel),
                plain(KeyCode::Enter),
            )
            .with_priority(110),
        ],
        CommandId::WorkspaceTogglePanel(target) if target.is_visibility_managed_panel() => vec![
            CommandShortcut::new(
                ShortcutScope::Focus(target),
                Keystroke::new(
                    KeyCode::Char('w'),
                    Modifiers::new(false, false, false, true),
                ),
            )
            .with_priority(120),
        ],
        _ => Vec::new(),
    }
}

const fn focus_panel_id(target: FocusTarget) -> &'static str {
    match target {
        FocusTarget::Editor => "workspace.focus_panel.editor",
        FocusTarget::Palette => "workspace.focus_panel.palette",
        FocusTarget::FileTreePanel => "workspace.focus_panel.file_tree",
        FocusTarget::GitPanel => "workspace.focus_panel.git",
        FocusTarget::OutlinePanel => "workspace.focus_panel.outline",
        FocusTarget::ProjectSearch => "workspace.focus_panel.project_search",
        FocusTarget::LanguageServers => "workspace.focus_panel.language_servers",
        FocusTarget::Terminal => "workspace.focus_panel.terminal",
        FocusTarget::DebugPanel => "workspace.focus_panel.debug",
        FocusTarget::Notification => "workspace.focus_panel.notification",
    }
}

const fn focus_panel_title(target: FocusTarget) -> &'static str {
    match target {
        FocusTarget::Editor => "Focus Editor",
        FocusTarget::Palette => "Focus Command Palette",
        FocusTarget::FileTreePanel => "Focus File Tree Panel",
        FocusTarget::GitPanel => "Focus Git Panel",
        FocusTarget::OutlinePanel => "Focus Outline Panel",
        FocusTarget::ProjectSearch => "Focus Project Search Panel",
        FocusTarget::LanguageServers => "Focus Language Servers Panel",
        FocusTarget::Terminal => "Focus Terminal Panel",
        FocusTarget::DebugPanel => "Focus Debug Panel",
        FocusTarget::Notification => "Focus Notification Panel",
    }
}

const fn focus_panel_description(target: FocusTarget) -> &'static str {
    match target {
        FocusTarget::Editor => "Move focus to the editor pane.",
        FocusTarget::Palette => "Move focus to the command palette.",
        FocusTarget::FileTreePanel => "Show file tree panel and move focus to it.",
        FocusTarget::GitPanel => "Show Git panel and move focus to it.",
        FocusTarget::OutlinePanel => "Show outline panel and move focus to it.",
        FocusTarget::ProjectSearch => "Show project search panel and move focus to it.",
        FocusTarget::LanguageServers => "Show language servers panel and move focus to it.",
        FocusTarget::Terminal => "Show terminal panel and move focus to it.",
        FocusTarget::DebugPanel => "Show debug panel and move focus to it.",
        FocusTarget::Notification => "Show notification panel and move focus to it.",
    }
}

const fn toggle_panel_id(target: FocusTarget) -> &'static str {
    match target {
        FocusTarget::Editor => "workspace.toggle_panel.editor",
        FocusTarget::Palette => "workspace.toggle_panel.palette",
        FocusTarget::FileTreePanel => "workspace.toggle_panel.file_tree",
        FocusTarget::GitPanel => "workspace.toggle_panel.git",
        FocusTarget::OutlinePanel => "workspace.toggle_panel.outline",
        FocusTarget::ProjectSearch => "workspace.toggle_panel.project_search",
        FocusTarget::LanguageServers => "workspace.toggle_panel.language_servers",
        FocusTarget::Terminal => "workspace.toggle_panel.terminal",
        FocusTarget::DebugPanel => "workspace.toggle_panel.debug",
        FocusTarget::Notification => "workspace.toggle_panel.notification",
    }
}

const fn toggle_panel_title(target: FocusTarget) -> &'static str {
    match target {
        FocusTarget::Editor => "Toggle Editor",
        FocusTarget::Palette => "Toggle Command Palette",
        FocusTarget::FileTreePanel => "Toggle File Tree Panel",
        FocusTarget::GitPanel => "Toggle Git Panel",
        FocusTarget::OutlinePanel => "Toggle Outline Panel",
        FocusTarget::ProjectSearch => "Toggle Project Search Panel",
        FocusTarget::LanguageServers => "Toggle Language Servers Panel",
        FocusTarget::Terminal => "Toggle Terminal Panel",
        FocusTarget::DebugPanel => "Toggle Debug Panel",
        FocusTarget::Notification => "Toggle Notification Panel",
    }
}

const fn toggle_panel_description(target: FocusTarget) -> &'static str {
    match target {
        FocusTarget::Editor => "Toggle editor visibility.",
        FocusTarget::Palette => "Toggle command palette visibility.",
        FocusTarget::FileTreePanel => "Toggle file tree panel visibility.",
        FocusTarget::GitPanel => "Toggle Git panel visibility.",
        FocusTarget::OutlinePanel => "Toggle outline panel visibility.",
        FocusTarget::ProjectSearch => "Toggle project search panel visibility.",
        FocusTarget::LanguageServers => "Toggle language servers panel visibility.",
        FocusTarget::Terminal => "Toggle terminal panel visibility.",
        FocusTarget::DebugPanel => "Toggle debug panel visibility.",
        FocusTarget::Notification => "Toggle notification panel visibility.",
    }
}

fn plain(key: KeyCode) -> Keystroke {
    Keystroke::new(key, Modifiers::default())
}

fn meta_char(c: char) -> Keystroke {
    Keystroke::new(KeyCode::Char(c), Modifiers::new(false, false, false, true))
}

fn meta_shift_char(c: char) -> Keystroke {
    Keystroke::new(KeyCode::Char(c), Modifiers::new(false, false, true, true))
}

fn ctrl_char(c: char) -> Keystroke {
    Keystroke::new(KeyCode::Char(c), Modifiers::new(true, false, false, false))
}

#[cfg(test)]
mod tests {
    use crate::{Command, FocusTarget, command::WorkspaceCommand};

    use super::{CommandId, command_catalog, command_descriptor, default_shortcut_bindings};

    #[test]
    fn descriptors_provide_stable_ids_for_parameterized_commands() {
        let command = Command::from(WorkspaceCommand::FocusPanel(FocusTarget::FileTreePanel));
        let descriptor = command_descriptor(&command);

        assert_eq!(descriptor.id, "workspace.focus_panel.file_tree");
        assert_eq!(descriptor.title, "Focus File Tree Panel");
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
    fn command_catalog_contains_entries_without_default_shortcuts() {
        let catalog = command_catalog();
        let editor_insert_text = catalog
            .iter()
            .find(|entry| entry.id == CommandId::EditorInsertText)
            .expect("editor.insert_text should be cataloged");

        assert!(editor_insert_text.default_shortcuts.is_empty());
        assert_eq!(editor_insert_text.descriptor.id, "editor.insert_text");
    }
}
