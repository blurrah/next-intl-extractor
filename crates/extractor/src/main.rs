use std::{env, path::Path};

use anyhow::Result;
use oxc::{allocator::Allocator, ast::Visit, parser::Parser, span::SourceType};

use crate::visitor::TranslationFunctionVisitor;

mod visitor;

// TODO: Temp CLI tool, move to `cli` crate
fn main() -> Result<()> {
    env_logger::init();
    // TODO: Use proper argument parsing for input folders and output source.json file
    let name = env::args()
        .nth(1)
        .unwrap_or_else(|| "./examples/component.tsx".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
    }

    let program = ret.program;

    let mut translation_function_visitor = TranslationFunctionVisitor::new();
    translation_function_visitor.visit_program(&program);
    println!("{translation_function_visitor:#?}");
    let merged = translation_function_visitor.merge_by_namespace();
    println!("{merged:#?}");

    // TODO: Import existing translations json file and merge existing labels with input files

    // TODO: Write merged translations to a json file

    println!("Done!");

    Ok(())
}
