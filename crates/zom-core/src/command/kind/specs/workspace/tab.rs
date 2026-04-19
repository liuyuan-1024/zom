use crate::command::kind::{Buildability, CommandKind, CommandKindSpec};
use crate::{CommandInvocation, TabAction};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceTabCloseActive,
        "workspace.tab.close_active",
        "Close Active Tab",
        "Close the currently active tab.",
        Buildability::Static(|| CommandInvocation::from(TabAction::CloseActiveTab)),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceTabActivatePrev,
        "workspace.tab.activate_prev",
        "Activate Previous Tab",
        "Activate the previous tab.",
        Buildability::Static(|| CommandInvocation::from(TabAction::ActivatePrevTab)),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceTabActivateNext,
        "workspace.tab.activate_next",
        "Activate Next Tab",
        "Activate the next tab.",
        Buildability::Static(|| CommandInvocation::from(TabAction::ActivateNextTab)),
        &[],
    ),
];
