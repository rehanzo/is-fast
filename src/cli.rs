use clap::{Parser, ArgAction};

/// is-fast - Internet Search Fast from the Terminal
///
/// is-fast is a command-line tool that allows you to quickly search the internet
/// from a terminal-only environment. Instead of loading a full web browser,
/// it fetches the first search result from Google and presents only the key information.
///
/// Navigation Controls:
///
/// - Next result: n / →
///
/// - Go back: b / ←
///
/// - Scroll down: j / ↓
///
/// - Scroll up: k / ↑
///
/// - Page down: CTRL + d
///
/// - Page up: CTRL + u
///
/// - Quit: q
#[derive(Parser)]
#[command(name = "is-fast")]
#[command(about = "A fast content extractor for terminal-based internet searches")]
#[command(version = "1.0.0", author = "Joseph Daunt")]
#[command(after_help = "For more details, visit https://github.com/Magic-JD/is-fast")]
pub struct Cli {
    /// Generate a default configuration file
    ///
    /// Running this option will create a `config.toml` file inside the default configuration
    /// directory (`$XDG_CONFIG_HOME/is-fast/` or `~/.config/is-fast/`).
    /// If the directory does not exist, it will be created automatically.
    ///
    /// Example Usage:
    ///
    /// is-fast --generate-config
    #[arg(long, action = ArgAction::SetTrue, help = "Generate a default configuration file")]
    pub(crate) generate_config: bool,

    /// The search query to extract content from websites
    ///
    /// The provided string will be used as a search query. If multiple words
    /// are given, they will be combined into a single query.
    ///
    /// Example Usage:
    ///
    /// is-fast "how to install Rust"
    ///
    /// is-fast Rust tutorial
    #[arg(help = "The search query to extract content from websites")]
    pub(crate) query: Option<Vec<String>>,
}
