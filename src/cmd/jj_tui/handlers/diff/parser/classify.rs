use crate::cmd::jj_tui::state::DiffLineKind;

pub(super) fn classify_diff_line<'a>(
    line: &'a str,
    current_file: &mut Option<String>,
) -> (DiffLineKind, Option<&'a str>) {
    if line.starts_with("diff --git") {
        if let Some(b_path) = line.split(" b/").nth(1) {
            *current_file = Some(b_path.to_string());
        }
        return (DiffLineKind::FileHeader, None);
    }

    if line.starts_with("+++") || line.starts_with("---") {
        return (DiffLineKind::FileHeader, None);
    }

    if line.starts_with("@@") {
        return (DiffLineKind::Hunk, None);
    }

    if let Some(rest) = line.strip_prefix('+') {
        return (DiffLineKind::Added, Some(rest));
    }

    if let Some(rest) = line.strip_prefix('-') {
        return (DiffLineKind::Removed, Some(rest));
    }

    if let Some(rest) = line.strip_prefix(' ') {
        return (DiffLineKind::Context, Some(rest));
    }

    (DiffLineKind::Context, Some(line))
}
