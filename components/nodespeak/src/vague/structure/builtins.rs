use crate::high_level::problem::FilePosition;
use crate::vague::structure::{
    DataType, KnownData, MacroData, Program, Statement, UnaryOperator, VCExpression, VPExpression,
    Variable,
};

fn add_data_type(program: &mut Program, name: &str, dtype: DataType) {
    let scope = program.get_builtins_scope();
    let var = Variable::data_type(FilePosition::placeholder(), dtype);
    let var_id = program.adopt_and_define_symbol(scope, name, var);
    let data_type_literal = Box::new(VPExpression::Literal(
        KnownData::DataType(DataType::DataType),
        FilePosition::placeholder(),
    ));
    program[scope].add_statement(Statement::CreationPoint {
        var: var_id,
        var_type: data_type_literal,
        position: FilePosition::placeholder(),
    });
}

fn add_constant(program: &mut Program, name: &str, data: KnownData) {
    let scope = program.get_builtins_scope();
    let typ = data.get_data_type();
    let var = Variable::constant(FilePosition::placeholder(), data);
    let var_id = program.adopt_and_define_symbol(scope, name, var);
    let data_type_literal = Box::new(VPExpression::Literal(
        KnownData::DataType(typ),
        FilePosition::placeholder(),
    ));
    program[scope].add_statement(Statement::CreationPoint {
        var: var_id,
        var_type: data_type_literal,
        position: FilePosition::placeholder(),
    });
}

fn add_unary_op_macro(
    program: &mut Program,
    operator: UnaryOperator,
    name: &str,
    in_name: &str,
    out_name: &str,
) {
    let root = program.get_builtins_scope();
    let body = program.create_child_scope(root);

    let in_var = Variable::variable(FilePosition::placeholder(), None);
    let in_var_id = program.adopt_and_define_symbol(body, in_name, in_var);
    program[body].add_input(in_var_id);
    let out_var = Variable::variable(FilePosition::placeholder(), None);
    let out_var_id = program.adopt_and_define_symbol(body, out_name, out_var);
    program[body].add_output(out_var_id);
    let out_type = KnownData::DataType(DataType::Automatic);
    let out_type = VPExpression::Literal(out_type, FilePosition::placeholder());
    program[body].add_statement(Statement::CreationPoint {
        var: out_var_id,
        var_type: Box::new(out_type),
        position: FilePosition::placeholder(),
    });

    let p = FilePosition::placeholder();
    program[body].add_statement(Statement::Assign {
        target: Box::new(VCExpression::Variable(out_var_id, p.clone())),
        value: Box::new(VPExpression::UnaryOperation(
            operator,
            Box::new(VPExpression::Variable(in_var_id, p.clone())),
            p.clone(),
        )),
        position: p.clone(),
    });

    let var = Variable::macro_def(MacroData::new(body, p.clone()));
    let var_id = program.adopt_and_define_symbol(root, name, var);
    let data_type_literal = Box::new(VPExpression::Literal(
        KnownData::DataType(DataType::Macro),
        FilePosition::placeholder(),
    ));
    program[root].add_statement(Statement::CreationPoint {
        var: var_id,
        var_type: data_type_literal,
        position: FilePosition::placeholder(),
    });
}

// Adds built-in methods to the root scope.
pub fn add_builtins(program: &mut Program) {
    add_data_type(program, "AUTO", DataType::Automatic);
    add_data_type(program, "BOOL", DataType::Bool);
    add_data_type(program, "INT", DataType::Int);
    add_data_type(program, "FLOAT", DataType::Float);
    add_data_type(program, "DATA_TYPE", DataType::DataType);
    add_data_type(program, "MACRO", DataType::Macro);

    add_constant(program, "PI", KnownData::Float(std::f64::consts::PI));
    add_constant(program, "TAU", KnownData::Float(std::f64::consts::PI * 2.0));
    add_constant(program, "E", KnownData::Float(std::f64::consts::E));
    add_constant(program, "TRUE", KnownData::Bool(true));
    add_constant(program, "FALSE", KnownData::Bool(false));

    add_unary_op_macro(program, UnaryOperator::Ftoi, "Ftoi", "float", "int");
    add_unary_op_macro(program, UnaryOperator::Itof, "Itof", "int", "float");
    add_unary_op_macro(program, UnaryOperator::Sine, "Sin", "radians", "ratio");
    add_unary_op_macro(program, UnaryOperator::Cosine, "Cos", "radians", "ratio");
    add_unary_op_macro(
        program,
        UnaryOperator::SquareRoot,
        "Sqrt",
        "value",
        "result",
    );
    add_unary_op_macro(program, UnaryOperator::Exp, "Exp", "power", "result");
    add_unary_op_macro(program, UnaryOperator::Exp2, "Exp2", "power", "result");
    add_unary_op_macro(program, UnaryOperator::Log, "Log", "value", "power");
    add_unary_op_macro(program, UnaryOperator::Log10, "Log10", "value", "power");
    add_unary_op_macro(program, UnaryOperator::Log2, "Log2", "value", "power");
    add_unary_op_macro(program, UnaryOperator::Absolute, "Abs", "value", "result");
    add_unary_op_macro(program, UnaryOperator::Floor, "Floor", "value", "result");
    add_unary_op_macro(program, UnaryOperator::Ceiling, "Ceil", "value", "result");
    add_unary_op_macro(program, UnaryOperator::Truncate, "Trunc", "value", "result");
}
