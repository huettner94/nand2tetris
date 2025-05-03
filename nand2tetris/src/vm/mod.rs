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
        let mut out = Statement::pop(Target::D);
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

    fn compile(&self, lg: &mut LabelGenerator) -> Vec<Instruction> {
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
    label_generator: LabelGenerator,
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
            label_generator: LabelGenerator::new(path),
        })
    }

    pub fn compile(mut self) -> Result<CodeType, String> {
        let mut out = Vec::new();

        for statement in self.ast {
            out.append(&mut statement.compile(&mut self.label_generator));
        }

        Ok(CodeType::Assembly(Assembly::from_instructions(out)))
    }
}

#[derive(Debug)]
struct LabelGenerator {
    filename: String,
    last_var: u16,
    last_statement: u16,
}

impl LabelGenerator {
    fn new(filename: &PathBuf) -> Self {
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
