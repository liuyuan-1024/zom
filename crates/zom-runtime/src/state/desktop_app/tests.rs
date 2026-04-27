use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use zom_protocol::{
    CommandInvocation, EditorAction, EditorInvocation, FocusTarget, KeyCode, Keystroke, Modifiers,
    OverlayTarget, Position,
    command::{FileTreeAction, TabAction, WorkspaceAction},
};

use super::{DesktopAppState, DesktopUiAction};
use crate::state::{FileTreeNodeKind, PanelDock};

fn shortcut_for(command: CommandInvocation) -> Keystroke {
    zom_input::default_shortcut_registry()
        .bindings()
        .iter()
        .find(|binding| binding.command == command)
        .map(|binding| binding.keystroke)
        .unwrap_or_else(|| panic!("default shortcut should exist for command: {command:?}"))
}

#[test]
fn activating_file_tree_file_opens_tab_and_activates_it() {
    let workspace = create_temp_workspace("activate-file-opens-tab");
    fs::write(workspace.join("main.rs"), "fn main() {}").expect("write main.rs");

    let mut state = DesktopAppState::from_current_workspace();
    state.switch_project(workspace.clone());
    let before_len = state.pane.tabs.len();

    state.handle_file_tree_node_activate("main.rs", FileTreeNodeKind::File);

    assert_eq!(state.pane.tabs.len(), before_len + 1);
    let active_tab = state.pane.active_tab().expect("active tab should exist");
    assert_eq!(active_tab.relative_path, "main.rs");
    assert!(
        !state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text
            .is_empty()
    );
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));

    remove_temp_workspace(workspace);
}

#[test]
fn keyboard_select_and_activate_opens_file_in_pane() {
    let workspace = create_temp_workspace("keyboard-open");
    fs::write(workspace.join("main.rs"), "fn main() {}").expect("write main.rs");

    let mut state = DesktopAppState::from_current_workspace();
    state.switch_project(workspace.clone());

    state.file_tree.select_next_visible();
    state.file_tree.select_next_visible();
    state.pane.tabs.clear();
    state.pane.active_tab_index = None;

    state.handle_command(CommandInvocation::from(FileTreeAction::ActivateSelection));

    assert_eq!(state.pane.tabs.len(), 1);
    let active_tab = state.pane.active_tab().expect("active tab should exist");
    assert_eq!(active_tab.relative_path, "main.rs");

    remove_temp_workspace(workspace);
}

#[test]
fn focus_panel_shows_file_tree_and_requests_focus() {
    let mut state = DesktopAppState::from_current_workspace();
    state.visible_panels.remove(&FocusTarget::FileTreePanel);
    state.file_tree.roots[0].is_selected = false;

    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::FileTreePanel,
    )));

    assert!(state.is_panel_visible(FocusTarget::FileTreePanel));
    assert_eq!(state.focused_target, FocusTarget::FileTreePanel);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::FileTreePanel)
    );
    assert_eq!(
        state.file_tree.selected_node().map(|(path, _)| path),
        Some("".to_string())
    );
}

#[test]
fn close_focused_hides_focused_file_tree_and_falls_back_to_editor() {
    let mut state = DesktopAppState::from_current_workspace();
    state.focused_target = FocusTarget::FileTreePanel;
    state.visible_panels.insert(FocusTarget::FileTreePanel);

    state.handle_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

    assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
}

#[test]
fn close_focused_closes_active_tab_when_editor_is_focused() {
    let mut state = DesktopAppState::from_current_workspace();
    state.focused_target = FocusTarget::Editor;
    set_tabs(
        &mut state,
        vec![
            zom_runtime_test_tab("a.rs", 1),
            zom_runtime_test_tab("b.rs", 2),
        ],
    );
    state.pane.active_tab_index = Some(1);

    state.handle_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

    assert_eq!(state.pane.tabs.len(), 1);
    assert_eq!(state.pane.tabs[0].relative_path, "a.rs");
    assert_eq!(state.pane.active_tab_index, Some(0));
}

