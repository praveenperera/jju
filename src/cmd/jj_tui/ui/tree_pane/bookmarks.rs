use crate::cmd::jj_tui::tree::BookmarkInfo;
use unicode_width::UnicodeWidthStr;

/// Format bookmarks to fit within max_width, showing "+N" for overflow
/// Diverged bookmarks are marked with * suffix
pub(crate) fn format_bookmarks_truncated(bookmarks: &[BookmarkInfo], max_width: usize) -> String {
    if bookmarks.is_empty() {
        return String::new();
    }

    if bookmarks.len() == 1 {
        return format_bookmark(&bookmarks[0]);
    }

    let mut result = String::new();

    for (index, bookmark) in bookmarks.iter().enumerate() {
        let bookmark_display = format_bookmark(bookmark);
        let remaining = bookmarks.len() - index - 1;
        let suffix = if remaining > 0 {
            format!(" +{}", remaining)
        } else {
            String::new()
        };
        let candidate = if result.is_empty() {
            format!("{bookmark_display}{suffix}")
        } else {
            format!("{result} {bookmark_display}{suffix}")
        };

        if candidate.width() <= max_width {
            if remaining == 0 {
                result = candidate;
            } else if result.is_empty() {
                result = bookmark_display;
            } else {
                result = format!("{result} {bookmark_display}");
            }
            continue;
        }

        let overflow = bookmarks.len() - index;
        if result.is_empty() {
            return format!("{} +{}", format_bookmark(&bookmarks[0]), overflow - 1);
        }
        return format!("{result} +{overflow}");
    }

    result
}

fn format_bookmark(bookmark: &BookmarkInfo) -> String {
    if bookmark.is_diverged {
        format!("{}*", bookmark.name)
    } else {
        bookmark.name.clone()
    }
}
