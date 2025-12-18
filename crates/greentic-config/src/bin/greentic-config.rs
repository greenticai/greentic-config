use clap::{Parser, Subcommand};
use greentic_config::ConfigResolver;

#[derive(Debug, Parser)]
#[command(name = "greentic-config", about = "Greentic configuration inspector")]
struct Cli {
    /// Project root override
    #[arg(long)]
    project_root: Option<std::path::PathBuf>,
    /// Explicit config path (replaces <project_root>/.greentic/config.toml discovery)
    #[arg(long)]
    config: Option<std::path::PathBuf>,
    /// Allow dev-only fields even when env_id is non-dev
    #[arg(long, default_value_t = false)]
    allow_dev: bool,
    /// Allow outbound network even when connection=offline
    #[arg(long, default_value_t = false)]
    allow_network: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show resolved configuration as JSON
    Show,
    /// Show explain report (text)
    Explain,
    /// Validate configuration (errors on invalid)
    Validate,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut resolver = ConfigResolver::new()
        .with_allow_dev(cli.allow_dev)
        .with_allow_network(cli.allow_network)
        .with_project_root_opt(cli.project_root.clone());
    if let Some(config) = cli.config.clone() {
        resolver = resolver.with_config_path(config);
    }

    match cli.command.unwrap_or(Command::Show) {
        Command::Show => {
            let resolved = resolver.load()?;
            println!("{}", serde_json::to_string_pretty(&resolved.config)?);
        }
        Command::Explain => {
            let resolved = resolver.load_detailed()?;
            println!("{}", resolved.explain().to_string());
        }
        Command::Validate => {
            let resolved = resolver.load()?;
            if resolved.warnings.is_empty() {
                println!("Configuration valid");
            } else {
                println!("Configuration valid with warnings:");
                for warn in &resolved.warnings {
                    println!("- {warn}");
                }
            }
        }
    }

    Ok(())
}
