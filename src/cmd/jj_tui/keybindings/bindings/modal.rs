use super::super::ModeId::{Confirm, Diff, Help, Selecting};
use super::super::{BindingSpec, CommandSpec, KeyDef};
use super::{chord, fixed, pending_prefix, single};
use crate::cmd::jj_tui::action::Action;
use ratatui::crossterm::event::KeyCode;

pub(super) fn commands() -> Vec<CommandSpec> {
    vec![
        BindingSpec::new(
            Help,
            "close",
            fixed(Action::ExitHelp),
            vec![
                single(KeyDef::Char('q')),
                single(KeyDef::Char('?')),
                single(KeyDef::Key(KeyCode::Esc)),
            ],
        ),
        BindingSpec::new(
            Help,
            "scroll_down",
            fixed(Action::ScrollHelpDown(1)),
            vec![
                single(KeyDef::Char('j')),
                single(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Help,
            "scroll_up",
            fixed(Action::ScrollHelpUp(1)),
            vec![single(KeyDef::Char('k')), single(KeyDef::Key(KeyCode::Up))],
        ),
        BindingSpec::new(
            Help,
            "page_down",
            fixed(Action::ScrollHelpDown(20)),
            vec![single(KeyDef::Char('d'))],
        ),
        BindingSpec::new(
            Help,
            "page_up",
            fixed(Action::ScrollHelpUp(20)),
            vec![single(KeyDef::Char('u'))],
        ),
        BindingSpec::new(
            Diff,
            "scroll_down",
            fixed(Action::ScrollDiffDown(1)),
            vec![
                single(KeyDef::Char('j')),
                single(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Diff,
            "scroll_up",
            fixed(Action::ScrollDiffUp(1)),
            vec![single(KeyDef::Char('k')), single(KeyDef::Key(KeyCode::Up))],
        ),
        BindingSpec::new(
            Diff,
            "page_down",
            fixed(Action::ScrollDiffDown(20)),
            vec![single(KeyDef::Char('d'))],
        ),
        BindingSpec::new(
            Diff,
            "page_up",
            fixed(Action::ScrollDiffUp(20)),
            vec![single(KeyDef::Char('u'))],
        ),
        BindingSpec::new(
            Diff,
            "nav",
            pending_prefix("nav"),
            vec![single(KeyDef::Char('z'))],
        ),
        BindingSpec::new(
            Diff,
            "close",
            fixed(Action::ExitDiffView),
            vec![single(KeyDef::Char('q')), single(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            Diff,
            "top",
            fixed(Action::ScrollDiffTop),
            vec![chord('z', KeyDef::Char('t'))],
        )
        .prefix_title("nav"),
        BindingSpec::new(
            Diff,
            "bottom",
            fixed(Action::ScrollDiffBottom),
            vec![chord('z', KeyDef::Char('b'))],
        )
        .prefix_title("nav"),
        BindingSpec::new(
            Confirm,
            "yes",
            fixed(Action::ConfirmYes),
            vec![
                single(KeyDef::Char('y')),
                single(KeyDef::Key(KeyCode::Enter)),
            ],
        ),
        BindingSpec::new(
            Confirm,
            "no",
            fixed(Action::ConfirmNo),
            vec![single(KeyDef::Char('n')), single(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            Selecting,
            "down",
            fixed(Action::MoveCursorDown),
            vec![
                single(KeyDef::Char('j')),
                single(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Selecting,
            "up",
            fixed(Action::MoveCursorUp),
            vec![single(KeyDef::Char('k')), single(KeyDef::Key(KeyCode::Up))],
        ),
        BindingSpec::new(
            Selecting,
            "exit",
            fixed(Action::ExitSelecting),
            vec![single(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            Selecting,
            "abandon",
            fixed(Action::EnterConfirmAbandon),
            vec![single(KeyDef::Char('a'))],
        ),
    ]
}
