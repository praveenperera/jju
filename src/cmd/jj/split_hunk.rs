pub(crate) struct SplitHunkOptions {
    pub(crate) message: Option<String>,
    pub(crate) revision: String,
    pub(crate) file_filter: Option<String>,
    pub(crate) lines: Option<String>,
    pub(crate) hunks: Option<String>,
    pub(crate) pattern: Option<String>,
    pub(crate) preview: bool,
    pub(crate) invert: bool,
    pub(crate) dry_run: bool,
}

pub(crate) struct SplitHunkCommand {
    options: SplitHunkOptions,
}

impl SplitHunkCommand {
    pub(crate) fn new(options: SplitHunkOptions) -> Self {
        Self { options }
    }

    pub(crate) fn run(self) -> eyre::Result<()> {
        jju_jj::split_hunk::SplitHunkCommand::new(jju_jj::split_hunk::SplitHunkOptions {
            message: self.options.message,
            revision: self.options.revision,
            file_filter: self.options.file_filter,
            lines: self.options.lines,
            hunks: self.options.hunks,
            pattern: self.options.pattern,
            preview: self.options.preview,
            invert: self.options.invert,
            dry_run: self.options.dry_run,
        })
        .run()
    }
}
