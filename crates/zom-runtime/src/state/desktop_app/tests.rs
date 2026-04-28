//! `DesktopAppState` 行为与命令分发测试。

use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use zom_protocol::{
    CommandInvocation, EditorAction, EditorInvocation, FocusTarget, KeyCode, Keystroke, Modifiers,
    OverlayTarget, Position,
    command::{FileTreeAction, NotificationAction, TabAction, WorkspaceAction},
};
use zom_text_tokens::LineEnding;

use super::{
    DesktopAppState, DesktopNotificationEvent, DesktopNotificationKind, DesktopNotificationLevel,
    DesktopNotificationSource, DesktopUiAction,
};
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

    state.activate_file_tree_node("main.rs", FileTreeNodeKind::File);

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

    state.dispatch_command(CommandInvocation::from(FileTreeAction::ActivateSelection));

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

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
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

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

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

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

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

    state.dispatch_command(CommandInvocation::from(TabAction::ActivateNextTab));

    assert_eq!(state.pane.active_tab_index, Some(1));
    assert_eq!(state.tool_bar.cursor, Position::new(0, 1));
    assert_eq!(state.tool_bar.language, "Python");

    state.dispatch_command(CommandInvocation::from(TabAction::ActivatePrevTab));
    assert_eq!(state.pane.active_tab_index, Some(0));
    assert_eq!(state.tool_bar.cursor, Position::zero());
    assert_eq!(state.tool_bar.language, "Rust");

    state.dispatch_command(CommandInvocation::from(TabAction::ActivatePrevTab));
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
    state.activate_file_tree_node("a.rs", FileTreeNodeKind::File);
    state.activate_file_tree_node("b.py", FileTreeNodeKind::File);
    state.activate_file_tree_node("c.md", FileTreeNodeKind::File);

    assert_eq!(
        state.file_tree.selected_node().map(|(path, _)| path),
        Some("c.md".to_string())
    );

    state.dispatch_command(CommandInvocation::from(TabAction::ActivatePrevTab));
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

    state.dispatch_command(CommandInvocation::from(TabAction::ActivateNextTab));
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
    let handled = state.dispatch_keystroke(&next_tab);

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

    let handled = state.dispatch_keystroke(&keystroke);

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
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.tool_bar.cursor = Position::new(0, 1);

    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("X")));

    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "aXb"
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 2));

    state.dispatch_command(CommandInvocation::from(EditorAction::DeleteBackward));
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
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("ab\ncd"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.tool_bar.cursor = Position::new(1, 1);

    state.dispatch_command(CommandInvocation::from(EditorAction::SelectAll));

    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .selection,
        zom_protocol::Selection::new(Position::new(0, 0), Position::new(1, 2))
    );
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .doc_version,
        1
    );
    assert_eq!(state.tool_bar.cursor, Position::new(1, 2));
}

#[test]
fn plain_character_keystroke_in_editor_focus_inserts_text() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 1);

    let handled =
        state.dispatch_keystroke(&Keystroke::new(KeyCode::Char('x'), Modifiers::default()));

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
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 1);

    let select_left = Keystroke::new(KeyCode::Left, Modifiers::new(false, false, true, false));
    assert!(state.dispatch_keystroke(&select_left));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .selection,
        zom_protocol::Selection::new(Position::new(0, 1), Position::new(0, 0))
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 0));

    let backspace = Keystroke::new(KeyCode::Backspace, Modifiers::default());
    assert!(state.dispatch_keystroke(&backspace));
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
fn undo_and_redo_restore_last_text_edit_transaction() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 1);

    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("X")));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "aXb"
    );

    state.dispatch_command(CommandInvocation::from(EditorAction::Undo));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "ab"
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 0));

    state.dispatch_command(CommandInvocation::from(EditorAction::Redo));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "aXb"
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 2));
}

#[test]
fn continuous_typing_merges_into_single_undo_step() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text(""),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 0);

    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("a")));
    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("b")));
    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("c")));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "abc"
    );

    state.dispatch_command(CommandInvocation::from(EditorAction::Undo));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        ""
    );
    assert_eq!(state.tool_bar.cursor, Position::new(0, 0));
}

#[test]
fn moving_cursor_breaks_typing_merge_group() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text(""),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 0);

    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("a")));
    state.dispatch_command(CommandInvocation::from(EditorAction::MoveLeft));
    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("b")));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "ba"
    );

    state.dispatch_command(CommandInvocation::from(EditorAction::Undo));
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "a"
    );
}

