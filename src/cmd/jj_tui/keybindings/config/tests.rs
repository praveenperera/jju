use super::*;
use crate::cmd::jj_tui::keybindings::KeyDef;
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_parse_single_key_override() {
    let overrides = parse_overrides(
        r#"
version = 2

[[binding]]
mode = "normal"
command = "down"
keys = [["j"], ["Down"]]
"#,
    )
    .expect("parse overrides");

    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].mode, ModeId::Normal);
    assert_eq!(overrides[0].command, "down");
    assert_eq!(
        overrides[0].keys,
        vec![
            KeySequence::Single(KeyDef::Char('j')),
            KeySequence::Single(KeyDef::Key(KeyCode::Down))
        ]
    );
}

#[test]
fn test_parse_chord_override() {
    let overrides = parse_overrides(
        r#"
version = 2

[[binding]]
mode = "normal"
command = "fetch"
keys = [["g", "f"]]
"#,
    )
    .expect("parse overrides");

    assert_eq!(
        overrides[0].keys,
        vec![KeySequence::Chord('g', KeyDef::Char('f'))]
    );
}

#[test]
fn test_parse_rejects_long_sequences() {
    let error = parse_overrides(
        r#"
version = 2

[[binding]]
mode = "normal"
command = "fetch"
keys = [["g", "f", "x"]]
"#,
    )
    .expect_err("expected invalid sequence");

    assert!(error.to_string().contains("one or two steps"));
}
