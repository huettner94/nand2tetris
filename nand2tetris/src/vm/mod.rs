use std::{collections::HashSet, fs::File, io::Read, path::PathBuf};

use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::{Parser, error::Rich};
use compiler::LabelGenerator;
use parser::Span;

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

    Label(String),
    Goto(String),
    IfGoto(String),
}

#[derive(Debug)]
pub struct VM {
    ast: Vec<Statement>,
    label_generator: LabelGenerator,
}

impl VM {
    pub fn from_file(path: &PathBuf) -> Result<Self, String> {
        let mut src = String::new();
        File::open(&path)
            .map_err(|e| e.to_string())?
            .read_to_string(&mut src)
            .map_err(|e| e.to_string())?;
        let src2 = src.clone();
        let (out, errs) = parser::parser(path.file_name().unwrap().to_str().unwrap())
            .parse(&src2)
            .into_output_errors();
        println!("Parse output: {:?}", out);
        if !errs.is_empty() {
            let filename = path.to_str().unwrap().to_string();
            Self::print_err(errs, filename, src);
            return Err("Failed to compile".to_string());
        }
        Ok(VM {
            ast: out.unwrap().into_iter().map(|(s, _)| s).collect(),
            label_generator: LabelGenerator::new(path),
        })
    }

    fn print_err<'a>(errs: Vec<Rich<'a, char, Span>>, filename: String, src: String) {
        errs.into_iter()
            .map(|e| e.map_token(|c| c.to_string()))
            .for_each(|e| {
                Report::build(ReportKind::Error, (filename.clone(), e.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_message(e.to_string())
                    .with_label(
                        Label::new((filename.clone(), e.span().into_range()))
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .with_labels(e.contexts().map(|(label, span)| {
                        Label::new((filename.clone(), span.into_range()))
                            .with_message(format!("while parsing this {}", label))
                            .with_color(Color::Yellow)
                    }))
                    .finish()
                    .print(sources([(filename.clone(), src.clone())]))
                    .unwrap()
            });
    }

    pub fn compile(mut self) -> Result<CodeType, String> {
        let mut out = Vec::new();

        let mut labels = HashSet::new();
        for s in &self.ast {
            if let Statement::Label(l) = s {
                if labels.contains(&l) {
                    return Err(format!("Duplicate Label definition '{}'", l));
                }
                labels.insert(l);
            }
        }

        for statement in self.ast {
            out.append(&mut statement.compile(&mut self.label_generator));
        }

        Ok(CodeType::Assembly(Assembly::from_instructions(out)))
    }
}
