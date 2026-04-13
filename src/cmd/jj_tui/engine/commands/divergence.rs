use super::super::{Effect, MessageKind, ReduceCtx};

pub(super) fn resolve_divergence(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    if !node.is_divergent {
        ctx.set_status("This revision is not divergent", MessageKind::Warning);
        return;
    }

    if node.divergent_versions.is_empty() {
        ctx.set_status("No divergent versions found", MessageKind::Error);
        return;
    }

    let local_version = node
        .divergent_versions
        .iter()
        .find(|version| version.is_local)
        .unwrap_or(&node.divergent_versions[0]);

    let abandon_ids: Vec<String> = node
        .divergent_versions
        .iter()
        .filter(|version| version.commit_id != local_version.commit_id)
        .map(|version| version.commit_id.clone())
        .collect();

    if abandon_ids.is_empty() {
        ctx.set_status(
            "Only one version exists, nothing to resolve",
            MessageKind::Warning,
        );
        return;
    }

    ctx.effects.push(Effect::SaveOperationForUndo);
    ctx.effects.push(Effect::RunResolveDivergence {
        keep_commit_id: local_version.commit_id.clone(),
        abandon_commit_ids: abandon_ids,
    });
    ctx.effects.push(Effect::RefreshTree);
}