#[test]
fn tab_activation_commands_cycle_tabs_and_sync_toolbar_state() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![
            zom_runtime_test_tab_with_text_and_cursor("a.rs", "first\nline", 0),
            zom_runtime_test_tab_with_text_and_cursor("b.py", "x\r\ny", 1),
            zom_runtime_test_tab_with_text_and_cursor("c.md", "tail", 2),
        ],
    );
    state.pane.active_tab_index = Some(0);
    state.tool_bar.cursor = Position::new(99, 99);
    state.tool_bar.language = "Plain Text".into();

    state.handle_command(CommandInvocation::from(TabAction::ActivateNextTab));

    assert_eq!(state.pane.active_tab_index, Some(1));
    assert_eq!(state.tool_bar.cursor, Position::new(0, 1));
    assert_eq!(state.tool_bar.language, "Python");

    state.handle_command(CommandInvocation::from(TabAction::ActivatePrevTab));
    assert_eq!(state.pane.active_tab_index, Some(0));
    assert_eq!(state.tool_bar.cursor, Position::zero());
    assert_eq!(state.tool_bar.language, "Rust");

    state.handle_command(CommandInvocation::from(TabAction::ActivatePrevTab));
    assert_eq!(state.pane.active_tab_index, Some(2));
    assert_eq!(state.tool_bar.cursor, Position::new(0, 2));
    assert_eq!(state.tool_bar.language, "Markdown");
}

#[test]
fn tab_activation_commands_sync_file_tree_selection_to_active_tab() {
    let workspace = create_temp_workspace("tab-activation-syncs-file-tree");
    fs::write(workspace.join("a.rs"), "fn a() {}").expect("write a.rs");
    fs::write(workspace.join("b.py"), "print('b')").expect("write b.py");
    fs::write(workspace.join("c.md"), "# c").expect("write c.md");

    let mut state = DesktopAppState::from_current_workspace();
    state.switch_project(workspace.clone());
    state.handle_file_tree_node_activate("a.rs", FileTreeNodeKind::File);
    state.handle_file_tree_node_activate("b.py", FileTreeNodeKind::File);
    state.handle_file_tree_node_activate("c.md", FileTreeNodeKind::File);

    assert_eq!(
        state.file_tree.selected_node().map(|(path, _)| path),
        Some("c.md".to_string())
    );

    state.handle_command(CommandInvocation::from(TabAction::ActivatePrevTab));
    assert_eq!(
        state
            .pane
            .active_tab()
            .map(|tab| tab.relative_path.as_str()),
        Some("b.py")
    );
    assert_eq!(
        state.file_tree.selected_node().map(|(path, _)| path),
        Some("b.py".to_string())
    );

    state.handle_command(CommandInvocation::from(TabAction::ActivateNextTab));
    assert_eq!(
        state
            .pane
            .active_tab()
            .map(|tab| tab.relative_path.as_str()),
        Some("c.md")
    );
    assert_eq!(
        state.file_tree.selected_node().map(|(path, _)| path),
        Some("c.md".to_string())
    );

    remove_temp_workspace(workspace);
}

#[test]
fn keyboard_shortcut_can_activate_next_tab() {
    let mut state = DesktopAppState::from_current_workspace();
    state.focused_target = FocusTarget::FileTreePanel;
    set_tabs(
        &mut state,
        vec![
            zom_runtime_test_tab_with_text_and_cursor("a.rs", "a", 0),
            zom_runtime_test_tab_with_text_and_cursor("b.ts", "bc", 1),
        ],
    );
    state.pane.active_tab_index = Some(0);

    let next_tab = shortcut_for(CommandInvocation::from(TabAction::ActivateNextTab));
    let handled = state.handle_keystroke(&next_tab);

    assert!(handled);
    assert_eq!(state.pane.active_tab_index, Some(1));
    assert_eq!(state.tool_bar.cursor, Position::new(0, 1));
    assert_eq!(state.tool_bar.language, "TypeScript");
}

#[test]
fn keyboard_shortcut_resolves_via_input_layer_and_dispatches_workspace_command() {
    let mut state = DesktopAppState::from_current_workspace();
    let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::FileTreePanel,
    )));

    let handled = state.handle_keystroke(&keystroke);

    assert!(handled);
    assert!(state.is_panel_visible(FocusTarget::FileTreePanel));
    assert_eq!(state.focused_target, FocusTarget::FileTreePanel);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::FileTreePanel)
    );
}

#[test]
fn editor_command_updates_active_tab_buffer_and_cursor() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), "LF"),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.tool_bar.cursor = Position::new(0, 1);

    state.handle_command(CommandInvocation::from(EditorInvocation::insert_text("X")));

    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "aXb"
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 2));

    state.handle_command(CommandInvocation::from(EditorAction::DeleteBackward));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "ab"
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 1));
}

