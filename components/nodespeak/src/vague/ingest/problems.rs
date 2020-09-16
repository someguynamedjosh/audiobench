use crate::high_level::problem::*;
use ProblemType::Error;
use ProblemType::Hint;

pub fn no_entity_with_name(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Invalid Entity Name\nThere is no macro, variable, or data type visible in this ",
            "scope with the specified name.",
        ),
    )])
}

pub fn bad_property_name(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        "Bad Property Name\nThere is no property with the specified name.",
    )])
}

pub fn return_from_root(pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        concat!(
            "Return Outside Macro\nReturn statements can only be used inside of macro ",
            "definitions. The code snippet below was understood to be a part of the root scope ",
            "of the file.",
        ),
    )])
}

pub fn missing_output_definition(
    pos: FilePosition,
    macro_name: &str,
    output_name: &str,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        &format!(
            concat!(
                "Missing Output Definition\nThe macro named {} defines an output named {} but ",
                "no such variable exists within the body of the macro. Define a variable named ",
                "{} inside the macro to fix this error."
            ),
            macro_name, output_name, output_name,
        ),
    )])
}

pub fn missing_export_definition(pos: FilePosition, exported_var_name: &str) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        pos,
        Error,
        &format!(
            concat!(
                "Missing Exported Variable Definition\nA static block exports a variable named ",
                "{}, but there is no variable declared with that name inside the static block."
            ),
            exported_var_name,
        ),
    )])
}

pub fn too_many_inline_returns(
    macro_call_pos: FilePosition,
    output_list_pos: FilePosition,
    num_inline_returns: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            macro_call_pos,
            Error,
            &format!(
                concat!(
                    "Too Many Inline Returns\nThis list of macro outputs uses the inline ",
                    "keyword {} times, but it should only be used once."
                ),
                num_inline_returns
            ),
        ),
        ProblemDescriptor::new(
            output_list_pos,
            Hint,
            concat!("Encountered while parsing this macro call."),
        ),
    ])
}

pub fn missing_inline_return(
    macro_call_pos: FilePosition,
    output_list_pos: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            macro_call_pos,
            Error,
            concat!(
                "Missing Inline Return\nThe position of the highlighted macro call requires it ",
                "to have an inline output so that the output can be used in an expression or ",
                "statement. However, there is no inline output argument in the output list.",
            ),
        ),
        ProblemDescriptor::new(
            output_list_pos,
            Hint,
            concat!(
                "Try replacing one of the output arguments with the keyword 'inline'. If the ",
                "macro being called only has one output, you can also delete the output list ",
                "entirely to automatically make the only output inline."
            ),
        ),
    ])
}

pub fn io_inside_macro(declaration_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        declaration_pos,
        Error,
        concat!(
            "I/O Inside Macro\nInput and output variables can only be declared in the root scope.",
        ),
    )])
}

pub fn write_to_read_only_variable(var_pos: FilePosition, var_name: &str) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        var_pos,
        Error,
        &format!(
            concat!(
                "Write To Read-Only Variable\nThe variable {} is read-only, so it cannot be ",
                "assigned a different value."
            ),
            var_name
        ),
    )])
}

pub fn nonexistant_include(include_pos: FilePosition, file_name: &str) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        include_pos,
        Error,
        &format!(
            concat!(
                "Nonexistant Include\nCould not find a file named \"{}\". Check the spelling and ",
                "ensure that the compiler knows that the file exists through Compiler::add_source ",
                "or another similarly named method.",
            ),
            file_name
        ),
    )])
}

pub fn hint_encountered_while_including(
    existing_error: &mut CompileProblem,
    include_pos: FilePosition,
) {
    existing_error.add_descriptor(ProblemDescriptor::new(
        include_pos,
        Hint,
        "Encountered while including this file.",
    ));
}
