mod generator;

use std::{fmt::Display, path::PathBuf};
use ast::Context;
use clap::{App, Arg};
use parser::Rule;
use pest::{Parser};

struct AppConfig {
    generators: Vec<String>,
    output_dir: PathBuf,
    files: Vec<PathBuf>,
    debug_parse: bool,
    debug_ast: bool,
    debug_context: bool
}

#[derive(Debug)]
enum ArgError {
    Msg(String)
}

impl From<&str> for ArgError {
    fn from(msg: &str) -> Self {
        ArgError::Msg(String::from(msg))
    }
}

impl Display for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("      Generators: {:?}\n", self.generators))?;
        f.write_fmt(format_args!("Output Directory: {:?}\n", self.output_dir))?;

        for file in &self.files {
            f.write_fmt(format_args!("           Input: {:?}\n", file))?;
        }

        Ok(())
    }
}

fn process_args(args: App) -> Result<AppConfig, ArgError> {
    let args = args.get_matches();

    let generators = args.values_of("generator").ok_or("No generators specified.")?;
    let output_dir = args.value_of("output").ok_or("No output dir specified.")?;
    let files = args.values_of("FILE").ok_or("No input files specified.")?;
    let debug_parse = args.is_present("debug-parse");
    let debug_ast = args.is_present("debug-ast");
    let debug_context = args.is_present("debug-context");

    let generators = generators.map(String::from).collect();
    let output_dir = PathBuf::from(output_dir);
    let files = files.map(PathBuf::from).collect();

    Ok(AppConfig {
        generators,
        output_dir,
        files,
        debug_parse,
        debug_ast,
        debug_context
    })
}

fn main() {

    let app = App::new("dcg")
        .version("0.1")
        .about("DataClass Generator")
        .arg(
            Arg::with_name("generator")
                .long("generator")
                .short("g")
                .long_help("Comma-separated list of generation executables. Must either be a path to an executable or \
the suffix part of a \"dcg-<suffix>\" executable that can be resolved based on the standard path.

The executable should take as arguments a port and the output dir. The JSON-ified AST will be written to the port.
")
                .required(true)
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .require_delimiter(true)
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .value_name("OUTPUT_DIR")
                .help("Path to the output directory.")
                .required(true)
                .takes_value(true)
                .multiple(false)
        )
        .arg(
            Arg::with_name("debug-parse")
                .long("debug-parse")
                .help("Display debug parse tree.")
        )
        .arg(
            Arg::with_name("debug-ast")
                .long("debug-ast")
                .help("Display debug AST.")
        )
        .arg(
            Arg::with_name("debug-context")
                .long("debug-context")
                .help("Display debug context.")
        )
        .arg(
            Arg::with_name("FILE")
                .help("Path to an input file.")
                .multiple(true)
                .required(true)
        );

    let config = match process_args(app) {
        Ok(config) => config,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1)
        }
    };

    println!("{}", config);

    let file_contents = {
        let mut file_contents = vec![];
        for file in config.files {
            let result = std::fs::read_to_string(file);
            match result {
                Ok(result) => file_contents.push(result),
                Err(e) => {
                    println!("Error: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        file_contents
    };

    if config.debug_parse {
        println!("Debug Parse Tree:");
        file_contents.iter().for_each(|f| {
            let pairs = parser::RawParser::parse(Rule::file, f)
                .unwrap_or_else(|e| panic!("{}", e));
            app_common::tree_format::display_debug_parse_tree(&pairs)
        });
    }

    let ast = parser::parse(&file_contents);
    if config.debug_ast {
        println!("Debug AST:");
        app_common::tree_format::display_debug_ast(&ast);
    }

    let ctx = Context::from(&ast);
    if config.debug_context {
        println!("Debug Context:");
        println!("{}", serde_json::to_string_pretty(&ctx).unwrap());    
    }


    for gen in &config.generators {
        if let Err(e) = generator::Generator::from(gen).run(&ctx, &config.output_dir) {
            println!("Error: {}", e);
        }
    }
}
