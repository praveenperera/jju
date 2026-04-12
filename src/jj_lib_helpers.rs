mod display;
mod prefixes;
mod queries;
mod revset;

/// Helpers for working with jj-lib directly instead of spawning jj CLI processes
///
/// Note: jj-lib is designed primarily for the jj CLI, so some operations
/// (especially git fetch/push) are easier to do via the CLI. This module
/// provides helpers for read operations and simple mutations.
use eyre::{Context, Result};
use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
use jj_lib::repo::{ReadonlyRepo, StoreFactories};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitDetails {
    pub unique_commit_prefix_len: usize,
    pub full_description: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: String,
}

/// Creates a minimal UserSettings for jj operations
pub fn create_user_settings() -> Result<UserSettings> {
    let config_text = r#"
        user.name = "jj-lib user"
        user.email = "jj-lib@localhost"
        operation.username = "jj-lib"
        operation.hostname = "localhost"
    "#;
    let mut config = StackedConfig::with_defaults();
    config.add_layer(ConfigLayer::parse(ConfigSource::User, config_text)?);
    UserSettings::from_config(config).wrap_err("failed to create user settings")
}

/// Wrapper around jj workspace and repo for convenient operations
pub struct JjRepo {
    workspace: Workspace,
    repo: Arc<ReadonlyRepo>,
}

impl JjRepo {
    /// Load a jj workspace from the given path (or current directory)
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let workspace_path = match path {
            Some(path) => path.to_path_buf(),
            None => std::env::current_dir().wrap_err("failed to get current directory")?,
        };

        let settings = create_user_settings()?;
        let store_factories = StoreFactories::default();
        let working_copy_factories = default_working_copy_factories();

        let workspace = Workspace::load(
            &settings,
            &workspace_path,
            &store_factories,
            &working_copy_factories,
        )
        .wrap_err("failed to load workspace")?;

        let repo = workspace
            .repo_loader()
            .load_at_head()
            .wrap_err("failed to load repo at head")?;

        Ok(Self { workspace, repo })
    }
}
