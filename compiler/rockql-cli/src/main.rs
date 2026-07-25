use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use rockql_parser::parse;
use rockql_sql::{compile, Dialect};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    let command = arguments.next().unwrap_or_else(|| "help".to_owned());
    let remaining: Vec<String> = arguments.collect();

    match command.as_str() {
        "compile" => {
            let options = Options::parse(&remaining, true)?;
            let source = read_source(options.input.as_deref())?;
            let query = parse(&source).map_err(|error| error.to_string())?;
            let sql = compile(&query, options.dialect).map_err(|error| error.to_string())?;
            println!("{sql}");
        }
        "check" => {
            let options = Options::parse(&remaining, false)?;
            let source = read_source(options.input.as_deref())?;
            parse(&source).map_err(|error| error.to_string())?;
            println!("RockQL query is valid.");
        }
        "format" => {
            let options = Options::parse(&remaining, false)?;
            let source = read_source(options.input.as_deref())?;
            let query = parse(&source).map_err(|error| error.to_string())?;
            print!("{}", rockql_formatter::format(&query));
        }
        "ast" => {
            let options = Options::parse(&remaining, false)?;
            let source = read_source(options.input.as_deref())?;
            let query = parse(&source).map_err(|error| error.to_string())?;
            println!("{query:#?}");
        }
        "help" | "--help" | "-h" => print_help(),
        "--version" | "-V" => println!("rockql {}", env!("CARGO_PKG_VERSION")),
        other => {
            return Err(format!(
                "unknown command `{other}`\n\nRun `rockql help` for usage."
            ))
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Options {
    input: Option<String>,
    dialect: Dialect,
}

impl Options {
    fn parse(arguments: &[String], allow_target: bool) -> Result<Self, String> {
        let mut input = None;
        let mut dialect = Dialect::Generic;
        let mut index = 0;
        while index < arguments.len() {
            match arguments[index].as_str() {
                "--target" if allow_target => {
                    index += 1;
                    let value = arguments
                        .get(index)
                        .ok_or_else(|| "missing value after `--target`".to_owned())?;
                    dialect = Dialect::from_name(value).map_err(|error| error.to_string())?;
                }
                "-" => input = Some("-".to_owned()),
                argument if argument.starts_with('-') => {
                    return Err(format!("unknown option `{argument}`"));
                }
                path => {
                    if input.replace(path.to_owned()).is_some() {
                        return Err("only one input file may be supplied".to_owned());
                    }
                }
            }
            index += 1;
        }
        Ok(Self { input, dialect })
    }
}

fn read_source(path: Option<&str>) -> Result<String, String> {
    match path {
        Some(path) if path != "-" => {
            fs::read_to_string(path).map_err(|error| format!("cannot read `{path}`: {error}"))
        }
        _ => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|error| format!("cannot read stdin: {error}"))?;
            Ok(source)
        }
    }
}

fn print_help() {
    println!(
        "RockQL — readable data pipelines for every database\n\n\
Usage:\n  rockql compile [FILE|-] [--target generic|sqlite|postgres]\n  rockql check [FILE|-]\n  rockql format [FILE|-]\n  rockql ast [FILE|-]\n  rockql --version\n\n\
When FILE is omitted or `-`, RockQL reads from standard input."
    );
}
