use shared_util::rcrc;

use super::{
    controls::AnyControl,
    parts::{IOJack, JackType},
};
use crate::engine::controls::InputControl;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryMathOp {
    Negate,
    Reciprocal,
    NaturalExp,
    NaturalLog,
    Exp10,
    Log10,
    Exp2,
    Log2,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
}

impl UnaryMathOp {
    pub fn label(&self) -> &str {
        match self {
            UnaryMathOp::Negate => "Negate",
            UnaryMathOp::Reciprocal => "Reciprocal",
            UnaryMathOp::NaturalExp => "Natural Exponent",
            UnaryMathOp::NaturalLog => "Natural Logarithm",
            UnaryMathOp::Exp10 => "Base 10 Exponent",
            UnaryMathOp::Log10 => "Base 10 Logarithm",
            UnaryMathOp::Exp2 => "Base 2 Exponent",
            UnaryMathOp::Log2 => "Base 2 Logarithm",
            UnaryMathOp::Sin => "Sine",
            UnaryMathOp::Cos => "Cosine",
            UnaryMathOp::Tan => "Tangent",
            UnaryMathOp::Asin => "Arcsin",
            UnaryMathOp::Acos => "Arccosine",
            UnaryMathOp::Atan => "Arctangent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryMathOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    ReplaceNan,
}

impl BinaryMathOp {
    pub fn label(&self) -> &str {
        match self {
            BinaryMathOp::Add => "Add",
            BinaryMathOp::Subtract => "Subtract",
            BinaryMathOp::Multiply => "Multiply",
            BinaryMathOp::Divide => "Divide",
            BinaryMathOp::Modulo => "Modulo",
            BinaryMathOp::Power => "Power",
            BinaryMathOp::ReplaceNan => "Replace NaN",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinModuleType {
    SawtoothOsc,
    SawtoothWave,
    UnaryMathOp {
        op: UnaryMathOp,
        typee: JackType,
    },
    BinaryMathOp {
        op: BinaryMathOp,
        lhs_type: JackType,
        rhs_type: JackType,
    },
}

impl BuiltinModuleType {
    pub fn label(&self) -> &str {
        use BuiltinModuleType::*;
        match self {
            SawtoothOsc => "Sawtooth Osc",
            SawtoothWave => "Sawtooth Wave",
            UnaryMathOp { op, .. } => op.label(),
            BinaryMathOp { op, .. } => op.label(),
        }
    }

    pub fn default_controls(&self) -> Vec<AnyControl> {
        use BuiltinModuleType::*;
        match self {
            SawtoothOsc => vec![AnyControl::Input(rcrc(InputControl::new(
                JackType::Pitch,
                0,
            )))],
            SawtoothWave => vec![],
            UnaryMathOp { typee, .. } => {
                vec![AnyControl::Input(rcrc(InputControl::new(*typee, 0)))]
            }
            BinaryMathOp {
                lhs_type, rhs_type, ..
            } => {
                vec![
                    AnyControl::Input(rcrc(InputControl::new(*lhs_type, 0))),
                    AnyControl::Input(rcrc(InputControl::new(*rhs_type, 0))),
                ]
            }
        }
    }

    pub fn outputs(self) -> Vec<IOJack> {
        match self {
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::SawtoothOsc,
            Self::UnaryMathOp {
                op: UnaryMathOp::Sin,
                typee: JackType::Audio,
            },
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleType {
    Builtin(BuiltinModuleType),
}

impl ModuleType {
    pub fn label(&self) -> &str {
        match self {
            Self::Builtin(builtin) => builtin.label(),
        }
    }

    pub fn default_controls(&self) -> Vec<AnyControl> {
        match self {
            Self::Builtin(builtin) => builtin.default_controls(),
        }
    }

    pub fn outputs(&self) -> &[IOJack] {
        match self {
            Self::Builtin(builtin) => builtin.outputs(),
        }
    }

    pub(crate) fn size(&self) -> (i32, i32) {
        (2, 2)
    }
}
