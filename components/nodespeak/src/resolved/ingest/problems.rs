use crate::high_level::problem::*;
use crate::vague::structure as i;
use ProblemType::Error;
use ProblemType::Hint;

pub fn wrong_number_of_inputs(
    macro_call_pos: FilePosition,
    header_pos: FilePosition,
    provided: usize,
    expected: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            macro_call_pos,
            Error,
            &format!(
                concat!(
                    "Wrong Number Of Inputs\nThis macro call has {} input ",
                    "arguments but the macro it is calling has {} input ",
                    "parameters.",
                ),
                provided, expected,
            ),
        ),
        ProblemDescriptor::new(
            header_pos,
            Hint,
            concat!("The header of the macro being called is as follows:"),
        ),
    ])
}

pub fn wrong_number_of_outputs(
    macro_call_pos: FilePosition,
    header_pos: FilePosition,
    provided: usize,
    expected: usize,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            macro_call_pos,
            Error,
            &format!(
                concat!(
                    "Wrong Number Of Outputs\nThis macro call has {} output ",
                    "arguments but the macro it is calling has {} output ",
                    "parameters.",
                ),
                provided, expected,
            ),
        ),
        ProblemDescriptor::new(
            header_pos,
            Hint,
            concat!("The header of the macro being called is as follows:"),
        ),
    ])
}

pub fn not_macro(expr_pos: FilePosition, typ: &i::DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        expr_pos,
        Error,
        &format!(
            concat!(
                "Incorrect Type\nThe highlighted expression should resolve to a macro because it ",
                "is being used in a macro call. However, it resolves to a {:?} instead.",
            ),
            typ
        ),
    )])
}

pub fn not_data_type(expr_pos: FilePosition, typ: &i::DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        expr_pos,
        Error,
        &format!(
            concat!(
                "Incorrect Type\nThe highlighted expression should resolve to a data type because ",
                "it is being used to declare a variable. However, it resolves to a {:?} instead.",
            ),
            typ
        ),
    )])
}

pub fn guaranteed_assert(assert_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        assert_pos,
        Error,
        "Assert Guranteed To Fail",
    )])
}

pub fn array_index_not_int(
    index: FilePosition,
    index_type: &i::DataType,
    expression: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            index,
            Error,
            &format!(
                "Array Index Not Int\nExpected an integer, got a {:?}:",
                index_type
            ),
        ),
        ProblemDescriptor::new(
            expression,
            Hint,
            "Encountered while resolving this expression:",
        ),
    ])
}

pub fn array_index_less_than_zero(
    index: FilePosition,
    value: i64,
    expression: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            index,
            Error,
            &format!(
                concat!(
                    "Array Index Less Than Zero\nThe value of the highlighted expression was ",
                    "computed to be {}:",
                ),
                value
            ),
        ),
        ProblemDescriptor::new(
            expression,
            Hint,
            "Encountered while resolving this expression:",
        ),
    ])
}

pub fn array_index_too_big(
    index: FilePosition,
    value: usize,
    arr_size: usize,
    expression: FilePosition,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            index,
            Error,
            &format!(
                concat!(
                    "Array Index Too Big\nThe value of the highlighted expression was ",
                    "computed to be {}, which is too big when indexing an array of size {}:",
                ),
                value, arr_size,
            ),
        ),
        ProblemDescriptor::new(
            expression,
            Hint,
            "Encountered while resolving this expression:",
        ),
    ])
}

pub fn array_base_not_data_type(base: FilePosition, typ: &i::DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        base,
        Error,
        &format!(
            concat!(
                "Array Base Not Data Type\nExpected a data type (as a base for an array type), ",
                "got a {:?} instead."
            ),
            typ,
        ),
    )])
}

pub fn array_size_less_than_one(size: FilePosition, value: i64) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size,
        Error,
        &format!(
            concat!("Array Size Less Than One\nThe highlighted expression resolves to {}. ",),
            value
        ),
    )])
}

pub fn array_size_not_int(size: FilePosition, size_type: &i::SpecificDataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size,
        Error,
        &format!(
            "Array Size Not Int\nExpected an integer, got a {:?}:",
            size_type
        ),
    )])
}

