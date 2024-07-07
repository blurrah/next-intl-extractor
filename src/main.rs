use std::{
    collections::{HashMap, HashSet},
    env,
    path::Path,
};

use anyhow::Result;
use oxc::{
    allocator::Allocator,
    ast::{
        ast::{
            BindingPattern, BindingPatternKind, CallExpression, Class, Expression, Function,
            FunctionType, TSImportAttributes, TSImportType,
        },
        visit::walk,
        Visit,
    },
    parser::Parser,
    span::SourceType,
    syntax::scope::ScopeFlags,
};

fn main() -> Result<()> {
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

    println!("Done!");

    Ok(())
}

#[derive(Debug)]
struct TranslationFunction {
    namespace: String,
    usages: HashSet<String>,
}

#[derive(Debug)]
struct TranslationFunctionVisitor {
    translation_functions: HashMap<String, TranslationFunction>,
    current_scope: Vec<String>,
}

impl TranslationFunctionVisitor {
    fn new() -> Self {
        Self {
            translation_functions: HashMap::new(),
            current_scope: Vec::new(),
        }
    }

    fn enter_scope(&mut self, name: &str) {
        self.current_scope.push(name.to_string());
    }

    fn exit_scope(&mut self) {
        self.current_scope.pop();
    }

    fn current_scope_name(&self) -> String {
        self.current_scope.join(".")
    }

    /// Merge translation functions by namespace
    ///
    /// Returns a hashmap with the namespace as key and a set of usages as value
    ///
    /// Since all translations are stored in one global json file we will need to merge all usages
    /// together in order to generate the correct json file
    fn merge_by_namespace(&self) -> HashMap<String, HashSet<String>> {
        let mut result: HashMap<String, HashSet<String>> = HashMap::new();
        for value in self.translation_functions.values() {
            let namespace = &value.namespace;
            let usages = &value.usages;
            if let Some(set) = result.get_mut(namespace) {
                set.extend(usages.clone());
            } else {
                result.insert(namespace.clone(), usages.clone());
            }
        }
        result
    }
}

impl<'a> Visit<'a> for TranslationFunctionVisitor {
    fn visit_function(&mut self, it: &Function<'a>, flags: Option<ScopeFlags>) {
        if let Some(ident) = &it.id {
            self.enter_scope(ident.name.as_str());
            println!("Entering scope: {}", self.current_scope_name());
        }
        walk::walk_function(self, it, flags);
        self.exit_scope();
    }

    fn visit_variable_declaration(&mut self, it: &oxc::ast::ast::VariableDeclaration<'a>) {
        for decl in &it.declarations {
            if let Some(Expression::CallExpression(call_expr)) = &decl.init {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    if ident.name == "useTranslations" {
                        // First argument as string
                        let namespace = if let Some(arg) = call_expr.arguments.first() {
                            if let Expression::StringLiteral(str_lit) = arg.to_expression() {
                                str_lit.value.to_string()
                            } else {
                                "default".to_string()
                            }
                        } else {
                            "default".to_string()
                        };

                        let decl_id =
                            if let BindingPatternKind::BindingIdentifier(identer) = &decl.id.kind {
                                identer.name.to_string()
                            } else {
                                // Shouldn't happen so let's just skip

                                return;
                            };

                        let scope = self.current_scope_name();
                        let key = format!("{}:{}", scope, decl_id);

                        self.translation_functions.insert(
                            key,
                            TranslationFunction {
                                namespace,
                                usages: HashSet::new(),
                            },
                        );
                    }
                }
            }
        }
    }
    fn visit_call_expression(&mut self, node: &CallExpression) {
        if let Expression::Identifier(callee) = &node.callee {
            let scope = self.current_scope_name();
            let key = format!("{}:{}", scope, callee.name);
            if let Some(translation_info) = self.translation_functions.get_mut(&key) {
                if let Some(arg) = node.arguments.first() {
                    if let Expression::StringLiteral(str_lit) = &arg.to_expression() {
                        translation_info.usages.insert(str_lit.value.to_string());
                    }
                }
            }
        }
    }
}