#[test]
fn editor_select_all_updates_active_selection_and_toolbar_cursor() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), "LF"),
            zom_editor::EditorState::from_text("ab\ncd"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.tool_bar.cursor = Position::new(1, 1);

    state.handle_command(CommandInvocation::from(EditorAction::SelectAll));

    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .selection,
        zom_protocol::Selection::new(Position::new(0, 0), Position::new(1, 2))
    );
    assert_eq!(state.tool_bar.cursor, Position::new(1, 2));
}

#[test]
fn plain_character_keystroke_in_editor_focus_inserts_text() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), "LF"),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 1);

    let handled = state.handle_keystroke(&Keystroke::new(KeyCode::Char('x'), Modifiers::default()));

    assert!(handled);
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "axb"
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 2));
}

#[test]
fn shift_left_then_backspace_deletes_selected_text_in_editor() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), "LF"),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 1);

    let select_left = Keystroke::new(KeyCode::Left, Modifiers::new(false, false, true, false));
    assert!(state.handle_keystroke(&select_left));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .selection,
        zom_protocol::Selection::new(Position::new(0, 1), Position::new(0, 0))
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 0));

    let backspace = Keystroke::new(KeyCode::Backspace, Modifiers::default());
    assert!(state.handle_keystroke(&backspace));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "b"
    );
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .selection,
        zom_protocol::Selection::caret(Position::new(0, 0))
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 0));
}

#[test]
fn editor_command_without_active_tab_is_noop() {
    let mut state = DesktopAppState::from_current_workspace();
    state.pane.tabs.clear();
    state.pane.active_tab_index = None;
    state.tool_bar.cursor = Position::new(3, 7);

    state.handle_command(CommandInvocation::from(EditorInvocation::insert_text("x")));

    assert_eq!(state.pane.tabs.len(), 0);
    assert_eq!(state.pane.active_tab_index, None);
    assert_eq!(state.tool_bar.cursor, Position::new(3, 7));
}

#[test]
fn keyboard_shortcut_can_focus_and_close_git_panel() {
    let mut state = DesktopAppState::from_current_workspace();
    let focus_git = shortcut_for(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::GitPanel,
    )));

    let handled_focus = state.handle_keystroke(&focus_git);

    assert!(handled_focus);
    assert!(state.is_panel_visible(FocusTarget::GitPanel));
    assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
    assert_eq!(state.focused_target, FocusTarget::GitPanel);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::GitPanel)
    );

    let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
    let handled_close = state.handle_keystroke(&close);

    assert!(handled_close);
    assert!(!state.is_panel_visible(FocusTarget::GitPanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
}

#[test]
fn focus_panel_replaces_existing_left_slot_panel() {
    let mut state = DesktopAppState::from_current_workspace();
    assert!(state.is_panel_visible(FocusTarget::FileTreePanel));

    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::OutlinePanel,
    )));

    assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
    assert!(state.is_panel_visible(FocusTarget::OutlinePanel));
    assert_eq!(state.focused_target, FocusTarget::OutlinePanel);
}

#[test]
fn focus_panel_replaces_existing_bottom_dock_panel() {
    let mut state = DesktopAppState::from_current_workspace();
    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::TerminalPanel,
    )));
    assert!(state.is_panel_visible(FocusTarget::TerminalPanel));

    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::DebugPanel,
    )));

    assert!(!state.is_panel_visible(FocusTarget::TerminalPanel));
    assert!(state.is_panel_visible(FocusTarget::DebugPanel));
    assert_eq!(state.focused_target, FocusTarget::DebugPanel);
}

#[test]
fn right_and_bottom_docks_can_stay_visible_together() {
    let mut state = DesktopAppState::from_current_workspace();
    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::NotificationPanel,
    )));
    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::TerminalPanel,
    )));

    assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
    assert!(state.is_panel_visible(FocusTarget::TerminalPanel));
}

#[test]
fn close_focused_bottom_panel_keeps_right_panel_visible() {
    let mut state = DesktopAppState::from_current_workspace();
    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::NotificationPanel,
    )));
    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::TerminalPanel,
    )));

    state.handle_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

    assert!(!state.is_panel_visible(FocusTarget::TerminalPanel));
    assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
}

