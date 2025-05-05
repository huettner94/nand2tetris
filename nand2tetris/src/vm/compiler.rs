use std::path::PathBuf;

use crate::assembly::{Compute, Instruction, Jump, LoadData, Target};

use super::{PopDest, PushSource, Statement};

#[derive(Debug)]
pub struct LabelGenerator {
    filename: String,
    last_var: u16,
    last_statement: u16,
}

impl LabelGenerator {
    pub fn new(filename: &PathBuf) -> Self {
        LabelGenerator {
            filename: filename.to_str().unwrap().to_string(),
            last_var: 0,
            last_statement: 0,
        }
    }

    fn next_statement(&mut self) -> String {
        let val = self.last_statement;
        self.last_statement += 1;
        format!("{}-stmt-{}", &self.filename, val)
    }
}

impl Statement {
    fn set_d(val: u16) -> Vec<Instruction> {
        [
            Instruction::Load {
                data: LoadData::Data(val),
            },
            Instruction::Command {
                compute: Compute::A,
                target: Target::D,
                jump: Jump::NONE,
            },
        ]
        .to_vec()
    }

    fn push_d() -> Vec<Instruction> {
        [
            Instruction::Load {
                data: LoadData::label("SP"),
            },
            Instruction::Command {
                compute: Compute::M,
                target: Target::A,
                jump: Jump::NONE,
            },
            Instruction::Command {
                compute: Compute::D,
                target: Target::M,
                jump: Jump::NONE,
            },
            Instruction::Load {
                data: LoadData::label("SP"),
            },
            Instruction::Command {
                compute: Compute::MplusOne,
                target: Target::M,
                jump: Jump::NONE,
            },
        ]
        .to_vec()
    }

    fn pop(target: Target) -> Vec<Instruction> {
        [
            Instruction::Load {
                data: LoadData::label("SP"),
            },
            Instruction::Command {
                compute: Compute::MminOne,
                target: Target::A | Target::M,
                jump: Jump::NONE,
            },
            Instruction::Command {
                compute: Compute::M,
                target,
                jump: Jump::NONE,
            },
        ]
        .to_vec()
    }

    fn cmp(lg: &mut LabelGenerator, jmp: Jump) -> Vec<Instruction> {
        let truelabel = lg.next_statement();
        let endlabel = lg.next_statement();
        let mut out = Self::pop(Target::D);
        out.append(&mut Self::pop(Target::A));
        out.extend(
            [
                Instruction::Command {
                    compute: Compute::AminD,
                    target: Target::D,
                    jump: Jump::NONE,
                },
                Instruction::Load {
                    data: LoadData::label(&truelabel),
                },
                Instruction::Command {
                    compute: Compute::D,
                    target: Target::empty(),
                    jump: jmp,
                },
            ]
            .to_vec(),
        );
        out.append(&mut Self::set_d(0));
        out.extend(
            [
                Instruction::Load {
                    data: LoadData::label(&endlabel),
                },
                Instruction::Command {
                    compute: Compute::A,
                    target: Target::empty(),
                    jump: Jump::JMP,
                },
                Instruction::label(&truelabel),
                // This loads -1 to D
                Instruction::Command {
                    compute: Compute::NegOne,
                    target: Target::D,
                    jump: Jump::NONE,
                },
            ]
            .to_vec(),
        );
        out.extend([Instruction::label(&endlabel)].to_vec());
        out.append(&mut Self::push_d());
        out
    }

    fn compute2(compute: Compute) -> Vec<Instruction> {
        let mut out = Statement::pop(Target::D);
        out.extend([
            Instruction::Load {
                data: LoadData::label("SP"),
            },
            Instruction::Command {
                compute: Compute::MminOne,
                target: Target::A,
                jump: Jump::NONE,
            },
            Instruction::Command {
                compute,
                target: Target::M,
                jump: Jump::NONE,
            },
        ]);
        out
    }

    fn compute1(compute: Compute) -> Vec<Instruction> {
        [
            Instruction::Load {
                data: LoadData::label("SP"),
            },
            Instruction::Command {
                compute: Compute::MminOne,
                target: Target::A,
                jump: Jump::NONE,
            },
            Instruction::Command {
                compute,
                target: Target::M,
                jump: Jump::NONE,
            },
        ]
        .to_vec()
    }

    fn push_common(label: &str, index: u16) -> Vec<Instruction> {
        let mut out = [
            Instruction::Load {
                data: LoadData::label(label),
            },
            Instruction::Command {
                compute: Compute::M,
                target: Target::D,
                jump: Jump::NONE,
            },
            Instruction::Load {
                data: LoadData::Data(index),
            },
            Instruction::Command {
                compute: Compute::DplusA,
                target: Target::A,
                jump: Jump::NONE,
            },
            Instruction::Command {
                compute: Compute::M,
                target: Target::D,
                jump: Jump::NONE,
            },
        ]
        .to_vec();
        out.append(&mut Self::push_d());
        out
    }

