pub mod visitor;

use anyhow::Result;
use oxc::{allocator::Allocator, ast::Visit, parser::Parser, span::SourceType};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::visitor::TranslationFunctionVisitor;

pub fn extract_translations(file_path: &Path) -> Result<HashMap<String, HashSet<String>>> {
    let source_text = std::fs::read_to_string(file_path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file_path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
    }

    let program = ret.program;

    let mut translation_function_visitor = TranslationFunctionVisitor::new();
    translation_function_visitor.visit_program(&program);

    Ok(translation_function_visitor.merge_by_namespace())
}
