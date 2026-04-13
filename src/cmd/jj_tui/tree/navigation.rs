mod cursor;
mod expansion;
mod focus;
mod scroll;

use super::TreeState;
use ahash::HashMap;

impl TreeState {
    pub fn bookmark_to_visible_index(&self) -> HashMap<String, usize> {
        let mut map = HashMap::default();
        for (visible_idx, entry) in self.projection.visible_entries.iter().enumerate() {
            let node = &self.snapshot.nodes[entry.node_index];
            for bookmark in &node.bookmarks {
                map.insert(bookmark.name.clone(), visible_idx);
            }
        }
        map
    }
}
