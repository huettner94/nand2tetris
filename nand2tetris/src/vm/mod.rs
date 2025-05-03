use std::{fs::File, io::Read, path::PathBuf};

use chumsky::Parser;
use compiler::LabelGenerator;

use crate::{CodeType, assembly::Assembly};

mod compiler;
mod parser;

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
        let out = parser::parser().parse(&stringbuf);
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
