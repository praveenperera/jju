use super::diff::FileDiff;
use ahash::HashMap;
use duct::cmd;
use eyre::{Context as _, Result};

#[derive(Debug, Clone, Copy)]
pub(crate) struct SplitHunkRepo;

impl SplitHunkRepo {
    pub(crate) fn load_diff(&self, revision: &str) -> Result<String> {
        cmd!("jj", "diff", "-r", revision, "--git")
            .stdout_capture()
            .stderr_capture()
            .read()
            .wrap_err("failed to get diff")
    }

    pub(crate) fn read_file_lines_or_empty(&self, revision: &str, path: &str) -> Vec<String> {
        cmd!("jj", "file", "show", "-r", revision, path)
            .stdout_capture()
            .stderr_capture()
            .read()
            .unwrap_or_default()
            .lines()
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(crate) fn collect_original_contents(
        &self,
        files: &[FileDiff],
        revision: &str,
    ) -> HashMap<String, String> {
        files.iter().fold(HashMap::default(), |mut contents, file| {
            let content = cmd!("jj", "file", "show", "-r", revision, file.path())
                .stdout_capture()
                .stderr_capture()
                .read()
                .unwrap_or_default();
            contents.insert(file.path().to_string(), content);
            contents
        })
    }

    pub(crate) fn execute_split(
        &self,
        revision: &str,
        message: &str,
        new_contents: &HashMap<String, String>,
        original_contents: &HashMap<String, String>,
    ) -> Result<()> {
        let parent_revision = format!("{revision}-");
        cmd!("jj", "new", &parent_revision)
            .stdout_null()
            .stderr_null()
            .run()
            .wrap_err("failed to create new commit")?;

        for (path, content) in new_contents {
            std::fs::write(path, content).wrap_err_with(|| format!("failed to write {path}"))?;
        }

        cmd!("jj", "describe", "-m", message)
            .stdout_null()
            .stderr_null()
            .run()
            .wrap_err("failed to set commit message")?;

        let split_change_id = cmd!(
            "jj",
            "log",
            "-r",
            "@",
            "--no-graph",
            "-T",
            "change_id.short()"
        )
        .stdout_capture()
        .stderr_capture()
        .read()
        .wrap_err("failed to get split commit change id")?;

        cmd!("jj", "rebase", "-s", revision, "-d", "@")
            .stdout_null()
            .stderr_null()
            .run()
            .wrap_err("failed to rebase original revision")?;

        cmd!("jj", "edit", revision)
            .stdout_null()
            .stderr_null()
            .run()
            .wrap_err("failed to edit original revision")?;

        for (path, content) in original_contents {
            std::fs::write(path, content).wrap_err_with(|| format!("failed to restore {path}"))?;
        }

        cmd!("jj", "edit", split_change_id.trim())
            .stdout_null()
            .stderr_null()
            .run()
            .wrap_err("failed to return to split commit")?;

        Ok(())
    }
}
