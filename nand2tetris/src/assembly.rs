use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use bitflags::bitflags;

use crate::{CodeType, hex::Hex};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Target: u8 {
        const A = 0b100;
        const D = 0b010;
        const M = 0b001;
    }
}

impl TryFrom<&str> for Target {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut target = Target::empty();
        for c in value.chars() {
            match c {
                'A' => target |= Target::A,
                'M' => target |= Target::M,
                'D' => target |= Target::D,
                _ => return Err(format!("Unkown command target {}", c)),
            }
        }
        Ok(target)
    }
}

impl Target {
    fn compile(&self) -> u16 {
        (self.bits() as u16) << 3
    }
}

#[derive(Debug)]
pub enum Jump {
    NONE,
    JGT,
    JEQ,
    JGE,
    JLT,
    JNE,
    JLE,
    JMP,
}

impl TryFrom<&str> for Jump {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "" => Ok(Jump::NONE),
            "JGT" => Ok(Jump::JGT),
            "JEQ" => Ok(Jump::JEQ),
            "JGE" => Ok(Jump::JGE),
            "JLT" => Ok(Jump::JLT),
            "JNE" => Ok(Jump::JNE),
            "JLE" => Ok(Jump::JLE),
            "JMP" => Ok(Jump::JMP),
            _ => Err(format!("Unkown Jump condition {}", value)),
        }
    }
}

impl Jump {
    fn compile(&self) -> u16 {
        match self {
            Jump::NONE => 0b000,
            Jump::JGT => 0b001,
            Jump::JEQ => 0b010,
            Jump::JGE => 0b011,
            Jump::JLT => 0b100,
            Jump::JNE => 0b101,
            Jump::JLE => 0b110,
            Jump::JMP => 0b111,
        }
    }
}

#[derive(Debug)]
pub enum Compute {
    Zero,
    One,
    NegOne,
    D,
    A,
    NotD,
    NotA,
    NegD,
    NegA,
    DplusOne,
    AplusOne,
    DminOne,
    AminOne,
    DplusA,
    DminA,
    AminD,
    DandA,
    DorA,
    M,
    NotM,
    NegM,
    MplusOne,
    MminOne,
    DplusM,
    DminM,
    MminD,
    DandM,
    DorM,
}

impl TryFrom<&str> for Compute {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "0" => Ok(Compute::Zero),
            "1" => Ok(Compute::One),
            "-1" => Ok(Compute::NegOne),
            "D" => Ok(Compute::D),
            "A" => Ok(Compute::A),
            "!D" => Ok(Compute::NotD),
            "!A" => Ok(Compute::NotA),
            "-D" => Ok(Compute::NegD),
            "-A" => Ok(Compute::NegA),
            "D+1" => Ok(Compute::DplusOne),
            "A+1" => Ok(Compute::AplusOne),
            "D-1" => Ok(Compute::DminOne),
            "A-1" => Ok(Compute::AminOne),
            "D+A" => Ok(Compute::DplusA),
            "D-A" => Ok(Compute::DminA),
            "A-D" => Ok(Compute::AminD),
            "D&A" => Ok(Compute::DandA),
            "D|A" => Ok(Compute::DorA),
            "M" => Ok(Compute::M),
            "!M" => Ok(Compute::NotM),
            "-M" => Ok(Compute::NegM),
            "M+1" => Ok(Compute::MplusOne),
            "M-1" => Ok(Compute::MminOne),
            "D+M" => Ok(Compute::DplusM),
            "D-M" => Ok(Compute::DminM),
            "M-D" => Ok(Compute::MminD),
            "D&M" => Ok(Compute::DandM),
            "D|M" => Ok(Compute::DorM),
            _ => Err(format!("Unkown Compute command {}", value)),
        }
    }
}

impl Compute {
    fn compile(&self) -> u16 {
        let out = match self {
            Compute::Zero => 0b101010,
            Compute::One => 0b111111,
            Compute::NegOne => 0b111010,
            Compute::D => 0b001100,
            Compute::A | Compute::M => 0b110000,
            Compute::NotD => 0b001101,
            Compute::NotA | Compute::NotM => 0b110001,
            Compute::NegD => 0b001111,
            Compute::NegA | Compute::NegM => 0b110011,
            Compute::DplusOne => 0b011111,
            Compute::AplusOne | Compute::MplusOne => 0b110111,
            Compute::DminOne => 0b001110,
            Compute::AminOne | Compute::MminOne => 0b110010,
            Compute::DplusA | Compute::DplusM => 0b000010,
            Compute::DminA | Compute::DminM => 0b010011,
            Compute::AminD | Compute::MminD => 0b000111,
            Compute::DandA | Compute::DandM => 0b000000,
            Compute::DorA | Compute::DorM => 0b010101,
        };
        let mem: u16 = match self {
            Compute::M
            | Compute::NotM
            | Compute::NegM
            | Compute::MplusOne
            | Compute::MminOne
            | &Compute::DplusM
            | Compute::DminM
            | Compute::MminD
            | Compute::DandM
            | Compute::DorM => 1,
            _ => 0,
        };
        ((mem << 6) | out) << 6
    }
}

