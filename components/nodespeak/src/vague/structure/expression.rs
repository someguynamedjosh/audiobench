use crate::high_level::problem::FilePosition;
use crate::vague::structure::{KnownData, ScopeId, VariableId};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub enum Property {
    Type,
    Dims,
}

impl Property {
    pub fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "TYPE" => Self::Type,
            "DIMS" => Self::Dims,
            _ => return Err(()),
        })
    }
}

impl Debug for Property {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Type => write!(formatter, "TYPE"),
            Self::Dims => write!(formatter, "DIMS"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
    BNot,
    Reciprocal,

    Sine,
    Cosine,
    SquareRoot,
    Exp,
    Exp2,
    Log,
    Log10,
    Log2,
    Absolute,
    Floor,
    Ceiling,
    Truncate,

    Ftoi,
    Itof,

    PropertyAccess(Property),
}

impl Debug for UnaryOperator {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            UnaryOperator::Negate => write!(formatter, "-"),
            UnaryOperator::Not => write!(formatter, "not"),
            UnaryOperator::BNot => write!(formatter, "bnot"),
            UnaryOperator::Reciprocal => write!(formatter, "reciprocal of"),
            UnaryOperator::Sine => write!(formatter, "sine of"),
            UnaryOperator::Cosine => write!(formatter, "cosine of"),
            UnaryOperator::SquareRoot => write!(formatter, "square root of"),
            UnaryOperator::Exp => write!(formatter, "e to the power of"),
            UnaryOperator::Exp2 => write!(formatter, "2 to the power of"),
            UnaryOperator::Log => write!(formatter, "log base e of"),
            UnaryOperator::Log2 => write!(formatter, "log base 2 of"),
            UnaryOperator::Log10 => write!(formatter, "log base 10 of"),
            UnaryOperator::Absolute => write!(formatter, "abs"),
            UnaryOperator::Floor => write!(formatter, "floor"),
            UnaryOperator::Ceiling => write!(formatter, "ceil"),
            UnaryOperator::Truncate => write!(formatter, "truncate"),
            UnaryOperator::Ftoi => write!(formatter, "float to int"),
            UnaryOperator::Itof => write!(formatter, "int to float"),
            UnaryOperator::PropertyAccess(property) => write!(formatter, ":{:?}", property),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    As,
    In,

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    And,
    Or,
    Xor,
    BAnd,
    BOr,
    BXor,
    LeftShift,
    RightShift,

    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

impl Debug for BinaryOperator {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            BinaryOperator::As => write!(formatter, "as"),
            BinaryOperator::In => write!(formatter, "in"),

            BinaryOperator::Add => write!(formatter, "+"),
            BinaryOperator::Subtract => write!(formatter, "-"),
            BinaryOperator::Multiply => write!(formatter, "*"),
            BinaryOperator::Divide => write!(formatter, "/"),
            BinaryOperator::Modulo => write!(formatter, "%"),
            BinaryOperator::Power => write!(formatter, "**"),

            BinaryOperator::And => write!(formatter, "and"),
            BinaryOperator::Or => write!(formatter, "or"),
            BinaryOperator::Xor => write!(formatter, "xor"),
            BinaryOperator::BAnd => write!(formatter, "band"),
            BinaryOperator::BOr => write!(formatter, "bor"),
            BinaryOperator::BXor => write!(formatter, "bxor"),
            BinaryOperator::LeftShift => write!(formatter, "<<"),
            BinaryOperator::RightShift => write!(formatter, ">>"),

            BinaryOperator::Equal => write!(formatter, "=="),
            BinaryOperator::NotEqual => write!(formatter, "!="),
            BinaryOperator::LessThan => write!(formatter, "<"),
            BinaryOperator::GreaterThan => write!(formatter, ">"),
            BinaryOperator::LessThanOrEqual => write!(formatter, "<="),
            BinaryOperator::GreaterThanOrEqual => write!(formatter, ">="),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum VPExpression {
    Literal(KnownData, FilePosition),
    Variable(VariableId, FilePosition),
    Collect(Vec<VPExpression>, FilePosition),
    UnboundedTypeBound(FilePosition),
    TypeBound {
        lower: Option<Box<VPExpression>>,
        upper: Box<VPExpression>,
        position: FilePosition,
    },
    BuildArrayType {
        dimensions: Vec<VPExpression>,
        base: Box<VPExpression>,
        position: FilePosition,
    },

    UnaryOperation(UnaryOperator, Box<VPExpression>, FilePosition),
    BinaryOperation(
        Box<VPExpression>,
        BinaryOperator,
        Box<VPExpression>,
        FilePosition,
    ),
    Index {
        base: Box<VPExpression>,
        indexes: Vec<(VPExpression, bool)>,
        position: FilePosition,
    },
    MacroCall {
        mcro: Box<VPExpression>,
        inputs: Vec<VPExpression>,
        outputs: Vec<FuncCallOutput>,
        position: FilePosition,
    },
}

impl Debug for VPExpression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal(value, ..) => write!(formatter, "{:?}", value),
            Self::Variable(id, ..) => write!(formatter, "{:?}", id),
            Self::Collect(values, ..) => {
                write!(formatter, "[")?;
                for value in values.iter() {
                    write!(formatter, "{:?}, ", value)?;
                }
                write!(formatter, "]")
            }
            Self::UnboundedTypeBound(..) => write!(formatter, "<>"),
            Self::TypeBound { lower, upper, .. } => {
                write!(formatter, "<")?;
                if let Some(lower) = lower {
                    write!(formatter, "{:?}, ", lower)?;
                }
                write!(formatter, "{:?}>", upper)
            }
            Self::BuildArrayType {
                dimensions, base, ..
            } => {
                for dimension in dimensions.iter() {
                    write!(formatter, "[{:?}]", dimension)?;
                }
                write!(formatter, "{:?}", base)
            }

