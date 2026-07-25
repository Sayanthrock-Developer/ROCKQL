use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use rockql_ast::Query;
use rockql_parser::{format_source, parse as parse_rockql};
use rockql_sql::{compile, Dialect};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "rockql",
    version,
    about = "Compile readable RockQL pipelines into SQL"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Compile RockQL source into SQL.
    Compile {
        /// RockQL file. Reads stdin when omitted.
        file: Option<PathBuf>,
        /// SQL target: generic, sqlite, or postgres.
        #[arg(long, default_value = "generic")]
        target: String,
    },
    /// Validate RockQL syntax without generating SQL.
    Check {
        /// RockQL file. Reads stdin when omitted.
        file: Option<PathBuf>,
    },
    /// Print the parsed abstract syntax tree as JSON.
    Ast {
        /// RockQL file. Reads stdin when omitted.
        file: Option<PathBuf>,
    },
    /// Format RockQL into canonical multiline source.
    Format {
        /// RockQL file. Reads stdin when omitted.
        file: Option<PathBuf>,
        /// Replace the input file instead of printing formatted source.
        #[arg(long)]
        write: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Compile { file, target } => {
            let input = read_input(file.as_deref())?;
            let query = parse_query(&input.source, &input.label)?;
            let dialect = target.parse::<Dialect>().map_err(anyhow::Error::msg)?;
            println!("{}", compile(&query, dialect)?);
        }
        Command::Check { file } => {
            let input = read_input(file.as_deref())?;
            parse_query(&input.source, &input.label)?;
            println!("OK");
        }
        Command::Ast { file } => {
            let input = read_input(file.as_deref())?;
            let query = parse_query(&input.source, &input.label)?;
            println!("{}", serde_json::to_string_pretty(&query)?);
        }
        Command::Format { file, write } => {
            let input = read_input(file.as_deref())?;
            let formatted = match format_source(&input.source) {
                Ok(formatted) => formatted,
                Err(diagnostics) => return report_diagnostics(&input.label, diagnostics),
            };

            if write {
                let Some(path) = file else {
                    bail!("--write requires a file path");
                };
                fs::write(&path, formatted)
                    .with_context(|| format!("failed to write {}", path.display()))?;
            } else {
                print!("{formatted}");
            }
        }
    }

    Ok(())
}

struct Input {
    source: String,
    label: String,
}

fn read_input(path: Option<&Path>) -> Result<Input> {
    match path {
        Some(path) => {
            let source = fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            Ok(Input {
                source,
                label: path.display().to_string(),
            })
        }
        None => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .context("failed to read RockQL source from stdin")?;
            Ok(Input {
                source,
                label: "<stdin>".to_owned(),
            })
        }
    }
}

fn parse_query(source: &str, label: &str) -> Result<Query> {
    match parse_rockql(source) {
        Ok(query) => Ok(query),
        Err(diagnostics) => report_diagnostics(label, diagnostics),
    }
}

fn report_diagnostics<T>(label: &str, diagnostics: Vec<rockql_parser::Diagnostic>) -> Result<T> {
    for diagnostic in diagnostics {
        eprintln!(
            "{label}:{}:{}: {}",
            diagnostic.span.line, diagnostic.span.column, diagnostic.message
        );
    }

    Err(anyhow!("RockQL validation failed"))
}
