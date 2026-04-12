pub fn set(name: &str, rev: &str) -> eyre::Result<()> {
    jju_jj::ops::BookmarkOps.set(name, rev)
}

pub fn set_allow_backwards(name: &str, rev: &str) -> eyre::Result<()> {
    jju_jj::ops::BookmarkOps.set_allow_backwards(name, rev)
}

pub fn delete(name: &str) -> eyre::Result<()> {
    jju_jj::ops::BookmarkOps.delete(name)
}
