use crate::cmd::jj_tui::vm::{RowDetails, TreeRowVm};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub(super) fn render_commit_details_from_vm(
    vm: &TreeRowVm,
    details: &RowDetails,
    label_color: Color,
) -> Vec<Line<'static>> {
    let indent = "  ".repeat(vm.visual_depth + 1);
    let dim = Style::default().fg(Color::Reset);
    let label_style = Style::default().fg(label_color);
    let change_id = format!("{}{}", vm.change_id_prefix, vm.change_id_suffix);
    let stats_str = match details.diff_stats.as_ref() {
        Some(stats) => format!(
            "{} file{}, +{} -{}",
            stats.files_changed,
            if stats.files_changed == 1 { "" } else { "s" },
            stats.insertions,
            stats.deletions
        ),
        None => "loading...".to_string(),
    };

    let mut lines = vec![
        metadata_line(&indent, "Change ID", change_id, label_style, dim),
        Line::from(vec![
            Span::styled(format!("{indent}Commit: "), label_style),
            Span::styled(
                details.commit_id_prefix.clone(),
                Style::default().fg(Color::Blue),
            ),
            Span::styled(
                details.commit_id_suffix.clone(),
                Style::default().add_modifier(ratatui::style::Modifier::DIM),
            ),
        ]),
        metadata_line(&indent, "Author", details.author.clone(), label_style, dim),
        metadata_line(&indent, "Date", details.timestamp.clone(), label_style, dim),
        changes_line(&indent, details, &stats_str, label_style, dim),
        Line::from(vec![Span::styled(
            format!("{indent}Description:"),
            label_style,
        )]),
    ];

    let description = details.full_description.trim();
    if description.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(format!("{indent}  "), label_style),
            Span::styled("(empty)", dim),
        ]));
        return lines;
    }

    for line in description.lines() {
        lines.push(Line::from(vec![
            Span::styled(format!("{indent}  "), label_style),
            Span::styled(line.to_string(), dim),
        ]));
    }

    lines
}

fn metadata_line(
    indent: &str,
    label: &str,
    value: String,
    label_style: Style,
    value_style: Style,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{indent}{label}: "), label_style),
        Span::styled(value, value_style),
    ])
}

fn changes_line(
    indent: &str,
    details: &RowDetails,
    stats_str: &str,
    label_style: Style,
    dim: Style,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{indent}Changes: "), label_style),
        Span::styled(
            format!(
                "+{}",
                details
                    .diff_stats
                    .as_ref()
                    .map(|stats| stats.insertions)
                    .unwrap_or(0)
            ),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" "),
        Span::styled(
            format!(
                "-{}",
                details
                    .diff_stats
                    .as_ref()
                    .map(|stats| stats.deletions)
                    .unwrap_or(0)
            ),
            Style::default().fg(Color::Red),
        ),
        Span::styled(format!(" ({stats_str})"), dim),
    ])
}