#[test]
fn copy_command_emits_clipboard_write_ui_action() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("abcd"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 3);
    state.dispatch_command(CommandInvocation::from(EditorAction::SelectLeft));
    state.dispatch_command(CommandInvocation::from(EditorAction::SelectLeft));

    state.dispatch_command(CommandInvocation::from(EditorAction::Copy));

    assert_eq!(
        state.take_pending_ui_action(),
        Some(DesktopUiAction::WriteClipboard("bc".into()))
    );
}

#[test]
fn cut_command_writes_clipboard_and_deletes_selected_text() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("abcd"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 3);
    state.dispatch_command(CommandInvocation::from(EditorAction::SelectLeft));
    state.dispatch_command(CommandInvocation::from(EditorAction::SelectLeft));

    state.dispatch_command(CommandInvocation::from(EditorAction::Cut));

    assert_eq!(
        state.take_pending_ui_action(),
        Some(DesktopUiAction::WriteClipboard("bc".into()))
    );
    assert_eq!(
        state
            .active_editor_snapshot()
            .expect("active editor should exist")
            .text,
        "ad"
    );
}

#[test]
fn paste_command_emits_clipboard_read_ui_action() {
    let mut state = DesktopAppState::from_current_workspace();
    set_tabs(
        &mut state,
        vec![(
            runtime_test_tab_state("demo.rs", zom_protocol::BufferId::new(1), LineEnding::Lf),
            zom_editor::EditorState::from_text("ab"),
        )],
    );
    state.pane.active_tab_index = Some(0);
    state.focused_target = FocusTarget::Editor;
    state.tool_bar.cursor = Position::new(0, 1);

    state.dispatch_command(CommandInvocation::from(EditorAction::Paste));

    assert_eq!(
        state.take_pending_ui_action(),
        Some(DesktopUiAction::PasteFromClipboard)
    );
}

#[test]
fn save_active_buffer_writes_current_text_with_original_line_ending() {
    let workspace = create_temp_workspace("save-active-buffer");
    let file_path = workspace.join("main.rs");
    fs::write(&file_path, "a\r\nb\r\n").expect("write CRLF file");

    let mut state = DesktopAppState::from_current_workspace();
    state.switch_project(workspace.clone());
    state.activate_file_tree_node("main.rs", FileTreeNodeKind::File);
    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("x")));

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::SaveActiveBuffer));

    let content = fs::read_to_string(&file_path).expect("read saved file");
    assert_eq!(content, "xa\r\nb\r\n");

    remove_temp_workspace(workspace);
}

#[test]
fn editor_command_without_active_tab_is_noop() {
    let mut state = DesktopAppState::from_current_workspace();
    state.pane.tabs.clear();
    state.pane.active_tab_index = None;
    state.tool_bar.cursor = Position::new(3, 7);

    state.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text("x")));

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

    let handled_focus = state.dispatch_keystroke(&focus_git);

    assert!(handled_focus);
    assert!(state.is_panel_visible(FocusTarget::GitPanel));
    assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
    assert_eq!(state.focused_target, FocusTarget::GitPanel);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::GitPanel)
    );

    let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
    let handled_close = state.dispatch_keystroke(&close);

    assert!(handled_close);
    assert!(!state.is_panel_visible(FocusTarget::GitPanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
}

#[test]
fn focus_panel_replaces_existing_left_slot_panel() {
    let mut state = DesktopAppState::from_current_workspace();
    assert!(state.is_panel_visible(FocusTarget::FileTreePanel));

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::OutlinePanel,
    )));

    assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
    assert!(state.is_panel_visible(FocusTarget::OutlinePanel));
    assert_eq!(state.focused_target, FocusTarget::OutlinePanel);
}

#[test]
fn focus_panel_replaces_existing_bottom_dock_panel() {
    let mut state = DesktopAppState::from_current_workspace();
    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::TerminalPanel,
    )));
    assert!(state.is_panel_visible(FocusTarget::TerminalPanel));

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::DebugPanel,
    )));

    assert!(!state.is_panel_visible(FocusTarget::TerminalPanel));
    assert!(state.is_panel_visible(FocusTarget::DebugPanel));
    assert_eq!(state.focused_target, FocusTarget::DebugPanel);
}

#[test]
fn right_and_bottom_docks_can_stay_visible_together() {
    let mut state = DesktopAppState::from_current_workspace();
    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::NotificationPanel,
    )));
    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::TerminalPanel,
    )));

    assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
    assert!(state.is_panel_visible(FocusTarget::TerminalPanel));
}