pub fn array_size_not_resolved(size: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        size,
        Error,
        concat!(
            "Dynamic Array Size\nArray sizes must be specified at compile time. The following ",
            "expression can only be evaluated at runtime:"
        ),
    )])
}

pub fn bad_array_literal(
    bad_item_pos: FilePosition,
    bad_item_type: &i::DataType,
    first_item_pos: FilePosition,
    first_item_type: &i::DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            bad_item_pos,
            Error,
            &format!(
                "Bad Array Literal\nThe highlighted item has an unexpected data type of {:?}.",
                bad_item_type,
            ),
        ),
        ProblemDescriptor::new(
            first_item_pos,
            Hint,
            &format!(
                "The first item in the array literal is of data type {:?}:",
                first_item_type
            ),
        ),
    ])
}

pub fn no_bct_binop(
    expression: FilePosition,
    op1: FilePosition,
    op1_type: &i::DataType,
    op2: FilePosition,
    op2_type: &i::DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            expression,
            Error,
            concat!(
                "No Biggest Common Type\nCannot determine what data type the result of the ",
                "highlighted expression will have:"
            ),
        ),
        ProblemDescriptor::new(
            op1,
            Hint,
            &format!("The first operand has data type {:?}:", op1_type),
        ),
        ProblemDescriptor::new(
            op2,
            Hint,
            &format!("But the second operand has data type {:?}:", op2_type),
        ),
    ])
}

pub fn cannot_inflate(
    expression: FilePosition,
    expr_type: &i::DataType,
    as_type: &i::SpecificDataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        expression,
        Error,
        &format!(
            "Cannot Inflate\nCannot inflate a value of type {:?} to type {:?}:",
            expr_type, as_type
        ),
    )])
}

pub fn as_type_bound(type_bound: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        type_bound,
        Error,
        "Unresolved type bounds cannot be used in an 'as' expression.",
    )])
}

pub fn mismatched_assign(
    expression: FilePosition,
    lhs: FilePosition,
    lhs_type: &i::DataType,
    rhs: FilePosition,
    rhs_type: &i::DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            expression,
            Error,
            concat!(
                "Mismatched Datatype In Assignment\nCannot figure out how to assign the ",
                "right hand side of this statement to the left hand side:"
            ),
        ),
        ProblemDescriptor::new(
            rhs,
            Hint,
            &format!("The right hand side has data type {:?}:", rhs_type),
        ),
        ProblemDescriptor::new(
            lhs,
            Hint,
            &format!("But the left hand side has data type {:?}:", lhs_type),
        ),
    ])
}

pub fn value_not_run_time_compatible(
    value_pos: FilePosition,
    dtype: &i::DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        value_pos,
        Error,
        &format!(
            concat!(
                "Value Not Run Time Compatible\nThe value of the highlighted expression was ",
                "calculated at compile time, but the way it is used requires it to be available ",
                "at run time. This is not possible as it yields a value of type {:?}."
            ),
            dtype
        ),
    )])
}

pub fn rt_indexes_on_ct_variable(expr_pos: FilePosition, dtype: &i::DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        expr_pos,
        Error,
        &format!(
            concat!(
                "Runtime Index On Compiletime Variable\nThe highlighted expression has indexes ",
                "which will only be known at run time. However, it refers to a value of type ",
                "{:?}, which can only be used at compile time."
            ),
            dtype
        ),
    )])
}

pub fn too_many_indexes(
    expr_pos: FilePosition,
    num_indexes: usize,
    max_indexes: usize,
    base_pos: FilePosition,
    base_type: &i::DataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            expr_pos,
            Error,
            &format!(
                concat!(
                    "Too Many Indexes\nThe highlighted expression is indexing a value {} times, ",
                    "but the value it is indexing can only be indexed at most {} times."
                ),
                num_indexes, max_indexes
            ),
        ),
        ProblemDescriptor::new(
            base_pos,
            Hint,
            &format!(
                concat!("The base of the expression has the data type {:?}"),
                base_type
            ),
        ),
    ])
}