#[test]
fn hide_visible_panel_in_dock_hides_target_and_falls_back_to_editor() {
    let mut state = DesktopAppState::from_current_workspace();
    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::GitPanel,
    )));
    assert_eq!(state.focused_target, FocusTarget::GitPanel);

    let hidden = state.hide_visible_panel_in_dock(PanelDock::Left);

    assert!(hidden);
    assert!(!state.is_panel_visible(FocusTarget::GitPanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
}

#[test]
fn keyboard_shortcut_can_focus_and_close_notification_panel() {
    let mut state = DesktopAppState::from_current_workspace();
    let focus_notification = shortcut_for(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::NotificationPanel,
    )));

    let handled_focus = state.handle_keystroke(&focus_notification);

    assert!(handled_focus);
    assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
    assert_eq!(state.focused_target, FocusTarget::NotificationPanel);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::NotificationPanel)
    );

    let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
    let handled_close = state.handle_keystroke(&close);

    assert!(handled_close);
    assert!(!state.is_panel_visible(FocusTarget::NotificationPanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
}

#[test]
fn keyboard_shortcut_can_request_open_project_picker_ui_action() {
    let mut state = DesktopAppState::from_current_workspace();
    let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::OpenProjectPicker));

    let handled = state.handle_keystroke(&keystroke);

    assert!(handled);
    assert_eq!(
        state.take_pending_ui_action(),
        Some(DesktopUiAction::OpenProjectPicker)
    );
}

#[test]
fn keyboard_shortcut_can_request_minimize_window_ui_action() {
    let mut state = DesktopAppState::from_current_workspace();
    let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::MinimizeWindow));

    let handled = state.handle_keystroke(&keystroke);

    assert!(handled);
    assert_eq!(
        state.take_pending_ui_action(),
        Some(DesktopUiAction::MinimizeWindow)
    );
}

#[test]
fn keyboard_shortcut_can_request_quit_app_ui_action() {
    let mut state = DesktopAppState::from_current_workspace();
    let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::QuitApp));

    let handled = state.handle_keystroke(&keystroke);

    assert!(handled);
    assert_eq!(
        state.take_pending_ui_action(),
        Some(DesktopUiAction::QuitApp)
    );
}

#[test]
fn keyboard_shortcut_can_focus_settings_overlay() {
    let mut state = DesktopAppState::from_current_workspace();
    let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::FocusOverlay(
        OverlayTarget::Settings,
    )));

    let handled = state.handle_keystroke(&keystroke);

    assert!(handled);
    assert_eq!(state.active_overlay, Some(OverlayTarget::Settings));
    assert_eq!(state.focused_target, FocusTarget::SettingsOverlay);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::SettingsOverlay)
    );
    assert_eq!(state.take_pending_ui_action(), None);
}

#[test]
fn close_focused_closes_active_settings_overlay_first() {
    let mut state = DesktopAppState::from_current_workspace();
    state.handle_command(CommandInvocation::from(WorkspaceAction::FocusOverlay(
        OverlayTarget::Settings,
    )));

    let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
    let handled = state.handle_keystroke(&close);

    assert!(handled);
    assert_eq!(state.active_overlay, None);
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
}

#[test]
fn switch_project_reloads_real_file_tree_and_clears_tabs() {
    let workspace = create_temp_workspace("switch-project-tree");
    fs::create_dir_all(workspace.join("src")).expect("create src directory");
    fs::write(workspace.join("src/lib.rs"), "pub fn answer() -> u8 { 42 }").expect("write lib.rs");

    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(&mut state, vec![zom_runtime_test_tab("old.rs", 999)]);
    state.pane.active_tab_index = Some(0);

    state.switch_project(workspace.clone());

    assert_eq!(
        state.project_root,
        crate::workspace_paths::normalize_workspace_root(workspace.clone())
    );
    assert_eq!(state.pane.tabs.len(), 0);
    assert!(state.pane.active_tab_index.is_none());
    assert!(state.active_editor_snapshot().is_none());
    assert_eq!(
        state.file_tree.roots[0]
            .children
            .iter()
            .map(|node| node.path.as_str())
            .collect::<Vec<_>>(),
        vec!["src"]
    );
    assert_eq!(state.tool_bar.cursor, Position::zero());
    assert_eq!(state.tool_bar.language, "");

    remove_temp_workspace(workspace);
}

