//! Instruction utilities.

use super::Label;
use rspirv::dr;

#[derive(Debug)]
pub enum Merge {
    Selection { merge: Label },
    Loop { merge: Label, r#continue: Label },
}

#[derive(Debug)]
pub enum Branch {
    Unconditional(Label),
    Conditional { then: Label, r#else: Label },
    Switch { cases: Vec<Label>, default: Label },
    Return,
    Kill,
    Unreachable,
}

pub trait InsnUtils {
    fn opcode(&self) -> spirv_headers::Op;
    fn as_merge(&self) -> Option<Merge>;
    fn as_branch(&self) -> Option<Branch>;
}

impl InsnUtils for dr::Instruction {
    fn opcode(&self) -> spirv_headers::Op {
        self.class.opcode
    }

    fn as_merge(&self) -> Option<Merge> {
        use spirv_headers::Op;

        match self.opcode() {
            Op::LoopMerge => Some(Merge::Loop {
                merge: self.operands[0].unwrap_id_ref(),
                r#continue: self.operands[1].unwrap_id_ref(),
            }),
            Op::SelectionMerge => Some(Merge::Selection {
                merge: self.operands[0].unwrap_id_ref(),
            }),
            _ => None,
        }
    }

    fn as_branch(&self) -> Option<Branch> {
        use spirv_headers::Op;

        match self.opcode() {
            Op::Branch => Some(Branch::Unconditional(self.operands[0].unwrap_id_ref())),
            Op::BranchConditional => Some(Branch::Conditional {
                then: self.operands[1].unwrap_id_ref(),
                r#else: self.operands[2].unwrap_id_ref(),
            }),
            Op::Switch => Some(Branch::Switch {
                cases: self.operands[2..]
                    .chunks_exact(2)
                    .map(|case| case[1].unwrap_id_ref())
                    .collect(),
                default: self.operands[1].unwrap_id_ref(),
            }),
            Op::Return | Op::ReturnValue => Some(Branch::Return),
            Op::Kill => Some(Branch::Kill),
            Op::Unreachable => Some(Branch::Unreachable),
            _ => None,
        }
    }
}
