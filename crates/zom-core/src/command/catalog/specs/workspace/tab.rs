use crate::command::catalog::{CommandKey, CommandSpec};
use crate::{Command, command::TabCommand};

pub const SPECS: &[CommandSpec] = &[
    CommandSpec::new(
        CommandKey::WorkspaceTabCloseActive,
        "workspace.tab.close_active",
        "Close Active Tab",
        "Close the currently active tab.",
        || Some(Command::from(TabCommand::CloseActiveTab)),
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTabActivatePrev,
        "workspace.tab.activate_prev",
        "Activate Previous Tab",
        "Activate the previous tab.",
        || Some(Command::from(TabCommand::ActivatePrevTab)),
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTabActivateNext,
        "workspace.tab.activate_next",
        "Activate Next Tab",
        "Activate the next tab.",
        || Some(Command::from(TabCommand::ActivateNextTab)),
        &[],
    ),
];
