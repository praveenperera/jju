use super::SplitHunkOptions;
use super::application::SplitHunkApplication;
use super::diff::ParsedDiff;
use super::plan::SplitHunkPlan;
use super::preview::preview_plan;
use super::repo::SplitHunkRepo;
use super::selection::SplitHunkPlanner;
use colored::Colorize;
use eyre::Result;

#[derive(Debug, Clone)]
pub(super) struct SplitHunkWorkflow {
    repo: SplitHunkRepo,
    options: SplitHunkOptions,
}

impl SplitHunkWorkflow {
    pub(super) fn new(options: SplitHunkOptions, repo: SplitHunkRepo) -> Self {
        Self { repo, options }
    }

    pub(super) fn run(self) -> Result<()> {
        let diff = self.load_diff()?;
        if diff.is_empty() {
            return Ok(());
        }

        let plan = self.build_plan(diff)?;
        if self.options.preview {
            preview_plan(&plan);
            return Ok(());
        }

        if !plan.has_selection() {
            println!("{}", "No hunks matched selection criteria".yellow());
            return Ok(());
        }

        println!(
            "{} {} hunks",
            "Selected".green(),
            plan.selected_count().to_string().cyan()
        );

        let application = SplitHunkApplication::build(&plan, &self.repo, &self.options.revision)?;
        if self.options.dry_run {
            println!("\n{}", "Dry run - would commit:".yellow());
            for path in application.new_contents.keys() {
                println!("  {}", path.cyan());
            }
            return Ok(());
        }

        let message = self.options.commit_message()?;
        self.repo.execute_split(
            &self.options.revision,
            message,
            &application.new_contents,
            &application.original_contents,
        )?;

        println!("{} {}", "Created split commit:".green(), message.cyan());
        Ok(())
    }

    fn load_diff(&self) -> Result<ParsedDiff> {
        let diff_output = self.repo.load_diff(&self.options.revision)?;
        if diff_output.is_empty() {
            println!("{}", "No changes in revision".yellow());
            return Ok(ParsedDiff::empty());
        }

        let diff =
            ParsedDiff::parse(&diff_output).filter_by_path(self.options.file_filter.as_deref());
        if diff.is_empty() {
            println!("{}", "No matching files found".yellow());
        }

        Ok(diff)
    }

    fn build_plan(&self, diff: ParsedDiff) -> Result<SplitHunkPlan> {
        Ok(SplitHunkPlanner::from_options(&self.options)?.build(diff))
    }
}
