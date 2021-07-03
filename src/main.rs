mod insn;
mod read_spirv;

use anyhow::Result;
use rspirv::{binary, dr};
use spirv_headers::Word;
use std::collections::BTreeMap as Map;
use std::path;
use structopt::StructOpt;

use insn::InsnUtils;

#[derive(StructOpt)]
#[structopt(name = "spirv-tree")]
/// Show the tree structure of the functions in a SPIR-V file.
struct Options {
    #[structopt(parse(from_os_str))]
    /// SPIR-V (.spv) or SPIR-V assembly (.spvasm) files.
    inputs: Vec<path::PathBuf>,
}

type Label = Word;

fn main() -> Result<()> {
    let options = Options::from_args();

    let mut first = true;
    for input in options.inputs {
        let bytes = read_spirv::read_spirv_bytes(&input)?;
        let module = {
            let mut loader = dr::Loader::new();
            binary::Parser::new(&bytes, &mut loader).parse()?;
            loader.module()
        };

        if !first {
            println!();
        }
        first = false;

        println!("file: {}", input.display());
        let mut roles = Map::new();
        for (i, function) in module.functions.iter().enumerate() {
            println!("fn {}: {} blocks", i, function.blocks.len());

            let label_to_index = label_to_index_map(function);
            println!("{:#?}", label_to_index);

            for block in &function.blocks {
                let label = block.label_id().unwrap();
                let flow = flow(block, &mut roles);
                println!(
                    "{:?}: {}{:#?}",
                    label,
                    match roles.get(&label) {
                        Some(role) => format!("{:?} ", role),
                        None => String::new(),
                    },
                    flow
                );
            }
        }
    }
    Ok(())
}

fn label_to_index_map(function: &dr::Function) -> Map<Label, usize> {
    function
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(i, block)| block.label_id().map(|label| (label, i)))
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LabelRole {
    SelectionMerge { header: Label },
    LoopHead,
    LoopMerge { header: Label },
    LoopContinue { header: Label },
    Case { header: Label },
}

#[derive(Debug)]
enum Flow {
    BareBranch(Label),
    BareConditional {
        then: Label,
        r#else: Label,
    },
    If {
        then: Label,
        r#else: Label,
        merge: Label,
    },
    Switch {
        cases: Vec<Label>,
        default: Label,
        merge: Label,
    },
    ToSelectionMerge {
        header: Label,
    },
    Loop {
        merge: Label,
        r#continue: Label,
        branch: LoopBranch,
    },
    LoopBackEdge,
    UnconditionalJump {
        target: Label,
        role: LabelRole,
    },
    ConditionalJump {
        then: (Label, Option<LabelRole>),
        r#else: (Label, Option<LabelRole>),
    },
    Return,
    Kill,
    Unreachable,
}

#[derive(Debug)]
enum LoopBranch {
    Unconditional(Label),
    Conditional { then: Label, r#else: Label },
}

fn flow(block: &dr::Block, roles: &mut Map<Label, LabelRole>) -> Flow {
    use insn::{Branch, Merge};

    let label = block.label_id().unwrap();

    let termination = block.termination().as_branch().unwrap_or_else(|| {
        panic!("Block didn't end with a branch: {:?}", block.termination());
    });

    let merge = {
        let instructions = &block.instructions;
        if instructions.len() >= 2 {
            instructions[instructions.len() - 2].as_merge()
        } else {
            None
        }
    };

    match merge {
        Some(Merge::Selection { merge }) => match termination {
            Branch::Conditional { then, r#else } => {
                roles.insert(merge, LabelRole::SelectionMerge { header: label });
                Flow::If {
                    then,
                    r#else,
                    merge,
                }
            },
            Branch::Switch { cases, default } => {
                roles.extend(
                    cases
                        .iter()
                        .map(|&label| (label, LabelRole::Case { header: label })),
                );
                roles.insert(default, LabelRole::Case { header: label });
                roles.insert(merge, LabelRole::SelectionMerge { header: label });

                Flow::Switch {
                    cases,
                    default,
                    merge,
                }
            },
            _ => panic!("OpSelectionMerge must be followed by an OpBranchConditional or OpSwitch"),
        },
        Some(Merge::Loop { merge, r#continue }) => {
            roles.insert(label, LabelRole::LoopHead);
            roles.insert(merge, LabelRole::LoopMerge { header: label });
            roles.insert(r#continue, LabelRole::LoopContinue { header: label });

            match termination {
                Branch::Conditional { then, r#else } => Flow::Loop {
                    merge,
                    r#continue,
                    branch: LoopBranch::Conditional { then, r#else },
                },
                Branch::Unconditional(target) => Flow::Loop {
                    merge,
                    r#continue,
                    branch: LoopBranch::Unconditional(target),
                },
                _ => panic!("OpLoopMerge must be followed by an OpBranch or OpBranchConditional"),
            }
        },
        None => match termination {
            Branch::Unconditional(target) => match roles.get(&target).cloned() {
                Some(LabelRole::SelectionMerge { header }) => Flow::ToSelectionMerge { header },
                Some(LabelRole::LoopHead) => Flow::LoopBackEdge,
                Some(role) => Flow::UnconditionalJump { target, role },
                None => Flow::BareBranch(target),
            },
            Branch::Conditional { then, r#else } => {
                match (roles.get(&then).cloned(), roles.get(&r#else).cloned()) {
                    (None, None) => Flow::BareConditional { then, r#else },
                    (then_role, else_role) => Flow::ConditionalJump {
                        then: (then, then_role),
                        r#else: (r#else, else_role),
                    },
                }
            }
            Branch::Switch { .. } => {
                panic!("OpSwitch without OpSelectionMerge");
            }
            Branch::Return => Flow::Return,
            Branch::Kill => Flow::Kill,
            Branch::Unreachable => Flow::Unreachable,
        },
    }
}

trait BlockUtils {
    fn termination(&self) -> &dr::Instruction;
}

impl BlockUtils for dr::Block {
    fn termination(&self) -> &dr::Instruction {
        self.instructions.last().unwrap()
    }
}
