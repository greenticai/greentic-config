use clap::{Parser, Subcommand};
use greentic_config::{ConfigResolver, explain};

#[derive(Debug, Parser)]
#[command(name = "greentic-config", about = "Greentic configuration inspector")]
struct Cli {
    /// Project root override
    #[arg(long)]
    project_root: Option<std::path::PathBuf>,
    /// Allow dev-only fields even when env_id is non-dev
    #[arg(long, default_value_t = false)]
    allow_dev: bool,
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
    let resolver = ConfigResolver::new()
        .allow_dev(cli.allow_dev)
        .with_project_root_opt(cli.project_root.clone());
    let resolved = resolver.load()?;

    match cli.command.unwrap_or(Command::Show) {
        Command::Show => {
            println!("{}", serde_json::to_string_pretty(&resolved.config)?);
        }
        Command::Explain => {
            let report = explain(&resolved.config, &resolved.provenance, &resolved.warnings);
            println!("{}", report.text);
        }
        Command::Validate => {
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
