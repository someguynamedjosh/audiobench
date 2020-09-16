use super::{LabelId, Value};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Debug)]
pub enum Condition {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

impl Condition {
    pub fn as_negative(&self) -> Condition {
        match self {
            Self::LessThan => Self::GreaterThanOrEqual,
            Self::GreaterThan => Self::LessThanOrEqual,
            Self::LessThanOrEqual => Self::GreaterThan,
            Self::GreaterThanOrEqual => Self::LessThan,
            Self::Equal => Self::NotEqual,
            Self::NotEqual => Self::Equal,
        }
    }
}

pub enum UnaryOperator {
    NegI,
    NegF,
    Not,
    BNot,
    FSin,
    FCos,
    FSqrt,
    FExp,
    FExp2,
    FLog,
    FLog10,
    FLog2,
    FAbs,
    IAbs,
    FFloor,
    FCeil,
    FTrunc,
    Ftoi,
    Itof,
}

pub enum BinaryOperator {
    AddI,
    SubI,
    MulI,
    DivI,
    ModI,

    AddF,
    SubF,
    MulF,
    DivF,
    ModF,
    PowF,

    BAnd,
    BOr,
    BXor,
    LeftShift,
    RightShift,

    And,
    Or,
    Xor,
    CompI(Condition),
    CompF(Condition),
}

pub enum Instruction {
    Move {
        from: Value,
        to: Value,
    },
    Load {
        from: Value,
        from_indexes: Vec<Value>,
        to: Value,
    },
    Store {
        from: Value,
        to: Value,
        to_indexes: Vec<Value>,
    },

    UnaryOperation {
        op: UnaryOperator,
        a: Value,
        x: Value,
    },
    BinaryOperation {
        op: BinaryOperator,
        a: Value,
        b: Value,
        x: Value,
    },

    Label(LabelId),
    Jump {
        label: LabelId,
    },
    Branch {
        condition: Value,
        true_target: LabelId,
        false_target: LabelId,
    },
    Abort(u32),
}

impl Debug for Instruction {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Instruction::Move { from, to } => write!(formatter, "move {:?} -> {:?}", from, to),
            Instruction::Load {
                from,
                to,
                from_indexes,
            } => {
                write!(formatter, "load ({:?})", from)?;
                for index in from_indexes {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, " -> {:?}", to)
            }
            Instruction::Store {
                from,
                to,
                to_indexes,
            } => {
                write!(formatter, "store {:?} -> ({:?})", from, to)?;
                for index in to_indexes {
                    write!(formatter, "[{:?}]", index)?;
                }
                write!(formatter, "")
            }

            Instruction::UnaryOperation { op, a, x } => write!(
                formatter,
                "{} {:?} -> {:?}",
                match op {
                    UnaryOperator::BNot => "bnot",
                    UnaryOperator::FAbs => "fabs",
                    UnaryOperator::FCeil => "fceil",
                    UnaryOperator::FCos => "fcos",
                    UnaryOperator::FExp => "fexp",
                    UnaryOperator::FExp2 => "fexp2",
                    UnaryOperator::FFloor => "ffloor",
                    UnaryOperator::FLog => "flog",
                    UnaryOperator::FLog10 => "flog10",
                    UnaryOperator::FLog2 => "flog2",
                    UnaryOperator::FSin => "fsin",
                    UnaryOperator::FSqrt => "fsqrt",
                    UnaryOperator::FTrunc => "ftrunc",
                    UnaryOperator::IAbs => "iabs",
                    UnaryOperator::NegF => "negf",
                    UnaryOperator::NegI => "negi",
                    UnaryOperator::Not => "not",
                    UnaryOperator::Ftoi => "ftoi",
                    UnaryOperator::Itof => "itof",
                },
                a,
                x,
            ),
            Instruction::BinaryOperation { op, a, b, x } => write!(
                formatter,
                "{} {:?}, {:?} -> {:?}",
                match op {
                    BinaryOperator::AddI => "addi".to_owned(),
                    BinaryOperator::SubI => "subi".to_owned(),
                    BinaryOperator::MulI => "muli".to_owned(),
                    BinaryOperator::DivI => "divi".to_owned(),
                    BinaryOperator::ModI => "modi".to_owned(),

                    BinaryOperator::AddF => "addf".to_owned(),
                    BinaryOperator::SubF => "subf".to_owned(),
                    BinaryOperator::MulF => "mulf".to_owned(),
                    BinaryOperator::DivF => "divf".to_owned(),
                    BinaryOperator::ModF => "modf".to_owned(),
                    BinaryOperator::PowF => "powf".to_owned(),

                    BinaryOperator::BAnd => "band".to_owned(),
                    BinaryOperator::BOr => "bor ".to_owned(),
                    BinaryOperator::BXor => "bxor".to_owned(),
                    BinaryOperator::LeftShift => "<<".to_owned(),
                    BinaryOperator::RightShift => ">>".to_owned(),

                    BinaryOperator::And => "and".to_owned(),
                    BinaryOperator::Or => "or ".to_owned(),
                    BinaryOperator::Xor => "xor".to_owned(),
                    BinaryOperator::CompI(cond) => format!("compi {:?}", cond),
                    BinaryOperator::CompF(cond) => format!("compf {:?}", cond),
                },
                a,
                b,
                x
            ),

            Instruction::Label(id) => write!(formatter, "labl {:?}", id),
            Instruction::Jump { label } => write!(formatter, "jump to {:?}", label),
            Instruction::Branch {
                condition,
                true_target,
                false_target,
            } => write!(
                formatter,
                "if {:?} jump to {:?} else {:?}",
                condition, true_target, false_target
            ),
            Instruction::Abort(error_code) => write!(formatter, "abort {}", error_code),
        }
    }
}
