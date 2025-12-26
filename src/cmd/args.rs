use clap::{
    Args, Parser, Subcommand,
    builder::styling::{self, AnsiColor},
};

use crate::providers::provider::ProviderKind;

pub const STYLING: styling::Styles = clap::builder::Styles::styled()
    .header(AnsiColor::White.on_default().bold())
    .usage(AnsiColor::BrightBlue.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Magenta.on_default())
    .error(AnsiColor::Red.on_default().bold())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles = STYLING)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// Override config option for this command
    #[arg(short = 'c', long, value_name = "KEY=VALUE")]
    pub config: Option<Vec<String>>,

    /// Override the current provider
    #[arg(short = 'p', long)]
    pub provider: Option<ProviderKind>,

    /// Provide an additional 'hint' to the LLM
    #[arg(short = 'H', long)]
    pub hint: Option<String>,

    /// Print with compact outputs (no pretty trees)
    #[arg(long)]
    pub compact: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Authenticate with GitHub OAuth to use the Gai provider
    Auth(AuthArgs),

    /// Prints gai repository status
    Status(StatusArgs),

    /// Show the commit history in the format of gai commits
    Log(LogArgs),

    /// Create commits from the diffs in the working tree
    Commit(CommitArgs),

    #[command(hide = true)]
    Test(TestArgs),
    /* /// Create a rebase plan for commits
    Rebase,

    /// Find a specific commit
    Find,

    /// Initiate interactive bisect
    Bisect, */
}

#[derive(Debug, Subcommand)]
pub enum Auth {
    /// Login using GitHub OAuth
    Login,

    /// Get the status of the logged-in user
    /// including requests made and when the count
    /// resets
    Status,

    /// Logout/clear the stored user token
    Logout,
}

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub auth: Auth,
}

// Each command has its own args struct
#[derive(Debug, Args)]
pub struct CommitArgs {
    /// Skips the confirmation prompt
    #[arg(short = 'y', long)]
    pub skip_confirmation: bool,

    /// Only generate for currently staged files/hunks
    #[arg(short = 's', long)]
    pub staged: bool,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Print verbose status with request prompt and diffs
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct LogArgs {
    /// Max number of commits to show
    #[arg(short = 'n', long)]
    pub number: Option<usize>,

    /// Reverse the order of commits
    #[arg(short = 'r', long)]
    pub reverse: bool,
}

#[derive(Debug, Args)]
pub struct TestArgs {
    pub count: Option<usize>,

    #[command(subcommand)]
    pub command: TestCommands,
}

#[derive(Debug, Subcommand)]
pub enum TestCommands {
    Status,
    Commit,
}
