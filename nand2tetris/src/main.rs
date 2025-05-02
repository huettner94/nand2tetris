use std::{ffi::OsStr, fmt::Display, path::PathBuf, process::exit};

use assembly::Assembly;
use clap::Parser;
use hex::Hex;
use vm::VM;

pub mod assembly;
pub mod hex;
pub mod vm;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: PathBuf,
}

enum FileType {
    ASSEMBLY,
    VM,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::ASSEMBLY => f.write_str("asm"),
            FileType::VM => f.write_str("vm"),
        }
    }
}

impl TryFrom<&OsStr> for FileType {
    type Error = &'static str;

    fn try_from(value: &OsStr) -> Result<Self, Self::Error> {
        match value.to_str().unwrap_or_default() {
            "asm" => Ok(FileType::ASSEMBLY),
            "vm" => Ok(FileType::VM),
            _ => Err("Filetype not recognized"),
        }
    }
}

fn main() {
    let args = Args::parse();
    if !args.file.is_file() {
        println!(
            "File {} does not exist or is not a file",
            args.file.to_string_lossy()
        );
        exit(1);
    }
    let filetype = match FileType::try_from(args.file.extension().unwrap_or_default()) {
        Ok(v) => v,
        Err(e) => {
            println!("Error getting filetype: {e}");
            exit(1);
        }
    };
    println!(
        "Compiling {} of type {}",
        args.file.to_string_lossy(),
        filetype
    );
    let code = load_file(&args.file, filetype).unwrap();
    let basepath = args.file.clone().with_extension("");
    code.compile(basepath).unwrap();
}

pub enum CodeType {
    VM(VM),
    Assembly(Assembly),
    Hex(Hex),
}

fn load_file(file: &PathBuf, filetype: FileType) -> Result<CodeType, String> {
    match filetype {
        FileType::ASSEMBLY => Ok(CodeType::Assembly(Assembly::from_file(file)?)),
        FileType::VM => Ok(CodeType::VM(VM::from_file(file)?)),
    }
}

impl CodeType {
    fn compile(self, basepath: PathBuf) -> Result<(), String> {
        let out = match self {
            CodeType::VM(v) => v.compile()?,
            CodeType::Assembly(v) => v.compile()?,
            CodeType::Hex(_) => return Ok(()),
        };
        out.write(&basepath);
        out.compile(basepath)?;
        Ok(())
    }

    fn write(&self, basepath: &PathBuf) {
        match self {
            CodeType::VM(_) => (),
            CodeType::Assembly(_) => (),
            CodeType::Hex(v) => v.write(basepath.clone()),
        }
    }
}
