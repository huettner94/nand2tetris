use chumsky::prelude::*;
use std::{fs::File, io::Read, path::PathBuf};
use text::{inline_whitespace, newline};

use crate::{
    CodeType,
    assembly::{Assembly, Compute, Instruction, Jump, LoadData, Target},
};

#[derive(Debug, Clone)]
enum PushSource {
    Constant,
    Local,
    Argument,
    Static(String),
    This,
    That,
    Temp,
    Pointer,
}

#[derive(Debug, Clone)]
enum PopDest {
    Local,
    Argument,
    Static(String),
    This,
    That,
    Temp,
    Pointer,
}

#[derive(Debug, Clone)]
enum Statement {
    Not,
    And,
    Or,

    Neg,
    Add,
    Sub,

    Eq,
    Lt,
    Gt,

    Push(PushSource, u16),
    Pop(PopDest, u16),
}

impl Statement {
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

    fn compile(&self) -> Vec<Instruction> {
        match self {
            Statement::Not => todo!(),
            Statement::And => todo!(),
            Statement::Or => todo!(),
            Statement::Neg => todo!(),
            Statement::Add => {
                let mut out = Statement::pop(Target::D);
                out.append(&mut Statement::pop(Target::A));
                out.extend([Instruction::Command {
                    compute: Compute::DplusA,
                    target: Target::D,
                    jump: Jump::NONE,
                }]);
                out.append(&mut Statement::push_d());
                out
            }
            Statement::Sub => todo!(),
            Statement::Eq => todo!(),
            Statement::Lt => todo!(),
            Statement::Gt => todo!(),
            Statement::Push(PushSource::Constant, i) => {
                let mut out = [
                    Instruction::Load {
                        data: LoadData::Data(*i),
                    },
                    Instruction::Command {
                        compute: Compute::A,
                        target: Target::D,
                        jump: Jump::NONE,
                    },
                ]
                .to_vec();
                out.append(&mut Statement::push_d());
                out
            }
            Statement::Push(push_source, _) => todo!(),
            Statement::Pop(pop_dest, _) => todo!(),
        }
    }
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Statement>> {
    let comment = just("//").then(any().and_is(newline().not()).repeated());

    let opt_comment_and_newline = comment.repeated().at_most(1).ignore_then(newline());

    let int = text::int(10)
        .padded_by(inline_whitespace())
        .map(|s: &str| u16::from_str_radix(s, 10).unwrap());

    let pc = just("push")
        .padded_by(inline_whitespace())
        .ignore_then(just("constant").padded_by(inline_whitespace()))
        .ignore_then(int)
        .map(|n| Statement::Push(PushSource::Constant, n));

    let line = choice((
        just("not").to(Statement::Not),
        just("and").to(Statement::And),
        just("or").to(Statement::Or),
        just("neg").to(Statement::Neg),
        just("add").to(Statement::Add),
        just("sub").to(Statement::Sub),
        just("eq").to(Statement::Eq),
        just("lt").to(Statement::Lt),
        just("gt").to(Statement::Gt),
        pc,
    ));

    line.separated_by(opt_comment_and_newline.repeated())
        .allow_trailing()
        .allow_leading()
        .collect()
}

#[derive(Debug)]
pub struct VM {
    ast: Vec<Statement>,
}

impl VM {
    pub fn from_file(path: &PathBuf) -> Result<Self, String> {
        let mut stringbuf = String::new();
        File::open(&path)
            .map_err(|e| e.to_string())?
            .read_to_string(&mut stringbuf)
            .map_err(|e| e.to_string())?;
        let out = parser().parse(&stringbuf);
        println!("Parse output: {:?}", out);
        if out.has_errors() {
            return Err(out.errors().map(|e| e.to_string()).collect());
        }
        Ok(VM {
            ast: out.output().unwrap().clone(),
        })
    }

    pub fn compile(self) -> Result<CodeType, String> {
        let mut out = Vec::new();

        for statement in self.ast {
            out.append(&mut statement.compile());
        }

        Ok(CodeType::Assembly(Assembly::from_instructions(out)))
    }
}
