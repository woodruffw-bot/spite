//! Parser integration coverage in one Cargo test executable.

#[path = "arrays.rs"]
mod arrays;
#[path = "arrow_covers.rs"]
mod arrow_covers;
#[path = "arrows.rs"]
mod arrows;
#[path = "assignment.rs"]
mod assignment;
#[path = "assignment_patterns.rs"]
mod assignment_patterns;
#[path = "async_arrows.rs"]
mod async_arrows;
#[path = "async_function_syntax.rs"]
mod async_function_syntax;
#[path = "async_methods.rs"]
mod async_methods;
#[path = "await_expression.rs"]
mod await_expression;
#[path = "bigint.rs"]
mod bigint;
#[path = "binary_chains.rs"]
mod binary_chains;
#[path = "calls.rs"]
mod calls;
#[path = "catch.rs"]
mod catch;
#[path = "catch_parameters.rs"]
mod catch_parameters;
#[path = "catch_patterns.rs"]
mod catch_patterns;
#[path = "class_fields.rs"]
mod class_fields;
#[path = "class_static_blocks.rs"]
mod class_static_blocks;
#[path = "classes.rs"]
mod classes;
#[path = "construction.rs"]
mod construction;
#[path = "debugger.rs"]
mod debugger;
#[path = "default_parameters.rs"]
mod default_parameters;
#[path = "delete.rs"]
mod delete;
#[path = "derived_classes.rs"]
mod derived_classes;
#[path = "dynamic_async_function.rs"]
mod dynamic_async_function;
#[path = "dynamic_function.rs"]
mod dynamic_function;
#[path = "dynamic_source_utf16.rs"]
mod dynamic_source_utf16;
#[path = "eval_context.rs"]
mod eval_context;
#[path = "for_in.rs"]
mod for_in;
#[path = "for_loops.rs"]
mod for_loops;
#[path = "for_of.rs"]
mod for_of;
#[path = "function_bodies.rs"]
mod function_bodies;
#[path = "function_declarations.rs"]
mod function_declarations;
#[path = "function_expressions.rs"]
mod function_expressions;
#[path = "identifiers.rs"]
mod identifiers;
#[path = "in_operator.rs"]
mod in_operator;
#[path = "instanceof.rs"]
mod instanceof;
#[path = "json.rs"]
mod json;
#[path = "labels.rs"]
mod labels;
#[path = "legacy_literals.rs"]
mod legacy_literals;
#[path = "lexical_goals.rs"]
mod lexical_goals;
#[path = "lexical_patterns.rs"]
mod lexical_patterns;
#[path = "loops.rs"]
mod loops;
#[path = "members.rs"]
mod members;
#[path = "new_target.rs"]
mod new_target;
#[path = "object_methods.rs"]
mod object_methods;
#[path = "objects.rs"]
mod objects;
#[path = "optional_chaining.rs"]
mod optional_chaining;
#[path = "parameter_patterns.rs"]
mod parameter_patterns;
#[path = "parser.rs"]
mod parser;
#[path = "pattern_covers.rs"]
mod pattern_covers;
#[path = "private_elements.rs"]
mod private_elements;
#[path = "regexp_literals.rs"]
mod regexp_literals;
#[path = "regexp_metadata.rs"]
mod regexp_metadata;
#[path = "regexp_names.rs"]
mod regexp_names;
#[path = "regexp_pattern.rs"]
mod regexp_pattern;
#[path = "regexp_pattern_source.rs"]
mod regexp_pattern_source;
#[path = "regexp_properties.rs"]
mod regexp_properties;
#[path = "regexp_reference_metadata.rs"]
mod regexp_reference_metadata;
#[path = "regexp_sets.rs"]
mod regexp_sets;
#[path = "regexp_validation.rs"]
mod regexp_validation;
#[path = "rest_parameters.rs"]
mod rest_parameters;
#[path = "scripts_utf16.rs"]
mod scripts_utf16;
#[path = "streaming.rs"]
mod streaming;
#[path = "super_properties.rs"]
mod super_properties;
#[path = "switch.rs"]
mod switch;
#[path = "tagged_templates.rs"]
mod tagged_templates;
#[path = "templates.rs"]
mod templates;
#[path = "test262.rs"]
mod test262;
#[path = "this.rs"]
mod this;
#[path = "try_finally.rs"]
mod try_finally;
#[path = "updates.rs"]
mod updates;
#[path = "using.rs"]
mod using;
#[path = "var.rs"]
mod var;
#[path = "var_patterns.rs"]
mod var_patterns;
#[path = "with.rs"]
mod with;

#[path = "for_await.rs"]
mod for_await;

#[path = "generator_functions.rs"]
mod generator_functions;

#[path = "generator_methods.rs"]
mod generator_methods;

#[path = "dynamic_generator_function.rs"]
mod dynamic_generator_function;

#[path = "script_module_syntax.rs"]
mod script_module_syntax;

#[path = "import_call.rs"]
mod import_call;

#[path = "module_goal.rs"]
mod module_goal;
