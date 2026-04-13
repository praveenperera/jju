mod order;
mod updates;

use super::App;

pub(super) fn start_detail_hydration(app: &mut App) {
    updates::start_detail_hydration(app, order::detail_hydration_order(app));
}

pub(super) fn apply_detail_updates(app: &mut App) {
    updates::apply_detail_updates(app);
}
