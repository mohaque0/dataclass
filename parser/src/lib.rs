extern crate pest;
#[macro_use] extern crate pest_derive;

mod raw_to_ast;

use pest::Parser;

#[derive(Parser)]
#[grammar = "dataclass.pest"]
pub struct RawParser;

pub fn parse(file_contents: &Vec<String>) -> ast::Root {
    let parsed_files = file_contents.iter()
        .map(
            |input| RawParser::parse(Rule::file, input)
                .unwrap_or_else(|e| panic!("{}", e))
                .next()
                .unwrap()
        )
        .map(raw_to_ast::convert_file)
        .collect();

    ast::Root::new(parsed_files)
}