#[test]
fn close_focused_bottom_panel_keeps_right_panel_visible() {
    let mut state = DesktopAppState::from_current_workspace();
    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::NotificationPanel,
    )));
    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::TerminalPanel,
    )));

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

    assert!(!state.is_panel_visible(FocusTarget::TerminalPanel));
    assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
}

#[test]
fn hide_visible_panel_in_dock_hides_target_and_falls_back_to_editor() {
    let mut state = DesktopAppState::from_current_workspace();
    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
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

    let handled_focus = state.dispatch_keystroke(&focus_notification);

    assert!(handled_focus);
    assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
    assert_eq!(state.focused_target, FocusTarget::NotificationPanel);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::NotificationPanel)
    );

    let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
    let handled_close = state.dispatch_keystroke(&close);

    assert!(handled_close);
    assert!(!state.is_panel_visible(FocusTarget::NotificationPanel));
    assert_eq!(state.focused_target, FocusTarget::Editor);
    assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
}

#[test]
fn keyboard_shortcut_can_request_open_project_picker_ui_action() {
    let mut state = DesktopAppState::from_current_workspace();
    let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::OpenProjectPicker));

    let handled = state.dispatch_keystroke(&keystroke);

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

    let handled = state.dispatch_keystroke(&keystroke);

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

    let handled = state.dispatch_keystroke(&keystroke);

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

    let handled = state.dispatch_keystroke(&keystroke);

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
fn push_notification_sets_active_toast_and_persists_history() {
    let mut state = DesktopAppState::from_current_workspace();

    let first_id = state.push_notification(DesktopNotificationLevel::Info, "first");
    let second_id = state.push_notification(DesktopNotificationLevel::Warning, "second");

    assert_eq!(first_id, 1);
    assert_eq!(second_id, 2);
    assert_eq!(state.notifications.len(), 2);
    assert_eq!(state.notifications[0].message, "first");
    assert_eq!(state.notifications[1].message, "second");
    assert_eq!(
        state.notifications[1].level,
        DesktopNotificationLevel::Warning
    );
    assert_eq!(
        state
            .active_toast_notification
            .as_ref()
            .map(|notification| notification.message.as_str()),
        Some("second")
    );
    assert_eq!(state.unread_notification_count, 2);
}

#[test]
fn info_event_without_user_initiated_does_not_trigger_toast() {
    let mut state = DesktopAppState::from_current_workspace();

    state.publish_notification_event(DesktopNotificationEvent::new(
        DesktopNotificationLevel::Info,
        DesktopNotificationSource::Workspace,
        "background refreshed",
    ));

    assert_eq!(state.notifications.len(), 1);
    assert!(state.active_toast_notification.is_none());
}

#[test]
fn info_event_with_user_initiated_triggers_toast() {
    let mut state = DesktopAppState::from_current_workspace();

    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Info,
            DesktopNotificationSource::Workspace,
            "opened project",
        )
        .is_user_initiated(),
    );

    assert_eq!(state.notifications.len(), 1);
    assert_eq!(
        state
            .active_toast_notification
            .as_ref()
            .map(|notification| notification.message.as_str()),
        Some("opened project")
    );
}

#[test]
fn dedupe_event_aggregates_count_and_avoids_second_toast() {
    let mut state = DesktopAppState::from_current_workspace();

    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Warning,
            DesktopNotificationSource::Workspace,
            "indexing slow",
        )
        .with_dedupe_key("workspace.indexing.slow"),
    );
    let first_toast_id = state
        .active_toast_notification
        .as_ref()
        .map(|notification| notification.id);
    state.clear_active_toast_notification();

    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Warning,
            DesktopNotificationSource::Workspace,
            "indexing slow",
        )
        .with_dedupe_key("workspace.indexing.slow"),
    );

    assert_eq!(state.notifications.len(), 1);
    assert_eq!(state.notifications[0].occurrence_count, 2);
    assert_eq!(state.active_toast_notification, None);
    assert_eq!(first_toast_id, Some(state.notifications[0].id));
}

#[test]
fn progress_event_updates_status_bar_only() {
    let mut state = DesktopAppState::from_current_workspace();
    let mut event = DesktopNotificationEvent::new(
        DesktopNotificationLevel::Info,
        DesktopNotificationSource::System,
        "indexing 12%",
    );
    event.kind = DesktopNotificationKind::Progress;

    state.publish_notification_event(event);

    assert!(state.notifications.is_empty());
    assert!(state.active_toast_notification.is_none());
    assert_eq!(
        state
            .active_status_notification
            .as_ref()
            .map(|notification| notification.message.as_str()),
        Some("indexing 12%")
    );
}