type Label = String;

#[derive(Debug)]
pub enum LoadData {
    Data(u16),
    Label(Label),
}

#[derive(Debug)]
pub enum Instruction {
    Label {
        label: Label,
    },
    Load {
        data: LoadData,
    },
    Command {
        compute: Compute,
        target: Target,
        jump: Jump,
    },
}

impl Instruction {
    fn from_str(value: &str) -> Result<Option<Self>, String> {
        // Strip comments and emptylines
        let mut value = value.trim();
        if value.is_empty() || value.starts_with("//") {
            return Ok(None);
        }
        if let Some((newval, _)) = value.split_once("//") {
            value = newval.trim();
        }

        // Load
        if value.starts_with("@") {
            let data = if value.chars().nth(1).unwrap().is_ascii_digit() {
                let num = value[1..].parse().unwrap();
                if num > (u16::MAX >> 1) {
                    return Err("Value {num} is too large to be represented.".to_string());
                }
                LoadData::Data(num)
            } else {
                LoadData::Label(value[1..].to_string())
            };
            return Ok(Some(Instruction::Load { data }));
        }

        // Label
        if value.starts_with("(") {
            let label = value.trim_matches(['(', ')']).to_string();
            return Ok(Some(Instruction::Label { label }));
        }

        // C Instruction
        let (target, rest) = if let Some((t, rest)) = value.split_once("=") {
            (t.try_into()?, rest)
        } else {
            (Target::empty(), value)
        };
        let (jump, cmp) = if let Some((compute, j)) = rest.split_once(";") {
            (j.try_into()?, compute)
        } else {
            (Jump::NONE, rest)
        };
        let compute = cmp.try_into()?;

        Ok(Some(Instruction::Command {
            compute,
            target,
            jump,
        }))
    }

    fn compile(&self, ls: &mut LabelStore) -> Option<u16> {
        match self {
            Instruction::Label { label: _ } => None,
            Instruction::Load { data: ld } => match ld {
                LoadData::Data(data) => Some(data & 0x7FFF),
                LoadData::Label(label) => Some(ls.get(label) & 0x7FFF),
            },
            Instruction::Command {
                compute,
                target,
                jump,
            } => Some(0xE000 | compute.compile() | target.compile() | jump.compile()),
        }
    }
}

#[derive(Debug)]
pub struct Assembly {
    instructions: Vec<Instruction>,
}

impl Assembly {
    pub fn from_file(path: &PathBuf) -> Self {
        let mut stringbuf = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut stringbuf)
            .unwrap();
        let instructions = stringbuf
            .lines()
            .flat_map(|e| Instruction::from_str(e).unwrap())
            .collect();
        Self { instructions }
    }

    pub fn compile(self) -> Result<CodeType, String> {
        let mut ls = LabelStore::new();

        let mut ic: u16 = 0;
        for instruction in &self.instructions {
            match instruction {
                Instruction::Label { label } => {
                    ls.insert(label, ic)?;
                }
                Instruction::Load { data: _ }
                | Instruction::Command {
                    compute: _,
                    target: _,
                    jump: _,
                } => ic += 1,
            }
        }

        let instructions = self
            .instructions
            .iter()
            .flat_map(|i| i.compile(&mut ls))
            .collect();
        Ok(CodeType::Hex(Hex { instructions }))
    }
}

#[derive(Debug)]
struct LabelStore {
    labels: HashMap<String, u16>,
    nextindex: u16,
}

impl LabelStore {
    fn new() -> LabelStore {
        let hm: HashMap<String, u16> = [
            ("SP", 0),
            ("LCL", 1),
            ("ARG", 2),
            ("THIS", 3),
            ("THAT", 4),
            ("R0", 0),
            ("R1", 1),
            ("R2", 2),
            ("R3", 3),
            ("R4", 4),
            ("R5", 5),
            ("R6", 6),
            ("R7", 7),
            ("R8", 8),
            ("R9", 9),
            ("R10", 10),
            ("R11", 11),
            ("R12", 12),
            ("R13", 13),
            ("R14", 14),
            ("R15", 15),
            ("SCREEN", 0x4000),
            ("KBD", 0x6000),
        ]
        .into_iter()
        .map(|(e1, e2)| (e1.to_string(), e2))
        .collect();
        LabelStore {
            labels: hm,
            nextindex: 16,
        }
    }

    fn get(&mut self, key: &str) -> u16 {
        *self.labels.entry(key.to_string()).or_insert_with(|| {
            let r = self.nextindex;
            self.nextindex += 1;
            r
        })
    }

    fn insert(&mut self, key: &str, value: u16) -> Result<(), String> {
        if self.labels.contains_key(key) {
            return Err(format!("Key '{}' already exists", key));
        }
        self.labels.insert(key.to_string(), value);
        Ok(())
    }
}
