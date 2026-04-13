mod cli;
mod dispatch;
mod split_hunk;
mod stack_sync;
mod tree;

use clap::Parser;
pub use cli::{Jj, JjCmd};
use eyre::Result;
use log::debug;
use std::ffi::OsString;

pub fn run(args: &[OsString]) -> Result<()> {
    debug!("jj args: {args:?}");
    let flags = Jj::parse_from(args);
    dispatch::run_with_flags(flags)
}

#[cfg(test)]
mod tests {
    use super::{Jj, JjCmd};
    use clap::Parser;

    #[test]
    fn parses_neighborhood_flag_without_subcommand() {
        let flags = Jj::parse_from(["jju", "--neighborhood"]);

        assert!(flags.neighborhood);
        assert!(flags.subcommand.is_none());
    }

    #[test]
    fn parses_neighborhood_short_flag_without_subcommand() {
        let flags = Jj::parse_from(["jju", "-n"]);

        assert!(flags.neighborhood);
        assert!(flags.subcommand.is_none());
    }

    #[test]
    fn parses_tree_subcommand_without_neighborhood() {
        let flags = Jj::parse_from(["jju", "tree", "--full"]);

        assert!(!flags.neighborhood);
        assert!(matches!(
            flags.subcommand,
            Some(JjCmd::Tree { full: true, .. })
        ));
    }
}
