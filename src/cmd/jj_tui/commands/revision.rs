pub fn edit(rev: &str) -> eyre::Result<()> {
    jju_jj::ops::RevisionOps.edit(rev)
}

pub fn new(rev: &str) -> eyre::Result<()> {
    jju_jj::ops::RevisionOps.new_commit(rev)
}

pub fn commit(message: &str) -> eyre::Result<()> {
    jju_jj::ops::RevisionOps.commit(message)
}

pub fn abandon(revset: &str) -> eyre::Result<()> {
    jju_jj::ops::RevisionOps.abandon(revset)
}