            Self::UnaryOperation(operator, value, ..) => {
                write!(formatter, "({:?} {:?})", operator, value)
            }
            Self::BinaryOperation(v1, operator, v2, ..) => {
                write!(formatter, "({:?} {:?} {:?})", v1, operator, v2)
            }
            Self::Index { base, indexes, .. } => {
                write!(formatter, "{:?}", base)?;
                for (index, optional) in indexes.iter() {
                    write!(
                        formatter,
                        "[{:?}{}]",
                        index,
                        if *optional { "?" } else { "" }
                    )?;
                }
                write!(formatter, "")
            }

            Self::MacroCall {
                mcro,
                inputs,
                outputs,
                ..
            } => {
                write!(formatter, "call {:?}(", mcro)?;
                for expr in inputs {
                    write!(formatter, "{:?},", expr)?;
                }
                write!(formatter, "):(")?;
                for expr in outputs {
                    write!(formatter, "{:?},", expr)?;
                }
                write!(formatter, ")")
            }
        }
    }
}

impl VPExpression {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::Literal(_, position)
            | Self::Variable(_, position)
            | Self::Collect(_, position)
            | Self::UnboundedTypeBound(position)
            | Self::TypeBound { position, .. }
            | Self::BuildArrayType { position, .. }
            | Self::UnaryOperation(_, _, position)
            | Self::BinaryOperation(_, _, _, position)
            | Self::Index { position, .. }
            | Self::MacroCall { position, .. } => position.clone(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum VCExpression {
    Variable(VariableId, FilePosition),
    Index {
        base: Box<VCExpression>,
        indexes: Vec<(VPExpression, bool)>,
        position: FilePosition,
    },
}

impl Debug for VCExpression {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Variable(id, ..) => write!(formatter, "var {:?}", id),
            Self::Index { base, indexes, .. } => {
                write!(formatter, "({:?})", base)?;
                for (index, optional) in indexes {
                    write!(
                        formatter,
                        "[{:?}{}]",
                        index,
                        if *optional { "?" } else { "" }
                    )?;
                }
                write!(formatter, "")
            }
        }
    }
}

impl VCExpression {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::Variable(_, position) | Self::Index { position, .. } => position.clone(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum FuncCallOutput {
    InlineReturn(FilePosition),
    VCExpression(Box<VCExpression>),
}

impl Debug for FuncCallOutput {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::InlineReturn(..) => write!(formatter, "inline"),
            Self::VCExpression(expr) => write!(formatter, "{:?}", expr),
        }
    }
}

impl FuncCallOutput {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::InlineReturn(pos) => pos.clone(),
            Self::VCExpression(expr) => expr.clone_position(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Statement {
    CreationPoint {
        var: VariableId,
        var_type: Box<VPExpression>,
        position: FilePosition,
    },
    Assert(Box<VPExpression>, FilePosition),
    Return(FilePosition),
    Assign {
        target: Box<VCExpression>,
        value: Box<VPExpression>,
        position: FilePosition,
    },
    Branch {
        clauses: Vec<(VPExpression, ScopeId)>,
        else_clause: Option<ScopeId>,
        position: FilePosition,
    },
    ForLoop {
        allow_unroll: bool,
        counter: VariableId,
        start: Box<VPExpression>,
        end: Box<VPExpression>,
        body: ScopeId,
        position: FilePosition,
    },
    StaticInit {
        body: ScopeId,
        exports: Vec<VariableId>,
        position: FilePosition,
    },
    RawVPExpression(Box<VPExpression>),
}

impl Debug for Statement {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::CreationPoint { var, var_type, .. } => {
                write!(formatter, "define {:?} {:?}", var_type, var)
            }
            Self::Assert(value, ..) => write!(formatter, "assert {:?};", value),
            Self::Return(..) => write!(formatter, "return;"),
            Self::Assign { target, value, .. } => write!(formatter, "{:?} = {:?};", target, value),
            Self::Branch {
                clauses,
                else_clause,
                ..
            } => {
                assert!(clauses.len() > 0);
                write!(formatter, "if {:?} {{ {:?} }}", &clauses[0].0, clauses[0].1)?;
                for (condition, scope) in &clauses[1..] {
                    write!(formatter, " else if {:?} {{ {:?} }}", condition, scope)?;
                }
                if let Some(scope) = else_clause {
                    write!(formatter, " else {{ {:?} }}", scope)?;
                }
                write!(formatter, "")
            }
            Self::ForLoop {
                allow_unroll,
                counter,
                start,
                end,
                body,
                ..
            } => write!(
                formatter,
                "for {:?} = {:?} to {:?}{} {{ {:?} }}",
                counter,
                start,
                end,
                if *allow_unroll { "" } else { " no_unroll" },
                body
            ),
            Self::StaticInit { body, exports, .. } => write!(
                formatter,
                "static init at {:?}, exports {:?};",
                body, exports
            ),
            Self::RawVPExpression(expr) => write!(formatter, "{:?}", expr),
        }
    }
}

impl Statement {
    pub fn clone_position(&self) -> FilePosition {
        match self {
            Self::CreationPoint { position, .. }
            | Self::Assert(_, position)
            | Self::Return(position)
            | Self::Assign { position, .. }
            | Self::Branch { position, .. }
            | Self::ForLoop { position, .. } => position.clone(),
            Self::StaticInit { position, .. } => position.clone(),
            Self::RawVPExpression(expr) => expr.clone_position(),
        }
    }
}