#[test]
fn open_file_reads_from_selected_project_root() {
    let workspace = create_temp_workspace("open-file-from-root");
    fs::create_dir_all(workspace.join("src")).expect("create src directory");
    fs::write(workspace.join("src/main.rs"), "fn main() {}").expect("write main.rs");

    let mut state = DesktopAppState::from_current_workspace();
    state.switch_project(workspace.clone());
    state.handle_file_tree_node_activate("src/main.rs", FileTreeNodeKind::File);

    let active_tab = state.pane.active_tab().expect("active tab should exist");
    assert_eq!(active_tab.relative_path, "src/main.rs");
    let active_editor = state
        .active_editor_snapshot()
        .expect("active editor should exist");
    assert_eq!(active_editor.text.split('\n').next(), Some("fn main() {}"));
    assert_eq!(active_tab.language(), "Rust");
    assert_eq!(active_tab.line_ending(), &expected_platform_line_ending());
    assert_eq!(state.tool_bar.cursor, Position::zero());
    assert_eq!(state.tool_bar.language, "Rust");

    remove_temp_workspace(workspace);
}

#[test]
fn open_file_failure_keeps_existing_editor_state() {
    let workspace = create_temp_workspace("open-missing-file");
    fs::write(workspace.join("old.rs"), "let old = 1;").expect("write old.rs");

    let mut state = DesktopAppState::from_current_workspace();
    state.switch_project(workspace.clone());
    state.handle_file_tree_node_activate("old.rs", FileTreeNodeKind::File);

    let before_tabs = state.pane.tabs.clone();
    let before_active = state.pane.active_tab_index;
    let before_cursor = state.tool_bar.cursor;
    let before_language = state.tool_bar.language.clone();

    let opened = state.open_file_in_pane("missing.rs");

    assert!(!opened);
    assert_eq!(state.pane.tabs, before_tabs);
    assert_eq!(state.pane.active_tab_index, before_active);
    assert_eq!(state.tool_bar.cursor, before_cursor);
    assert_eq!(state.tool_bar.language, before_language);

    remove_temp_workspace(workspace);
}

fn create_temp_workspace(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time should be after unix epoch")
        .as_nanos();
    let workspace = std::env::temp_dir().join(format!("zom-desktop-state-{name}-{timestamp}"));
    fs::create_dir_all(&workspace).expect("create temp workspace");
    workspace
}

fn remove_temp_workspace(path: PathBuf) {
    fs::remove_dir_all(path).expect("remove temp workspace");
}

fn zom_runtime_test_tab(
    relative_path: &str,
    buffer_id: u64,
) -> (crate::state::TabState, zom_editor::EditorState) {
    (
        runtime_test_tab_state(relative_path, zom_protocol::BufferId::new(buffer_id), "LF"),
        zom_editor::EditorState::from_text("old"),
    )
}

fn runtime_test_tab_state(
    relative_path: &str,
    buffer_id: zom_protocol::BufferId,
    line_ending: &str,
) -> crate::state::TabState {
    crate::state::TabState {
        buffer_id,
        title: "old".into(),
        relative_path: relative_path.into(),
        language: crate::workspace_paths::language_from_path(relative_path),
        line_ending: line_ending.into(),
    }
}

fn zom_runtime_test_tab_with_text_and_cursor(
    relative_path: &str,
    text: &str,
    cursor_column: u32,
) -> (crate::state::TabState, zom_editor::EditorState) {
    let mut editor_state = zom_editor::EditorState::from_text(text);
    let mut cursor = Position::zero();
    for _ in 0..cursor_column {
        let result = zom_editor::apply_editor_invocation(
            &editor_state,
            cursor,
            &EditorInvocation::from(EditorAction::MoveRight),
        );
        editor_state = result.state;
        cursor = result.cursor;
    }
    let buffer_id = zom_protocol::BufferId::new((1000 + cursor_column).into());
    let line_ending = zom_text::detect_line_ending(editor_state.text());
    (
        runtime_test_tab_state(relative_path, buffer_id, &line_ending),
        editor_state,
    )
}

fn set_tabs(
    state: &mut DesktopAppState,
    tabs_with_states: Vec<(crate::state::TabState, zom_editor::EditorState)>,
) {
    state.pane.tabs = tabs_with_states
        .iter()
        .map(|(tab, _)| tab.clone())
        .collect();
    state.clear_editor_states();
    for (tab, editor_state) in tabs_with_states {
        state.replace_editor_state(tab.buffer_id, editor_state);
    }
}

fn expected_platform_line_ending() -> String {
    if cfg!(windows) {
        "CRLF".into()
    } else {
        "LF".into()
    }
}
