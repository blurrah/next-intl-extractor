use std::{
    collections::{HashMap, HashSet},
    env,
    path::Path,
};

use anyhow::Result;
use log::warn;
use oxc::{
    allocator::Allocator,
    ast::{
        ast::{
            Argument, BindingPattern, BindingPatternKind, CallExpression, ChainElement, ChainExpression, Class, Expression, Function, FunctionType, MemberExpression, ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind, TSImportAttributes, TSImportType
        },
        visit::walk,
        Visit,
    },
    parser::Parser,
    span::SourceType,
    syntax::scope::ScopeFlags,
};

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
    /// Visiting individual functions (mostly components) and set up a new function scope
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
            let (call_expr, is_get_translations) = match &decl.init {
                Some(Expression::CallExpression(call_expr)) => (call_expr, false),
                Some(Expression::AwaitExpression(await_expr)) => {
                    if let Expression::CallExpression(call_expr) = &await_expr.argument {
                    (call_expr, true)
                    } else {
                        continue;
                    }
                }
                // Not a call expression, skip the declaration
                _ => continue,
            };

            let (callee_name, callee_span) = match &call_expr.callee {
                Expression::Identifier(ident) => (ident.name.to_string(), ident.span),
                _ => continue,
            };

            // Early return if the callee is not a useTranslations/getTranslations function function
            if callee_name != "useTranslations" && callee_name != "getTranslations" {
                continue;
            }

            let namespace = match extract_namespace_from_translations_call(call_expr, is_get_translations) {
                Some(namespace) => namespace,
                None => {
                    // TODO: Calculate line and column from span
                    warn!("Could not find namespace for translations call at {:?}", callee_span);
                    continue
                },
            };


            let decl_id = match &decl.id.kind {
                BindingPatternKind::BindingIdentifier(identer) => identer.name.to_string(),
                _ => continue,
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

    /// Visiting individual translator functions
    /// e.g. `t("key");` or `t.rich("key");`
    fn visit_call_expression(&mut self, node: &CallExpression) {
        match &node.callee {
            // Static member expression, e.g. `t.rich("key");`
            Expression::StaticMemberExpression(member_expr) => {
                if let Expression::Identifier(callee) = &member_expr.object {
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
            // Identifier, e.g. `t("key");`
            Expression::Identifier(callee) => {
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
            _ => (),
        }
    }
}

fn extract_namespace_from_translations_call(call_expr: &CallExpression, is_get_translations: bool) -> Option<String> {
    if is_get_translations{
        // For getTranslations, expect an object with a namespace property
        call_expr.arguments.first()
            .and_then(|arg| {
                if let Argument::ObjectExpression(obj) = arg {
                    obj.properties.iter().find_map(|prop| {
                        if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                            match (&prop.key, &prop.value) {
                                (PropertyKey::StaticIdentifier(key_ident), Expression::StringLiteral(value_lit))
                                    if key_ident.name == "namespace" => Some(value_lit.value.to_string()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
    } else {
        // For useTranslations, expect a string literal as the first argument
        call_expr.arguments.first()
            .and_then(|arg| {
                if let Argument::StringLiteral(str_lit) = arg {
                    Some(str_lit.value.to_string())
                } else {
                    None
                }
            })
    }
}
