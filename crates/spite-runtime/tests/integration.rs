//! Runtime integration coverage in one Cargo test executable.

mod common;

#[path = "cases/aggregate_errors.rs"]
mod aggregate_errors;
#[path = "cases/array_at.rs"]
mod array_at;
#[path = "cases/array_callbacks.rs"]
mod array_callbacks;
#[path = "cases/array_class_species.rs"]
mod array_class_species;
#[path = "cases/array_concat.rs"]
mod array_concat;
#[path = "cases/array_copy.rs"]
mod array_copy;
#[path = "cases/array_find.rs"]
mod array_find;
#[path = "cases/array_flatten.rs"]
mod array_flatten;
#[path = "cases/array_from.rs"]
mod array_from;
#[path = "cases/array_front.rs"]
mod array_front;
#[path = "cases/array_index_lookup.rs"]
mod array_index_lookup;
#[path = "cases/array_iteration.rs"]
mod array_iteration;
#[path = "cases/array_linear_truncation.rs"]
mod array_linear_truncation;
#[path = "cases/array_literals.rs"]
mod array_literals;
#[path = "cases/array_locale_string.rs"]
mod array_locale_string;
#[path = "cases/array_mapping.rs"]
mod array_mapping;
#[path = "cases/array_mutation.rs"]
mod array_mutation;
#[path = "cases/array_of.rs"]
mod array_of;
#[path = "cases/array_range.rs"]
mod array_range;
#[path = "cases/array_reduce.rs"]
mod array_reduce;
#[path = "cases/array_reverse.rs"]
mod array_reverse;
#[path = "cases/array_search.rs"]
mod array_search;
#[path = "cases/array_slice.rs"]
mod array_slice;
#[path = "cases/array_sort.rs"]
mod array_sort;
#[path = "cases/array_splice.rs"]
mod array_splice;
#[path = "cases/array_spread.rs"]
mod array_spread;
#[path = "cases/array_strings.rs"]
mod array_strings;
#[path = "cases/array_to_spliced.rs"]
mod array_to_spliced;
#[path = "cases/arrays.rs"]
mod arrays;
#[path = "cases/arrows.rs"]
mod arrows;
#[path = "cases/assignment.rs"]
mod assignment;
#[path = "cases/assignment_patterns.rs"]
mod assignment_patterns;
#[path = "cases/bigint.rs"]
mod bigint;
#[path = "cases/bigint_api.rs"]
mod bigint_api;
#[path = "cases/bigint_width.rs"]
mod bigint_width;
#[path = "cases/binary_chains.rs"]
mod binary_chains;
#[path = "cases/boolean.rs"]
mod boolean;
#[path = "cases/builtin_functions.rs"]
mod builtin_functions;
#[path = "cases/call_spread.rs"]
mod call_spread;
#[path = "cases/calls.rs"]
mod calls;
#[path = "cases/catch.rs"]
mod catch;
#[path = "cases/catch_parameters.rs"]
mod catch_parameters;
#[path = "cases/catch_patterns.rs"]
mod catch_patterns;
#[path = "cases/class_fields.rs"]
mod class_fields;
#[path = "cases/class_static_blocks.rs"]
mod class_static_blocks;
#[path = "cases/classes.rs"]
mod classes;
#[path = "cases/construction.rs"]
mod construction;
#[path = "cases/date.rs"]
mod date;
#[path = "cases/date_invalid_local.rs"]
mod date_invalid_local;
#[path = "cases/date_local_getters.rs"]
mod date_local_getters;
#[path = "cases/date_local_inputs.rs"]
mod date_local_inputs;
#[path = "cases/date_local_setters.rs"]
mod date_local_setters;
#[path = "cases/date_local_strings.rs"]
mod date_local_strings;
#[path = "cases/date_locale_strings.rs"]
mod date_locale_strings;
#[path = "cases/date_utc.rs"]
mod date_utc;
#[path = "cases/date_utc_calendar_setters.rs"]
mod date_utc_calendar_setters;
#[path = "cases/date_utc_setters.rs"]
mod date_utc_setters;
#[path = "cases/date_utc_string.rs"]
mod date_utc_string;
#[path = "cases/debugger.rs"]
mod debugger;
#[path = "cases/default_parameters.rs"]
mod default_parameters;
#[path = "cases/delete.rs"]
mod delete;
#[path = "cases/derived_classes.rs"]
mod derived_classes;
#[path = "cases/dynamic_function.rs"]
mod dynamic_function;
#[path = "cases/dynamic_source_utf16.rs"]
mod dynamic_source_utf16;
#[path = "cases/errors.rs"]
mod errors;
#[path = "cases/eval_direct.rs"]
mod eval_direct;
#[path = "cases/eval_indirect.rs"]
mod eval_indirect;
#[path = "cases/eval_non_string.rs"]
mod eval_non_string;
#[path = "cases/evaluation.rs"]
mod evaluation;
#[path = "cases/for_in.rs"]
mod for_in;
#[path = "cases/for_loops.rs"]
mod for_loops;
#[path = "cases/for_of.rs"]
mod for_of;
#[path = "cases/function_apply.rs"]
mod function_apply;
#[path = "cases/function_bind.rs"]
mod function_bind;
#[path = "cases/function_bodies.rs"]
mod function_bodies;
#[path = "cases/function_gaps.rs"]
mod function_gaps;
#[path = "cases/function_intrinsics.rs"]
mod function_intrinsics;
#[path = "cases/function_methods.rs"]
mod function_methods;
#[path = "cases/global_numeric_predicates.rs"]
mod global_numeric_predicates;
#[path = "cases/globals.rs"]
mod globals;
#[path = "cases/host_exceptions.rs"]
mod host_exceptions;
#[path = "cases/in_operator.rs"]
mod in_operator;
#[path = "cases/instanceof.rs"]
mod instanceof;
#[path = "cases/iterator_callbacks.rs"]
mod iterator_callbacks;
#[path = "cases/iterator_concat.rs"]
mod iterator_concat;
#[path = "cases/iterator_constructor.rs"]
mod iterator_constructor;
#[path = "cases/iterator_flat_map.rs"]
mod iterator_flat_map;
#[path = "cases/iterator_for_each.rs"]
mod iterator_for_each;
#[path = "cases/iterator_from.rs"]
mod iterator_from;
#[path = "cases/iterator_limits.rs"]
mod iterator_limits;
#[path = "cases/iterator_predicates.rs"]
mod iterator_predicates;
#[path = "cases/iterator_reduce.rs"]
mod iterator_reduce;
#[path = "cases/iterator_reflection.rs"]
mod iterator_reflection;
#[path = "cases/iterator_to_array.rs"]
mod iterator_to_array;
#[path = "cases/json_parse.rs"]
mod json_parse;
#[path = "cases/json_raw.rs"]
mod json_raw;
#[path = "cases/json_reviver.rs"]
mod json_reviver;
#[path = "cases/json_stringify.rs"]
mod json_stringify;
#[path = "cases/labels.rs"]
mod labels;
#[path = "cases/legacy_literals.rs"]
mod legacy_literals;
#[path = "cases/lexical_patterns.rs"]
mod lexical_patterns;
#[path = "cases/limits.rs"]
mod limits;
#[path = "cases/loops.rs"]
mod loops;
#[path = "cases/map.rs"]
mod map;
#[path = "cases/mapped_arguments.rs"]
mod mapped_arguments;
#[path = "cases/math_float_rounding.rs"]
mod math_float_rounding;
#[path = "cases/math_hypot.rs"]
mod math_hypot;
#[path = "cases/math_integer_extrema.rs"]
mod math_integer_extrema;
#[path = "cases/math_inverse.rs"]
mod math_inverse;
#[path = "cases/math_log_exp.rs"]
mod math_log_exp;
#[path = "cases/math_power_root.rs"]
mod math_power_root;
#[path = "cases/math_random.rs"]
mod math_random;
#[path = "cases/math_rounding.rs"]
mod math_rounding;
#[path = "cases/math_sum_precise.rs"]
mod math_sum_precise;
#[path = "cases/math_trigonometric.rs"]
mod math_trigonometric;
#[path = "cases/new_target.rs"]
mod new_target;
#[path = "cases/number.rs"]
mod number;
#[path = "cases/number_formatting.rs"]
mod number_formatting;
#[path = "cases/numeric_parsing.rs"]
mod numeric_parsing;
#[path = "cases/object_builtins.rs"]
mod object_builtins;
#[path = "cases/object_coercion.rs"]
mod object_coercion;
#[path = "cases/object_copy.rs"]
mod object_copy;
#[path = "cases/object_creation.rs"]
mod object_creation;
#[path = "cases/object_descriptors.rs"]
mod object_descriptors;
#[path = "cases/object_from_entries.rs"]
mod object_from_entries;
#[path = "cases/object_group_by.rs"]
mod object_group_by;
#[path = "cases/object_integrity.rs"]
mod object_integrity;
#[path = "cases/object_keys.rs"]
mod object_keys;
#[path = "cases/object_literals.rs"]
mod object_literals;
#[path = "cases/object_methods.rs"]
mod object_methods;
#[path = "cases/object_prototypes.rs"]
mod object_prototypes;
#[path = "cases/object_spread.rs"]
mod object_spread;
#[path = "cases/object_values.rs"]
mod object_values;
#[path = "cases/objects.rs"]
mod objects;
#[path = "cases/optional_chaining.rs"]
mod optional_chaining;
#[path = "cases/ordinary_functions.rs"]
mod ordinary_functions;
#[path = "cases/parameter_patterns.rs"]
mod parameter_patterns;
#[path = "cases/primitive_properties.rs"]
mod primitive_properties;
#[path = "cases/private_fields.rs"]
mod private_fields;
#[path = "cases/private_methods.rs"]
mod private_methods;
#[path = "cases/private_syntax.rs"]
mod private_syntax;
#[path = "cases/properties.rs"]
mod properties;
#[path = "cases/prototypes.rs"]
mod prototypes;
#[path = "cases/realm_collection.rs"]
mod realm_collection;
#[path = "cases/reflect_call.rs"]
mod reflect_call;
#[path = "cases/reflect_descriptor.rs"]
mod reflect_descriptor;
#[path = "cases/reflect_property.rs"]
mod reflect_property;
#[path = "cases/reflect_prototype.rs"]
mod reflect_prototype;
#[path = "cases/reflect_set.rs"]
mod reflect_set;
#[path = "cases/regexp_alternative_captures.rs"]
mod regexp_alternative_captures;
#[path = "cases/regexp_anchored_groups.rs"]
mod regexp_anchored_groups;
#[path = "cases/regexp_anchored_repeated_groups.rs"]
mod regexp_anchored_repeated_groups;
#[path = "cases/regexp_anchors.rs"]
mod regexp_anchors;
#[path = "cases/regexp_backreference_alternatives.rs"]
mod regexp_backreference_alternatives;
#[path = "cases/regexp_backreference_assertions.rs"]
mod regexp_backreference_assertions;
#[path = "cases/regexp_backreference_characters.rs"]
mod regexp_backreference_characters;
#[path = "cases/regexp_backreference_inner_alternatives.rs"]
mod regexp_backreference_inner_alternatives;
#[path = "cases/regexp_backreference_nested_alternatives.rs"]
mod regexp_backreference_nested_alternatives;
#[path = "cases/regexp_backreference_sequential_alternatives.rs"]
mod regexp_backreference_sequential_alternatives;
#[path = "cases/regexp_branch_groups.rs"]
mod regexp_branch_groups;
#[path = "cases/regexp_capture_sharing.rs"]
mod regexp_capture_sharing;
#[path = "cases/regexp_captured_quantified_references.rs"]
mod regexp_captured_quantified_references;
#[path = "cases/regexp_captured_reference_sequences.rs"]
mod regexp_captured_reference_sequences;
#[path = "cases/regexp_captures.rs"]
mod regexp_captures;
#[path = "cases/regexp_character.rs"]
mod regexp_character;
#[path = "cases/regexp_character_unit_choices.rs"]
mod regexp_character_unit_choices;
#[path = "cases/regexp_constructor.rs"]
mod regexp_constructor;
#[path = "cases/regexp_count_one_captured_atoms.rs"]
mod regexp_count_one_captured_atoms;
#[path = "cases/regexp_count_one_multi_copy_captured_atoms.rs"]
mod regexp_count_one_multi_copy_captured_atoms;
#[path = "cases/regexp_count_one_partial_multi_copy_captured_atoms.rs"]
mod regexp_count_one_partial_multi_copy_captured_atoms;
#[path = "cases/regexp_count_one_reference_assertions.rs"]
mod regexp_count_one_reference_assertions;
#[path = "cases/regexp_counted_open_lookbehind.rs"]
mod regexp_counted_open_lookbehind;
#[path = "cases/regexp_counted_outside_reference_lookbehind.rs"]
mod regexp_counted_outside_reference_lookbehind;
#[path = "cases/regexp_dependent_outside_lookbehind.rs"]
mod regexp_dependent_outside_lookbehind;
#[path = "cases/regexp_disjunction.rs"]
mod regexp_disjunction;
#[path = "cases/regexp_empty_repetitions.rs"]
mod regexp_empty_repetitions;
#[path = "cases/regexp_empty_unit_branches.rs"]
mod regexp_empty_unit_branches;
#[path = "cases/regexp_escape.rs"]
mod regexp_escape;
#[path = "cases/regexp_exact_count_captured_atoms.rs"]
mod regexp_exact_count_captured_atoms;
#[path = "cases/regexp_exact_reference_assertions.rs"]
mod regexp_exact_reference_assertions;
#[path = "cases/regexp_exec.rs"]
mod regexp_exec;
#[path = "cases/regexp_fixed_alternatives.rs"]
mod regexp_fixed_alternatives;
#[path = "cases/regexp_fixed_lookahead_captures.rs"]
mod regexp_fixed_lookahead_captures;
#[path = "cases/regexp_fixed_lookahead_in_lookbehind.rs"]
mod regexp_fixed_lookahead_in_lookbehind;
#[path = "cases/regexp_fixed_lookbehind.rs"]
mod regexp_fixed_lookbehind;
#[path = "cases/regexp_fixed_lookbehind_assertion_counts.rs"]
mod regexp_fixed_lookbehind_assertion_counts;
#[path = "cases/regexp_fixed_lookbehind_captures.rs"]
mod regexp_fixed_lookbehind_captures;
#[path = "cases/regexp_fixed_lookbehind_choices.rs"]
mod regexp_fixed_lookbehind_choices;
#[path = "cases/regexp_fixed_lookbehind_counts.rs"]
mod regexp_fixed_lookbehind_counts;
#[path = "cases/regexp_fixed_lookbehind_empty_counts.rs"]
mod regexp_fixed_lookbehind_empty_counts;
#[path = "cases/regexp_fixed_lookbehind_repeated_captures.rs"]
mod regexp_fixed_lookbehind_repeated_captures;
#[path = "cases/regexp_fixed_lookbehind_sequences.rs"]
mod regexp_fixed_lookbehind_sequences;
#[path = "cases/regexp_fixed_lookbehind_zero_wrappers.rs"]
mod regexp_fixed_lookbehind_zero_wrappers;
#[path = "cases/regexp_fixed_unit_choices.rs"]
mod regexp_fixed_unit_choices;
#[path = "cases/regexp_fixed_units_outside_lookbehind.rs"]
mod regexp_fixed_units_outside_lookbehind;
#[path = "cases/regexp_forward_local_lookahead.rs"]
mod regexp_forward_local_lookahead;
#[path = "cases/regexp_forward_target_lookbehind.rs"]
mod regexp_forward_target_lookbehind;
#[path = "cases/regexp_generic.rs"]
mod regexp_generic;
#[path = "cases/regexp_identity.rs"]
mod regexp_identity;
#[path = "cases/regexp_inverted_unit_choices.rs"]
mod regexp_inverted_unit_choices;
#[path = "cases/regexp_linear_reference_assertions.rs"]
mod regexp_linear_reference_assertions;
#[path = "cases/regexp_literal_continuation_captures.rs"]
mod regexp_literal_continuation_captures;
#[path = "cases/regexp_literal_prefixes.rs"]
mod regexp_literal_prefixes;
#[path = "cases/regexp_literals.rs"]
mod regexp_literals;
#[path = "cases/regexp_local_reference_targets.rs"]
mod regexp_local_reference_targets;
#[path = "cases/regexp_lookahead.rs"]
mod regexp_lookahead;
#[path = "cases/regexp_lookbehind_local_future_reads.rs"]
mod regexp_lookbehind_local_future_reads;
#[path = "cases/regexp_match_all.rs"]
mod regexp_match_all;
#[path = "cases/regexp_match_search.rs"]
mod regexp_match_search;
#[path = "cases/regexp_middle_empty_captures.rs"]
mod regexp_middle_empty_captures;
#[path = "cases/regexp_mixed_empty_reference_lookbehind.rs"]
mod regexp_mixed_empty_reference_lookbehind;
#[path = "cases/regexp_mixed_outside_lookbehind.rs"]
mod regexp_mixed_outside_lookbehind;
#[path = "cases/regexp_named_backreferences.rs"]
mod regexp_named_backreferences;
#[path = "cases/regexp_named_groups.rs"]
mod regexp_named_groups;
#[path = "cases/regexp_named_reference_bindings.rs"]
mod regexp_named_reference_bindings;
#[path = "cases/regexp_nested_choice_loops.rs"]
mod regexp_nested_choice_loops;
#[path = "cases/regexp_nested_exact_reference_units.rs"]
mod regexp_nested_exact_reference_units;
#[path = "cases/regexp_nested_fixed_lookbehind.rs"]
mod regexp_nested_fixed_lookbehind;
#[path = "cases/regexp_nested_outside_lookbehind.rs"]
mod regexp_nested_outside_lookbehind;
#[path = "cases/regexp_nested_progressing_choices.rs"]
mod regexp_nested_progressing_choices;
#[path = "cases/regexp_nested_unit_choices.rs"]
mod regexp_nested_unit_choices;
#[path = "cases/regexp_nested_zero_counts.rs"]
mod regexp_nested_zero_counts;
#[path = "cases/regexp_nested_zero_references.rs"]
mod regexp_nested_zero_references;
#[path = "cases/regexp_noncapturing.rs"]
mod regexp_noncapturing;
#[path = "cases/regexp_nullable_choice_loops.rs"]
mod regexp_nullable_choice_loops;
#[path = "cases/regexp_nullable_choice_previous_gaps.rs"]
mod regexp_nullable_choice_previous_gaps;
#[path = "cases/regexp_numbered_backreferences.rs"]
mod regexp_numbered_backreferences;
#[path = "cases/regexp_ordinary_fallback.rs"]
mod regexp_ordinary_fallback;
#[path = "cases/regexp_outer_captures.rs"]
mod regexp_outer_captures;
#[path = "cases/regexp_outer_noncapturing.rs"]
mod regexp_outer_noncapturing;
#[path = "cases/regexp_outside_lookahead_in_lookbehind.rs"]
mod regexp_outside_lookahead_in_lookbehind;
#[path = "cases/regexp_outside_reference_lookbehind.rs"]
mod regexp_outside_reference_lookbehind;
#[path = "cases/regexp_partial_atom_captures.rs"]
mod regexp_partial_atom_captures;
#[path = "cases/regexp_partial_reference_captures.rs"]
mod regexp_partial_reference_captures;
#[path = "cases/regexp_partial_repetition_captures.rs"]
mod regexp_partial_repetition_captures;
#[path = "cases/regexp_prefix_captures.rs"]
mod regexp_prefix_captures;
#[path = "cases/regexp_progressing_choice_repetitions.rs"]
mod regexp_progressing_choice_repetitions;
#[path = "cases/regexp_quantified_alternatives.rs"]
mod regexp_quantified_alternatives;
#[path = "cases/regexp_quantified_anchors.rs"]
mod regexp_quantified_anchors;
#[path = "cases/regexp_quantified_atoms.rs"]
mod regexp_quantified_atoms;
#[path = "cases/regexp_quantified_captures.rs"]
mod regexp_quantified_captures;
#[path = "cases/regexp_quantified_continuations.rs"]
mod regexp_quantified_continuations;
#[path = "cases/regexp_quantified_line_assertions.rs"]
mod regexp_quantified_line_assertions;
#[path = "cases/regexp_quantified_noncapturing.rs"]
mod regexp_quantified_noncapturing;
#[path = "cases/regexp_quantified_progressing_bodies.rs"]
mod regexp_quantified_progressing_bodies;
#[path = "cases/regexp_quantified_reference_characters.rs"]
mod regexp_quantified_reference_characters;
#[path = "cases/regexp_quantified_references.rs"]
mod regexp_quantified_references;
#[path = "cases/regexp_reference_empty_captures.rs"]
mod regexp_reference_empty_captures;
#[path = "cases/regexp_reference_free_repetitions.rs"]
mod regexp_reference_free_repetitions;
#[path = "cases/regexp_reference_sequence_bodies.rs"]
mod regexp_reference_sequence_bodies;
#[path = "cases/regexp_repeated_assertions.rs"]
mod regexp_repeated_assertions;
#[path = "cases/regexp_repeated_branch_captures.rs"]
mod regexp_repeated_branch_captures;
#[path = "cases/regexp_repeated_continuations.rs"]
mod regexp_repeated_continuations;
#[path = "cases/regexp_repeated_literals.rs"]
mod regexp_repeated_literals;
#[path = "cases/regexp_repeated_prefixes.rs"]
mod regexp_repeated_prefixes;
#[path = "cases/regexp_repeated_reference_assertions.rs"]
mod regexp_repeated_reference_assertions;
#[path = "cases/regexp_repeated_reference_bodies.rs"]
mod regexp_repeated_reference_bodies;
#[path = "cases/regexp_repeated_reference_characters.rs"]
mod regexp_repeated_reference_characters;
#[path = "cases/regexp_repeated_sequences.rs"]
mod regexp_repeated_sequences;
#[path = "cases/regexp_repeated_unit_choices.rs"]
mod regexp_repeated_unit_choices;
#[path = "cases/regexp_replace.rs"]
mod regexp_replace;
#[path = "cases/regexp_required_empty_captured_atoms.rs"]
mod regexp_required_empty_captured_atoms;
#[path = "cases/regexp_required_empty_multi_copy_captured_atoms.rs"]
mod regexp_required_empty_multi_copy_captured_atoms;
#[path = "cases/regexp_result_initialization.rs"]
mod regexp_result_initialization;
#[path = "cases/regexp_sequence_anchors.rs"]
mod regexp_sequence_anchors;
#[path = "cases/regexp_sequence_continuations.rs"]
mod regexp_sequence_continuations;
#[path = "cases/regexp_sequence_line_assertions.rs"]
mod regexp_sequence_line_assertions;
#[path = "cases/regexp_sequence_prefixes.rs"]
mod regexp_sequence_prefixes;
#[path = "cases/regexp_sequence_word_assertions.rs"]
mod regexp_sequence_word_assertions;
#[path = "cases/regexp_sequences.rs"]
mod regexp_sequences;
#[path = "cases/regexp_set_sharing.rs"]
mod regexp_set_sharing;
#[path = "cases/regexp_split.rs"]
mod regexp_split;
#[path = "cases/regexp_test_native_boolean_results.rs"]
mod regexp_test_native_boolean_results;
#[path = "cases/regexp_undefined_counted_assertions.rs"]
mod regexp_undefined_counted_assertions;
#[path = "cases/regexp_unicode_alphabetic_properties.rs"]
mod regexp_unicode_alphabetic_properties;
#[path = "cases/regexp_unicode_arabic_script.rs"]
mod regexp_unicode_arabic_script;
#[path = "cases/regexp_unicode_ascii_hex_properties.rs"]
mod regexp_unicode_ascii_hex_properties;
#[path = "cases/regexp_unicode_assertion_sequences.rs"]
mod regexp_unicode_assertion_sequences;
#[path = "cases/regexp_unicode_assigned_properties.rs"]
mod regexp_unicode_assigned_properties;
#[path = "cases/regexp_unicode_atom_captures.rs"]
mod regexp_unicode_atom_captures;
#[path = "cases/regexp_unicode_atom_outer_assertions.rs"]
mod regexp_unicode_atom_outer_assertions;
#[path = "cases/regexp_unicode_bengali_script.rs"]
mod regexp_unicode_bengali_script;
#[path = "cases/regexp_unicode_bidi_control_properties.rs"]
mod regexp_unicode_bidi_control_properties;
#[path = "cases/regexp_unicode_bidi_mirrored_properties.rs"]
mod regexp_unicode_bidi_mirrored_properties;
#[path = "cases/regexp_unicode_binary_properties.rs"]
mod regexp_unicode_binary_properties;
#[path = "cases/regexp_unicode_bmp_character_sets.rs"]
mod regexp_unicode_bmp_character_sets;
#[path = "cases/regexp_unicode_bmp_literals.rs"]
mod regexp_unicode_bmp_literals;
#[path = "cases/regexp_unicode_braced_bmp_character_sets.rs"]
mod regexp_unicode_braced_bmp_character_sets;
#[path = "cases/regexp_unicode_braced_bmp_literals.rs"]
mod regexp_unicode_braced_bmp_literals;
#[path = "cases/regexp_unicode_captured_atoms.rs"]
mod regexp_unicode_captured_atoms;
#[path = "cases/regexp_unicode_case_ignorable_properties.rs"]
mod regexp_unicode_case_ignorable_properties;
#[path = "cases/regexp_unicode_cased_properties.rs"]
mod regexp_unicode_cased_properties;
#[path = "cases/regexp_unicode_changes_casefolded_properties.rs"]
mod regexp_unicode_changes_casefolded_properties;
#[path = "cases/regexp_unicode_changes_casemapped_properties.rs"]
mod regexp_unicode_changes_casemapped_properties;
#[path = "cases/regexp_unicode_changes_lowercased_properties.rs"]
mod regexp_unicode_changes_lowercased_properties;
#[path = "cases/regexp_unicode_changes_nfkc_casefolded_properties.rs"]
mod regexp_unicode_changes_nfkc_casefolded_properties;
#[path = "cases/regexp_unicode_changes_titlecased_properties.rs"]
mod regexp_unicode_changes_titlecased_properties;
#[path = "cases/regexp_unicode_changes_uppercased_properties.rs"]
mod regexp_unicode_changes_uppercased_properties;
#[path = "cases/regexp_unicode_character_anchors.rs"]
mod regexp_unicode_character_anchors;
#[path = "cases/regexp_unicode_character_folding.rs"]
mod regexp_unicode_character_folding;
#[path = "cases/regexp_unicode_character_word_boundaries.rs"]
mod regexp_unicode_character_word_boundaries;
#[path = "cases/regexp_unicode_class_anchors.rs"]
mod regexp_unicode_class_anchors;
#[path = "cases/regexp_unicode_class_folding.rs"]
mod regexp_unicode_class_folding;
#[path = "cases/regexp_unicode_class_properties.rs"]
mod regexp_unicode_class_properties;
#[path = "cases/regexp_unicode_class_word_boundaries.rs"]
mod regexp_unicode_class_word_boundaries;
#[path = "cases/regexp_unicode_common_script.rs"]
mod regexp_unicode_common_script;
#[path = "cases/regexp_unicode_cyrillic_script.rs"]
mod regexp_unicode_cyrillic_script;
#[path = "cases/regexp_unicode_dash_properties.rs"]
mod regexp_unicode_dash_properties;
#[path = "cases/regexp_unicode_default_ignorable_properties.rs"]
mod regexp_unicode_default_ignorable_properties;
#[path = "cases/regexp_unicode_deprecated_properties.rs"]
mod regexp_unicode_deprecated_properties;
#[path = "cases/regexp_unicode_devanagari_script.rs"]
mod regexp_unicode_devanagari_script;
#[path = "cases/regexp_unicode_diacritic_properties.rs"]
mod regexp_unicode_diacritic_properties;
#[path = "cases/regexp_unicode_dot_escapes.rs"]
mod regexp_unicode_dot_escapes;
#[path = "cases/regexp_unicode_emoji_component_properties.rs"]
mod regexp_unicode_emoji_component_properties;
#[path = "cases/regexp_unicode_emoji_modifier_base_properties.rs"]
mod regexp_unicode_emoji_modifier_base_properties;
#[path = "cases/regexp_unicode_emoji_modifier_properties.rs"]
mod regexp_unicode_emoji_modifier_properties;
#[path = "cases/regexp_unicode_emoji_presentation_properties.rs"]
mod regexp_unicode_emoji_presentation_properties;
#[path = "cases/regexp_unicode_emoji_properties.rs"]
mod regexp_unicode_emoji_properties;
#[path = "cases/regexp_unicode_empty_choice_nullable_alternatives.rs"]
mod regexp_unicode_empty_choice_nullable_alternatives;
#[path = "cases/regexp_unicode_empty_literals.rs"]
mod regexp_unicode_empty_literals;
#[path = "cases/regexp_unicode_enclosing_pure_empty_patterns.rs"]
mod regexp_unicode_enclosing_pure_empty_patterns;
#[path = "cases/regexp_unicode_extended_pictographic_properties.rs"]
mod regexp_unicode_extended_pictographic_properties;
#[path = "cases/regexp_unicode_extender_properties.rs"]
mod regexp_unicode_extender_properties;
#[path = "cases/regexp_unicode_folded_literal_anchors.rs"]
mod regexp_unicode_folded_literal_anchors;
#[path = "cases/regexp_unicode_folded_literals.rs"]
mod regexp_unicode_folded_literals;
#[path = "cases/regexp_unicode_general_category_properties.rs"]
mod regexp_unicode_general_category_properties;
#[path = "cases/regexp_unicode_grapheme_base_properties.rs"]
mod regexp_unicode_grapheme_base_properties;
#[path = "cases/regexp_unicode_grapheme_extend_properties.rs"]
mod regexp_unicode_grapheme_extend_properties;
#[path = "cases/regexp_unicode_greek_script.rs"]
mod regexp_unicode_greek_script;
#[path = "cases/regexp_unicode_gujarati_script.rs"]
mod regexp_unicode_gujarati_script;
#[path = "cases/regexp_unicode_gurmukhi_script.rs"]
mod regexp_unicode_gurmukhi_script;
#[path = "cases/regexp_unicode_han_script_properties.rs"]
mod regexp_unicode_han_script_properties;
#[path = "cases/regexp_unicode_hebrew_script.rs"]
mod regexp_unicode_hebrew_script;
#[path = "cases/regexp_unicode_hex_properties.rs"]
mod regexp_unicode_hex_properties;
#[path = "cases/regexp_unicode_id_continue_properties.rs"]
mod regexp_unicode_id_continue_properties;
#[path = "cases/regexp_unicode_id_start_properties.rs"]
mod regexp_unicode_id_start_properties;
#[path = "cases/regexp_unicode_ideographic_properties.rs"]
mod regexp_unicode_ideographic_properties;
#[path = "cases/regexp_unicode_ids_binary_operator_properties.rs"]
mod regexp_unicode_ids_binary_operator_properties;
#[path = "cases/regexp_unicode_ids_trinary_operator_properties.rs"]
mod regexp_unicode_ids_trinary_operator_properties;
#[path = "cases/regexp_unicode_inherited_script.rs"]
mod regexp_unicode_inherited_script;
#[path = "cases/regexp_unicode_join_control_properties.rs"]
mod regexp_unicode_join_control_properties;
#[path = "cases/regexp_unicode_latin_script.rs"]
mod regexp_unicode_latin_script;
#[path = "cases/regexp_unicode_line_assertions.rs"]
mod regexp_unicode_line_assertions;
#[path = "cases/regexp_unicode_literal_boundaries.rs"]
mod regexp_unicode_literal_boundaries;
#[path = "cases/regexp_unicode_literal_character_folding.rs"]
mod regexp_unicode_literal_character_folding;
#[path = "cases/regexp_unicode_literal_word_boundaries.rs"]
mod regexp_unicode_literal_word_boundaries;
#[path = "cases/regexp_unicode_logical_order_exception_properties.rs"]
mod regexp_unicode_logical_order_exception_properties;
#[path = "cases/regexp_unicode_lowercase_properties.rs"]
mod regexp_unicode_lowercase_properties;
#[path = "cases/regexp_unicode_mandatory_pure_empty_sequences.rs"]
mod regexp_unicode_mandatory_pure_empty_sequences;
#[path = "cases/regexp_unicode_math_properties.rs"]
mod regexp_unicode_math_properties;
#[path = "cases/regexp_unicode_noncapturing_branch_assertion_sequences.rs"]
mod regexp_unicode_noncapturing_branch_assertion_sequences;
#[path = "cases/regexp_unicode_noncapturing_branch_assertions.rs"]
mod regexp_unicode_noncapturing_branch_assertions;
#[path = "cases/regexp_unicode_noncapturing_branch_nested_assertion_words.rs"]
mod regexp_unicode_noncapturing_branch_nested_assertion_words;
#[path = "cases/regexp_unicode_noncapturing_branch_nested_assertions.rs"]
mod regexp_unicode_noncapturing_branch_nested_assertions;
#[path = "cases/regexp_unicode_noncapturing_branch_scoped_assertions.rs"]
mod regexp_unicode_noncapturing_branch_scoped_assertions;
#[path = "cases/regexp_unicode_noncapturing_branch_sequence_words.rs"]
mod regexp_unicode_noncapturing_branch_sequence_words;
#[path = "cases/regexp_unicode_noncapturing_empty_alternatives.rs"]
mod regexp_unicode_noncapturing_empty_alternatives;
#[path = "cases/regexp_unicode_noncapturing_empty_sequence_nullable_alternatives.rs"]
mod regexp_unicode_noncapturing_empty_sequence_nullable_alternatives;
#[path = "cases/regexp_unicode_noncharacter_properties.rs"]
mod regexp_unicode_noncharacter_properties;
#[path = "cases/regexp_unicode_nullable_alternatives.rs"]
mod regexp_unicode_nullable_alternatives;
#[path = "cases/regexp_unicode_one_iteration_nullable_alternatives.rs"]
mod regexp_unicode_one_iteration_nullable_alternatives;
#[path = "cases/regexp_unicode_one_minimum_open_nullable_alternatives.rs"]
mod regexp_unicode_one_minimum_open_nullable_alternatives;
#[path = "cases/regexp_unicode_overflowing_maximum_nullable_alternatives.rs"]
mod regexp_unicode_overflowing_maximum_nullable_alternatives;
#[path = "cases/regexp_unicode_pattern_syntax_properties.rs"]
mod regexp_unicode_pattern_syntax_properties;
#[path = "cases/regexp_unicode_pattern_white_space_properties.rs"]
mod regexp_unicode_pattern_white_space_properties;
#[path = "cases/regexp_unicode_positive_exact_nullable_alternatives.rs"]
mod regexp_unicode_positive_exact_nullable_alternatives;
#[path = "cases/regexp_unicode_positive_finite_nullable_alternatives.rs"]
mod regexp_unicode_positive_finite_nullable_alternatives;
#[path = "cases/regexp_unicode_positive_open_nullable_alternatives.rs"]
mod regexp_unicode_positive_open_nullable_alternatives;
#[path = "cases/regexp_unicode_positive_pure_empty_sequences.rs"]
mod regexp_unicode_positive_pure_empty_sequences;
#[path = "cases/regexp_unicode_property_dot.rs"]
mod regexp_unicode_property_dot;
#[path = "cases/regexp_unicode_pure_empty_disjunctions.rs"]
mod regexp_unicode_pure_empty_disjunctions;
#[path = "cases/regexp_unicode_pure_empty_patterns.rs"]
mod regexp_unicode_pure_empty_patterns;
#[path = "cases/regexp_unicode_quantified_empty_sequence_nullable_alternatives.rs"]
mod regexp_unicode_quantified_empty_sequence_nullable_alternatives;
#[path = "cases/regexp_unicode_quotation_mark_properties.rs"]
mod regexp_unicode_quotation_mark_properties;
#[path = "cases/regexp_unicode_radical_properties.rs"]
mod regexp_unicode_radical_properties;
#[path = "cases/regexp_unicode_regional_indicator_properties.rs"]
mod regexp_unicode_regional_indicator_properties;
#[path = "cases/regexp_unicode_remaining_scripts.rs"]
mod regexp_unicode_remaining_scripts;
#[path = "cases/regexp_unicode_repeated_atom_alternatives.rs"]
mod regexp_unicode_repeated_atom_alternatives;
#[path = "cases/regexp_unicode_repeated_atom_composed_word.rs"]
mod regexp_unicode_repeated_atom_composed_word;
#[path = "cases/regexp_unicode_repeated_atom_ends.rs"]
mod regexp_unicode_repeated_atom_ends;
#[path = "cases/regexp_unicode_repeated_atom_inner_line.rs"]
mod regexp_unicode_repeated_atom_inner_line;
#[path = "cases/regexp_unicode_repeated_atom_inner_word.rs"]
mod regexp_unicode_repeated_atom_inner_word;
#[path = "cases/regexp_unicode_repeated_atom_multiline_ends.rs"]
mod regexp_unicode_repeated_atom_multiline_ends;
#[path = "cases/regexp_unicode_repeated_atom_nested_assertion_groups.rs"]
mod regexp_unicode_repeated_atom_nested_assertion_groups;
#[path = "cases/regexp_unicode_repeated_atom_nested_assertion_word.rs"]
mod regexp_unicode_repeated_atom_nested_assertion_word;
#[path = "cases/regexp_unicode_repeated_atom_noncapturing_branch_line_words.rs"]
mod regexp_unicode_repeated_atom_noncapturing_branch_line_words;
#[path = "cases/regexp_unicode_repeated_atom_noncapturing_branch_lines.rs"]
mod regexp_unicode_repeated_atom_noncapturing_branch_lines;
#[path = "cases/regexp_unicode_repeated_atom_noncapturing_branch_words.rs"]
mod regexp_unicode_repeated_atom_noncapturing_branch_words;
#[path = "cases/regexp_unicode_repeated_atom_noncapturing_branches.rs"]
mod regexp_unicode_repeated_atom_noncapturing_branches;
#[path = "cases/regexp_unicode_repeated_atom_scoped_alternative_words.rs"]
mod regexp_unicode_repeated_atom_scoped_alternative_words;
#[path = "cases/regexp_unicode_repeated_atom_scoped_word.rs"]
mod regexp_unicode_repeated_atom_scoped_word;
#[path = "cases/regexp_unicode_repeated_atom_shared_alternative_assertions.rs"]
mod regexp_unicode_repeated_atom_shared_alternative_assertions;
#[path = "cases/regexp_unicode_repeated_atom_start_word.rs"]
mod regexp_unicode_repeated_atom_start_word;
#[path = "cases/regexp_unicode_repeated_atom_starts.rs"]
mod regexp_unicode_repeated_atom_starts;
#[path = "cases/regexp_unicode_repeated_atom_word_alternatives.rs"]
mod regexp_unicode_repeated_atom_word_alternatives;
#[path = "cases/regexp_unicode_repeated_atom_word_boundaries.rs"]
mod regexp_unicode_repeated_atom_word_boundaries;
#[path = "cases/regexp_unicode_repeated_atoms.rs"]
mod regexp_unicode_repeated_atoms;
#[path = "cases/regexp_unicode_scalar_classes.rs"]
mod regexp_unicode_scalar_classes;
#[path = "cases/regexp_unicode_scalar_literals.rs"]
mod regexp_unicode_scalar_literals;
#[path = "cases/regexp_unicode_script_extensions.rs"]
mod regexp_unicode_script_extensions;
#[path = "cases/regexp_unicode_sentence_terminal_properties.rs"]
mod regexp_unicode_sentence_terminal_properties;
#[path = "cases/regexp_unicode_sets_bmp_character_sets.rs"]
mod regexp_unicode_sets_bmp_character_sets;
#[path = "cases/regexp_unicode_sets_bmp_literals.rs"]
mod regexp_unicode_sets_bmp_literals;
#[path = "cases/regexp_unicode_soft_dotted_properties.rs"]
mod regexp_unicode_soft_dotted_properties;
#[path = "cases/regexp_unicode_surrogate_literals.rs"]
mod regexp_unicode_surrogate_literals;
#[path = "cases/regexp_unicode_terminal_punctuation_properties.rs"]
mod regexp_unicode_terminal_punctuation_properties;
#[path = "cases/regexp_unicode_unified_ideograph_properties.rs"]
mod regexp_unicode_unified_ideograph_properties;
#[path = "cases/regexp_unicode_unrepresentable_exact_nullable_alternatives.rs"]
mod regexp_unicode_unrepresentable_exact_nullable_alternatives;
#[path = "cases/regexp_unicode_unrepresentable_finite_nullable_alternatives.rs"]
mod regexp_unicode_unrepresentable_finite_nullable_alternatives;
#[path = "cases/regexp_unicode_uppercase_properties.rs"]
mod regexp_unicode_uppercase_properties;
#[path = "cases/regexp_unicode_v_sets.rs"]
mod regexp_unicode_v_sets;
#[path = "cases/regexp_unicode_variation_selector_properties.rs"]
mod regexp_unicode_variation_selector_properties;
#[path = "cases/regexp_unicode_whitespace_properties.rs"]
mod regexp_unicode_whitespace_properties;
#[path = "cases/regexp_unicode_word_asserted_whole_captures.rs"]
mod regexp_unicode_word_asserted_whole_captures;
#[path = "cases/regexp_unicode_word_assertions.rs"]
mod regexp_unicode_word_assertions;
#[path = "cases/regexp_unicode_xid_continue_properties.rs"]
mod regexp_unicode_xid_continue_properties;
#[path = "cases/regexp_unicode_xid_start_properties.rs"]
mod regexp_unicode_xid_start_properties;
#[path = "cases/regexp_unicode_zero_minimum_nullable_alternatives.rs"]
mod regexp_unicode_zero_minimum_nullable_alternatives;
#[path = "cases/regexp_unit_branch_wrappers.rs"]
mod regexp_unit_branch_wrappers;
#[path = "cases/regexp_unit_choice_captures.rs"]
mod regexp_unit_choice_captures;
#[path = "cases/regexp_unit_matching_direction.rs"]
mod regexp_unit_matching_direction;
#[path = "cases/regexp_unrepresentable_outside_lookbehind.rs"]
mod regexp_unrepresentable_outside_lookbehind;
#[path = "cases/regexp_word_boundaries.rs"]
mod regexp_word_boundaries;
#[path = "cases/regexp_wrapped_quantified_references.rs"]
mod regexp_wrapped_quantified_references;
#[path = "cases/regexp_zero_count_lookbehind_choices.rs"]
mod regexp_zero_count_lookbehind_choices;
#[path = "cases/regexp_zero_count_lookbehind_references.rs"]
mod regexp_zero_count_lookbehind_references;
#[path = "cases/regexp_zero_count_native_scopes.rs"]
mod regexp_zero_count_native_scopes;
#[path = "cases/regexp_zero_reference_lookbehind_counts.rs"]
mod regexp_zero_reference_lookbehind_counts;
#[path = "cases/regexp_zero_width_choices.rs"]
mod regexp_zero_width_choices;
#[path = "cases/regexp_zero_width_lookahead_branches.rs"]
mod regexp_zero_width_lookahead_branches;
#[path = "cases/regexp_zero_width_lookahead_repetitions.rs"]
mod regexp_zero_width_lookahead_repetitions;
#[path = "cases/regexp_zero_width_lookbehind_counts.rs"]
mod regexp_zero_width_lookbehind_counts;
#[path = "cases/rest_parameters.rs"]
mod rest_parameters;
#[path = "cases/roots.rs"]
mod roots;
#[path = "cases/set.rs"]
mod set;
#[path = "cases/stack_limits.rs"]
mod stack_limits;
#[path = "cases/strict_functions.rs"]
mod strict_functions;
#[path = "cases/string_case.rs"]
mod string_case;
#[path = "cases/string_characters.rs"]
mod string_characters;
#[path = "cases/string_locale_compare.rs"]
mod string_locale_compare;
#[path = "cases/string_matching.rs"]
mod string_matching;
#[path = "cases/string_normalize.rs"]
mod string_normalize;
#[path = "cases/string_predicates.rs"]
mod string_predicates;
#[path = "cases/string_raw.rs"]
mod string_raw;
#[path = "cases/string_repeat.rs"]
mod string_repeat;
#[path = "cases/string_replace.rs"]
mod string_replace;
#[path = "cases/string_replace_all.rs"]
mod string_replace_all;
#[path = "cases/string_search.rs"]
mod string_search;
#[path = "cases/string_sequences.rs"]
mod string_sequences;
#[path = "cases/string_split.rs"]
mod string_split;
#[path = "cases/string_trim.rs"]
mod string_trim;
#[path = "cases/string_well_formed.rs"]
mod string_well_formed;
#[path = "cases/string_wrappers.rs"]
mod string_wrappers;
#[path = "cases/super_properties.rs"]
mod super_properties;
#[path = "cases/switch.rs"]
mod switch;
#[path = "cases/symbols.rs"]
mod symbols;
#[path = "cases/tagged_templates.rs"]
mod tagged_templates;
#[path = "cases/templates.rs"]
mod templates;
#[path = "cases/try_finally.rs"]
mod try_finally;
#[path = "cases/updates.rs"]
mod updates;
#[path = "cases/uri_decoding.rs"]
mod uri_decoding;
#[path = "cases/uri_encoding.rs"]
mod uri_encoding;
#[path = "cases/var.rs"]
mod var;
#[path = "cases/var_patterns.rs"]
mod var_patterns;
#[path = "cases/weak_set.rs"]
mod weak_set;
#[path = "cases/with.rs"]
mod with;
