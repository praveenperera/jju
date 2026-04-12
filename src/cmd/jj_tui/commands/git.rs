pub fn push_bookmark(name: &str) -> eyre::Result<()> {
    jju_jj::ops::GitOps.push_bookmark(name)
}

pub fn import() -> eyre::Result<()> {
    jju_jj::ops::GitOps.import()
}

pub fn export() -> eyre::Result<()> {
    jju_jj::ops::GitOps.export()
}

pub fn push_all() -> eyre::Result<()> {
    jju_jj::ops::GitOps.push_all()
}

pub fn fetch() -> eyre::Result<()> {
    jju_jj::ops::GitOps.fetch()
}

/// Push a bookmark and create or open its PR
/// Returns true if an existing PR was opened, false if a new one was created
pub fn push_and_pr(bookmark: &str) -> eyre::Result<bool> {
    jju_jj::ops::GitOps.push_and_pr(bookmark)
}
