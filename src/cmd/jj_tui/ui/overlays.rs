mod bookmarks;
mod common;

use super::super::app::App;
use super::super::keybindings;
use super::super::state::{
    ConfirmState, ConflictsState, HelpState, MessageKind, ModeState, StatusMessage,
};
use super::super::theme;
use bookmarks::{render_bookmark_picker, render_bookmark_select, render_push_select};
use common::{centered_popup_area, key_hint, render_popup_shell, selectable_item};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use tui_popup::Popup;
use unicode_width::UnicodeWidthStr;

pub(super) fn render_overlays(frame: &mut Frame, app: &App) {
    if let ModeState::Help(ref help_state) = app.mode {
        render_help(frame, help_state);
    }

    if let ModeState::Confirming(ref state) = app.mode {
        render_confirmation(frame, state);
    }

    if let ModeState::BookmarkSelect(ref state) = app.mode {
        render_bookmark_select(frame, state);
    }

    if let ModeState::BookmarkPicker(ref state) = app.mode {
        render_bookmark_picker(frame, state);
    }

    if let ModeState::PushSelect(ref state) = app.mode {
        render_push_select(frame, state);
    }

    if let ModeState::Conflicts(ref state) = app.mode {
        render_conflicts_panel(frame, state);
    }

    if let Some(pending) = app.pending_key {
        render_prefix_key_popup(frame, keybindings::mode_id_from_state(&app.mode), pending);
    }

    if let Some(ref msg) = app.status_message
        && std::time::Instant::now() < msg.expires
    {
        render_toast(frame, msg);
    }
}

fn render_help(frame: &mut Frame, help_state: &HelpState) {
    let view = keybindings::build_help_view();
    let key_col_width = view
        .iter()
        .flat_map(|section| section.items.iter().map(|item| item.keys.width()))
        .max()
        .unwrap_or(9)
        .max(9);

    let mut help_text: Vec<Line> = Vec::new();
    for (section_idx, section) in view.iter().enumerate() {
        if section_idx > 0 {
            help_text.push(Line::from(""));
        }
        help_text.push(Line::from(Span::styled(
            section.title,
            Style::default().add_modifier(Modifier::BOLD),
        )));

        for item in &section.items {
            let pad_len = key_col_width.saturating_sub(item.keys.width());
            let pad = " ".repeat(pad_len);
            help_text.push(Line::from(format!(
                "  {}{}  {}",
                item.keys, pad, item.description
            )));
        }
    }

    let area = frame.area();
    let content_width = help_text
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.width())
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0) as u16;
    let content_height = help_text.len() as u16;
    let popup_width = (content_width + 4).min(area.width.saturating_sub(4));
    let popup_height = (content_height + 2).min(area.height.saturating_sub(4));
    let popup_area = centered_popup_area(area, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let help = Paragraph::new(help_text)
        .scroll((help_state.scroll_offset as u16, 0))
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(theme::POPUP_BG));

    frame.render_widget(help, popup_area);
}

fn render_confirmation(frame: &mut Frame, state: &ConfirmState) {
    let area = frame.area();
    let popup_width = 80u16.min(area.width.saturating_sub(4));
    let popup_height = (7 + state.revs.len().min(10)) as u16;
    let popup_area = centered_popup_area(area, popup_width, popup_height);
    let inner = render_popup_shell(
        frame,
        popup_area,
        " Confirm ",
        Color::Red,
        theme::POPUP_BG_DELETE,
    );

    let mut lines = vec![
        Line::from(Span::styled(
            state.message.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (idx, rev) in state.revs.iter().take(10).enumerate() {
        lines.push(Line::from(rev.to_string()));
        if idx == 9 && state.revs.len() > 10 {
            lines.push(Line::from(format!(
                "  ... and {} more",
                state.revs.len() - 10
            )));
        }
    }

    lines.push(Line::from(""));
    let yes_keys = key_hint(keybindings::ModeId::Confirm, "yes", true);
    let no_keys = key_hint(keybindings::ModeId::Confirm, "no", true);
    lines.push(Line::from(vec![
        Span::raw("  Press "),
        Span::styled(
            yes_keys,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to confirm or "),
        Span::styled(
            no_keys,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to cancel"),
    ]));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_conflicts_panel(frame: &mut Frame, state: &ConflictsState) {
    let area = frame.area();
    let list_height = state.files.len().clamp(1, 15);
    let popup_height = (5 + list_height) as u16;
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_area = centered_popup_area(area, popup_width, popup_height);
    let inner = render_popup_shell(
        frame,
        popup_area,
        " Conflicted Files ",
        Color::Red,
        theme::POPUP_BG,
    );

    let mut lines: Vec<Line> = Vec::new();
    if state.files.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No conflicts found",
            Style::default().fg(Color::Green),
        )));
    } else {
        for (idx, file) in state.files.iter().take(15).enumerate() {
            lines.push(selectable_item(idx == state.selected_index, file.clone()));
        }
        if state.files.len() > 15 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", state.files.len() - 15),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    let down_key = key_hint(keybindings::ModeId::Conflicts, "down", false);
    let up_key = key_hint(keybindings::ModeId::Conflicts, "up", false);
    let resolve_key = key_hint(keybindings::ModeId::Conflicts, "resolve", false);
    let exit_keys = key_hint(keybindings::ModeId::Conflicts, "exit", true);
    lines.push(Line::from(Span::styled(
        format!("{down_key}/{up_key}: navigate | {resolve_key}: resolve | {exit_keys}: exit"),
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_prefix_key_popup(frame: &mut Frame, mode: keybindings::ModeId, pending_key: char) {
    const KEY_SPACING: usize = 3;
    const TITLE_WRAPPER: usize = 4;
    const HORIZONTAL_PADDING: usize = 4;
    const FIXED_HEIGHT_LINES: usize = 4;
    const EDGE_MARGIN: u16 = 2;

    let Some(menu) = keybindings::prefix_menu(mode, pending_key) else {
        return;
    };

    let max_label_width = menu
        .items
        .iter()
        .map(|(_, label)| label.width())
        .max()
        .unwrap_or(0);
    let content_width = max_label_width + KEY_SPACING;
    let title_width = menu.title.width() + TITLE_WRAPPER;
    let popup_width = (content_width.max(title_width) + HORIZONTAL_PADDING) as u16;
    let popup_height = (menu.items.len() + FIXED_HEIGHT_LINES) as u16;

    let area = frame.area();
    let popup_area = Rect {
        x: area.width.saturating_sub(popup_width + EDGE_MARGIN),
        y: area.height.saturating_sub(popup_height + EDGE_MARGIN),
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(theme::PREFIX_POPUP_BG));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = vec![Line::from(Span::styled(
        format!("{} ({})", pending_key, menu.title),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(Span::styled(
        "─".repeat(inner.width as usize),
        Style::default().fg(Color::DarkGray),
    )));

    for (key, label) in menu.items {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{key}  "),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(label, Style::default().fg(Color::White)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_toast(frame: &mut Frame, msg: &StatusMessage) {
    let color = match msg.kind {
        MessageKind::Success => Color::Green,
        MessageKind::Warning => Color::Yellow,
        MessageKind::Error => Color::Red,
    };

    let popup = Popup::new(msg.text.clone()).style(Style::default().fg(color).bg(theme::TOAST_BG));
    frame.render_widget(popup, frame.area());
}