pub fn cannot_index(
    expr_pos: FilePosition,
    specific_index: FilePosition,
    smallest_type: &i::SpecificDataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            specific_index,
            Error,
            &format!(
                concat!(
                    "Cannot Index\nThe highlighted expression is indexing a value which may be ",
                    "as small as {:?} according to its type bound. Try making it optional by ",
                    "adding ? to the end."
                ),
                smallest_type
            ),
        ),
        ProblemDescriptor::new(expr_pos, Hint, "Encountered while indexing this value:"),
    ])
}

pub fn vpe_wrong_type(
    vpe_pos: FilePosition,
    expected: &i::SpecificDataType,
    found: &i::SpecificDataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        vpe_pos,
        Error,
        &format!(
            "Wrong Data Type\nExpected a {:?}, found a {:?}.",
            expected, found
        ),
    )])
}

pub fn bad_type_for_operator(
    operand_pos: FilePosition,
    operator_name: &str,
    operator_usage: &str,
    found: &i::SpecificDataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        operand_pos,
        Error,
        &format!(
            concat!(
                "Bad Type For Operator\nThe {} operator requires {}, but a value of type {:?} was ",
                "found instead."
            ),
            operator_name, operator_usage, found
        ),
    )])
}

pub fn unresolved_bounded_var(var_pos: FilePosition) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        var_pos,
        Error,
        concat!(
            "Unresolved Bounded Var\nThe highlighted variable was declared with a bounded type. ",
            "It has not been assigned any value before this point so its actual data type cannot ",
            "be determined. Assigning the variable a value somewhere earlier in the program will ",
            "fix this error."
        ),
    )])
}

pub fn dangling_value(bad_expr_pos: FilePosition, typ: &i::DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        bad_expr_pos,
        Error,
        &format!(
            concat!(
                "Dangling Value\nThe highlighted expression yields a value of type {:?}, but it ",
                "is not stored in any variable."
            ),
            typ
        ),
    )])
}

pub fn compile_time_input(input_decl_pos: FilePosition, typ: &i::DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        input_decl_pos,
        Error,
        &format!(
            concat!(
                "Compile Time Input\nThe highlighted input was given the data type {:?}, which ",
                "can only be used at compile time."
            ),
            typ
        ),
    )])
}

pub fn compile_time_output(output_decl_pos: FilePosition, typ: &i::DataType) -> CompileProblem {
    CompileProblem::from_descriptors(vec![ProblemDescriptor::new(
        output_decl_pos,
        Error,
        &format!(
            concat!(
                "Compile Time Output\nThe highlighted output was given the data type {:?}, which ",
                "can only be used at compile time."
            ),
            typ
        ),
    )])
}

pub fn value_too_big(
    value_pos: FilePosition,
    assign_to_pos: FilePosition,
    bigger_type: &i::SpecificDataType,
    upper_bound: &i::SpecificDataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            assign_to_pos,
            Error,
            &format!(
                concat!(
                    "Upper Bound Violation\nThe highlighted code can only accept a value with a ",
                    "data type that can inflate to {:?}."
                ),
                upper_bound,
            ),
        ),
        ProblemDescriptor::new(
            value_pos,
            Hint,
            &format!(
                "The upper bound of the highlighted value's type is {:?}.",
                bigger_type
            ),
        ),
    ])
}

pub fn value_too_small(
    value_pos: FilePosition,
    assign_to_pos: FilePosition,
    smaller_type: &i::SpecificDataType,
    lower_bound: &i::SpecificDataType,
) -> CompileProblem {
    CompileProblem::from_descriptors(vec![
        ProblemDescriptor::new(
            assign_to_pos,
            Error,
            &format!(
                concat!(
                    "Lower Bound Violation\nThe highlighted code can only accept a value with a ",
                    "data type that {:?} can be inflated to."
                ),
                lower_bound,
            ),
        ),
        ProblemDescriptor::new(
            value_pos,
            Hint,
            &format!(
                "The lower bound of the highlighted value's type is {:?}.",
                smaller_type
            ),
        ),
    ])
}
