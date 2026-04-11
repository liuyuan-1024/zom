#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Editor(EditorCommand),
    Workspace(WorkspaceCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorCommand {
    InsertText(String),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveToStart,
    MoveToEnd,
    MovePageUp,
    MovePageDown,
    DeleteBackward,
    DeleteForward,
    DeleteWordBackward,
    DeleteWordForward,
    InsertNewline,
    Undo,
    Redo,
    SelectAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    OpenCommandPalette,
    OpenFileFinder,
    ToggleSidebar,
    FocusEditor,
    FocusSidebar,
    FocusPalette,
    CloseActiveItem,
}