#[test]
fn notification_command_mark_all_read_clears_unread_counter() {
    let mut state = DesktopAppState::from_current_workspace();
    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Warning,
            DesktopNotificationSource::Workspace,
            "first warning",
        )
        .with_dedupe_key("warn.first"),
    );
    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Error,
            DesktopNotificationSource::Workspace,
            "second error",
        )
        .with_dedupe_key("error.second"),
    );

    state.dispatch_command(CommandInvocation::from(NotificationAction::MarkAllRead));

    assert_eq!(state.unread_notification_count, 0);
    assert!(
        state
            .notifications
            .iter()
            .all(|notification| notification.is_read)
    );
}

#[test]
fn notification_command_mark_selected_read_marks_only_one_item() {
    let mut state = DesktopAppState::from_current_workspace();
    let first_id = state
        .publish_notification_event(
            DesktopNotificationEvent::new(
                DesktopNotificationLevel::Warning,
                DesktopNotificationSource::Workspace,
                "first",
            )
            .with_dedupe_key("mark-selected.first"),
        )
        .expect("first id");
    let second_id = state
        .publish_notification_event(
            DesktopNotificationEvent::new(
                DesktopNotificationLevel::Warning,
                DesktopNotificationSource::Workspace,
                "second",
            )
            .with_dedupe_key("mark-selected.second"),
        )
        .expect("second id");
    state.selected_notification_id = Some(first_id);
    state.unread_notification_count = 2;

    state.dispatch_command(CommandInvocation::from(
        NotificationAction::MarkSelectedRead,
    ));

    assert_eq!(state.unread_notification_count, 1);
    let first = state
        .notifications
        .iter()
        .find(|item| item.id == first_id)
        .expect("first notification");
    let second = state
        .notifications
        .iter()
        .find(|item| item.id == second_id)
        .expect("second notification");
    assert!(first.is_read);
    assert!(!second.is_read);
}

#[test]
fn notification_command_clear_read_keeps_only_unread_items() {
    let mut state = DesktopAppState::from_current_workspace();
    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Warning,
            DesktopNotificationSource::Workspace,
            "keep me unread",
        )
        .with_dedupe_key("warn.unread"),
    );
    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Info,
            DesktopNotificationSource::Workspace,
            "will be read",
        )
        .with_dedupe_key("info.read"),
    );
    state.notifications[1].is_read = true;
    state.unread_notification_count = 1;

    state.dispatch_command(CommandInvocation::from(NotificationAction::ClearRead));

    assert_eq!(state.notifications.len(), 1);
    assert_eq!(state.notifications[0].message, "keep me unread");
    assert_eq!(state.unread_notification_count, 1);
}

#[test]
fn focusing_notification_panel_does_not_auto_mark_all_read() {
    let mut state = DesktopAppState::from_current_workspace();
    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Info,
            DesktopNotificationSource::Workspace,
            "keep unread on focus",
        )
        .with_dedupe_key("focus.keep-unread"),
    );
    assert_eq!(state.unread_notification_count, 1);

    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
        FocusTarget::NotificationPanel,
    )));

    assert_eq!(state.unread_notification_count, 1);
    assert!(
        state
            .notifications
            .iter()
            .any(|notification| !notification.is_read)
    );
}

#[test]
fn notification_command_clear_all_empties_history_and_status() {
    let mut state = DesktopAppState::from_current_workspace();
    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Error,
            DesktopNotificationSource::Workspace,
            "fatal issue",
        )
        .with_dedupe_key("error.fatal"),
    );
    assert!(!state.notifications.is_empty());
    assert!(state.active_status_notification.is_some());

    state.dispatch_command(CommandInvocation::from(NotificationAction::ClearAll));

    assert!(state.notifications.is_empty());
    assert!(state.active_status_notification.is_none());
    assert!(state.active_toast_notification.is_none());
    assert_eq!(state.unread_notification_count, 0);
}

