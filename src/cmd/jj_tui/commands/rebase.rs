/// Inline rebase: insert source after dest, reparenting dest's children under source
pub fn single(source: &str, dest: &str) -> eyre::Result<()> {
    jju_jj::ops::RebaseOps.single(source, dest)
}

/// Inline rebase with descendants
pub fn with_descendants(source: &str, dest: &str) -> eyre::Result<()> {
    jju_jj::ops::RebaseOps.with_descendants(source, dest)
}

/// Fork rebase: set dest as parent, dest's children unaffected
pub fn single_fork(source: &str, dest: &str) -> eyre::Result<()> {
    jju_jj::ops::RebaseOps.single_fork(source, dest)
}

/// Fork rebase with descendants
pub fn with_descendants_fork(source: &str, dest: &str) -> eyre::Result<()> {
    jju_jj::ops::RebaseOps.with_descendants_fork(source, dest)
}

/// Rebase single commit onto trunk()
pub fn single_onto_trunk(source: &str) -> eyre::Result<()> {
    jju_jj::ops::RebaseOps.single_onto_trunk(source)
}

/// Rebase commit with descendants onto trunk()
pub fn with_descendants_onto_trunk(source: &str) -> eyre::Result<()> {
    jju_jj::ops::RebaseOps.with_descendants_onto_trunk(source)
}