    fn pop_common(label: &str, index: u16) -> Vec<Instruction> {
        let mut out = [
            Instruction::Load {
                data: LoadData::label(label),
            },
            Instruction::Command {
                compute: Compute::M,
                target: Target::D,
                jump: Jump::NONE,
            },
            Instruction::Load {
                data: LoadData::Data(index),
            },
            Instruction::Command {
                compute: Compute::DplusA,
                target: Target::D,
                jump: Jump::NONE,
            },
            Instruction::Load {
                data: LoadData::label("R13"),
            },
            Instruction::Command {
                compute: Compute::D,
                target: Target::M,
                jump: Jump::NONE,
            },
        ]
        .to_vec();
        out.append(&mut Self::pop(Target::D));
        out.extend(
            [
                Instruction::Load {
                    data: LoadData::label("R13"),
                },
                Instruction::Command {
                    compute: Compute::M,
                    target: Target::A,
                    jump: Jump::NONE,
                },
                Instruction::Command {
                    compute: Compute::D,
                    target: Target::M,
                    jump: Jump::NONE,
                },
            ]
            .to_vec(),
        );
        out
    }

    fn static_name(filename: &str, index: u16) -> String {
        format!("staticvar.{}.{}", filename, index)
    }

    fn temp_name(index: u16) -> String {
        assert!(index <= 7);
        format!("R{}", index + 5)
    }

    fn pointer_name(index: u16) -> String {
        assert!(index <= 1);
        format!("R{}", index + 3)
    }

    fn push_fixed(label: &str) -> Vec<Instruction> {
        let mut out = [
            Instruction::Load {
                data: LoadData::label(label),
            },
            Instruction::Command {
                compute: Compute::M,
                target: Target::D,
                jump: Jump::NONE,
            },
        ]
        .to_vec();
        out.append(&mut Self::push_d());
        out
    }

    fn pop_fixed(label: &str) -> Vec<Instruction> {
        let mut out = Self::pop(Target::D);
        out.extend(
            [
                Instruction::Load {
                    data: LoadData::label(label),
                },
                Instruction::Command {
                    compute: Compute::D,
                    target: Target::M,
                    jump: Jump::NONE,
                },
            ]
            .to_vec(),
        );
        out
    }

    pub fn compile(&self, lg: &mut LabelGenerator) -> Vec<Instruction> {
        match self {
            Statement::Not => Self::compute1(Compute::NotM),
            Statement::And => Self::compute2(Compute::DandM),
            Statement::Or => Self::compute2(Compute::DorM),
            Statement::Neg => Self::compute1(Compute::NegM),
            Statement::Add => Self::compute2(Compute::DplusM),
            Statement::Sub => Self::compute2(Compute::MminD),
            Statement::Eq => Self::cmp(lg, Jump::JEQ),
            Statement::Lt => Self::cmp(lg, Jump::JLT),
            Statement::Gt => Self::cmp(lg, Jump::JGT),

            Statement::Push(PushSource::Constant, i) => {
                let mut out = Statement::set_d(*i);
                out.append(&mut Self::push_d());
                out
            }
            Statement::Push(PushSource::Local, i) => Self::push_common("LCL", *i),
            Statement::Push(PushSource::Argument, i) => Self::push_common("ARG", *i),
            Statement::Push(PushSource::This, i) => Self::push_common("THIS", *i),
            Statement::Push(PushSource::That, i) => Self::push_common("THAT", *i),
            Statement::Push(PushSource::Static(filename), i) => {
                Self::push_fixed(&Self::static_name(filename, *i))
            }
            Statement::Push(PushSource::Temp, i) => Self::push_fixed(&Self::temp_name(*i)),
            Statement::Push(PushSource::Pointer, i) => Self::push_fixed(&Self::pointer_name(*i)),

            Statement::Pop(PopDest::Local, i) => Self::pop_common("LCL", *i),
            Statement::Pop(PopDest::Argument, i) => Self::pop_common("ARG", *i),
            Statement::Pop(PopDest::This, i) => Self::pop_common("THIS", *i),
            Statement::Pop(PopDest::That, i) => Self::pop_common("THAT", *i),
            Statement::Pop(PopDest::Static(filename), i) => {
                Self::pop_fixed(&Self::static_name(filename, *i))
            }
            Statement::Pop(PopDest::Temp, i) => Self::pop_fixed(&Self::temp_name(*i)),
            Statement::Pop(PopDest::Pointer, i) => Self::pop_fixed(&Self::pointer_name(*i)),
        }
    }
}