#[test]
fn focus_unread_error_notification_sets_focus_and_selection_target() {
    let mut state = DesktopAppState::from_current_workspace();
    state.publish_notification_event(
        DesktopNotificationEvent::new(
            DesktopNotificationLevel::Info,
            DesktopNotificationSource::Workspace,
            "normal info",
        )
        .with_dedupe_key("info.1"),
    );
    let error_id = state
        .publish_notification_event(
            DesktopNotificationEvent::new(
                DesktopNotificationLevel::Error,
                DesktopNotificationSource::Workspace,
                "critical build error",
            )
            .with_dedupe_key("error.critical"),
        )
        .expect("error notification should be persisted");

    state.dispatch_command(CommandInvocation::from(
        NotificationAction::FocusUnreadError,
    ));

    assert_eq!(state.focused_target, FocusTarget::NotificationPanel);
    assert_eq!(
        state.take_pending_focus_target(),
        Some(FocusTarget::NotificationPanel)
    );
    assert_eq!(
        state.take_pending_notification_selection_id(),
        Some(error_id)
    );
}

#[test]
fn notification_selection_commands_move_between_rows() {
    let mut state = DesktopAppState::from_current_workspace();
    let newest_id = state
        .publish_notification_event(
            DesktopNotificationEvent::new(
                DesktopNotificationLevel::Info,
                DesktopNotificationSource::Workspace,
                "newest",
            )
            .with_dedupe_key("n.newest"),
        )
        .expect("newest id");
    let middle_id = state
        .publish_notification_event(
            DesktopNotificationEvent::new(
                DesktopNotificationLevel::Info,
                DesktopNotificationSource::Workspace,
                "middle",
            )
            .with_dedupe_key("n.middle"),
        )
        .expect("middle id");
    let oldest_id = state
        .publish_notification_event(
            DesktopNotificationEvent::new(
                DesktopNotificationLevel::Info,
                DesktopNotificationSource::Workspace,
                "oldest",
            )
            .with_dedupe_key("n.oldest"),
        )
        .expect("oldest id");

    // 面板显示顺序为 newest -> middle -> oldest，默认从首行开始。
    state.dispatch_command(CommandInvocation::from(NotificationAction::SelectNext));
    assert_eq!(
        state.take_pending_notification_selection_id(),
        Some(middle_id)
    );
    state.dispatch_command(CommandInvocation::from(NotificationAction::SelectNext));
    assert_eq!(
        state.take_pending_notification_selection_id(),
        Some(newest_id)
    );
    state.dispatch_command(CommandInvocation::from(NotificationAction::SelectPrev));
    assert_eq!(
        state.take_pending_notification_selection_id(),
        Some(middle_id)
    );
    state.dispatch_command(CommandInvocation::from(NotificationAction::SelectPrev));
    assert_eq!(
        state.take_pending_notification_selection_id(),
        Some(oldest_id)
    );
}

#[test]
fn close_focused_closes_active_settings_overlay_first() {
    let mut state = DesktopAppState::from_current_workspace();
    state.dispatch_command(CommandInvocation::from(WorkspaceAction::FocusOverlay(
        OverlayTarget::Settings,
    )));

    let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
    let handled = state.dispatch_keystroke(&close);

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
    state.activate_file_tree_node("src/main.rs", FileTreeNodeKind::File);

    let active_tab = state.pane.active_tab().expect("active tab should exist");
    assert_eq!(active_tab.relative_path, "src/main.rs");
    let active_editor = state
        .active_editor_snapshot()
        .expect("active editor should exist");
    assert_eq!(active_editor.text.split('\n').next(), Some("fn main() {}"));
    assert_eq!(active_tab.language(), "Rust");
    assert_eq!(active_tab.line_ending(), expected_platform_line_ending());
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
    state.activate_file_tree_node("old.rs", FileTreeNodeKind::File);

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
        runtime_test_tab_state(
            relative_path,
            zom_protocol::BufferId::new(buffer_id),
            LineEnding::Lf,
        ),
        zom_editor::EditorState::from_text("old"),
    )
}

fn runtime_test_tab_state(
    relative_path: &str,
    buffer_id: zom_protocol::BufferId,
    line_ending: LineEnding,
) -> crate::state::TabState {
    crate::state::TabState {
        buffer_id,
        title: "old".into(),
        relative_path: relative_path.into(),
        language: crate::workspace_paths::language_from_path(relative_path),
        line_ending,
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
    let line_ending = zom_text::detect_line_ending(&editor_state.text());
    (
        runtime_test_tab_state(relative_path, buffer_id, line_ending),
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

fn expected_platform_line_ending() -> LineEnding {
    if cfg!(windows) {
        LineEnding::Crlf
    } else {
        LineEnding::Lf
    }
}
