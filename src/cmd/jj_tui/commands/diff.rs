pub fn get_diff(rev: &str) -> eyre::Result<String> {
    jju_jj::ops::DiffOps.get_diff(rev)
}

pub fn get_stats(change_id: &str) -> eyre::Result<String> {
    jju_jj::ops::DiffOps.get_stats(change_id)
}
