use std::{fs::OpenOptions, io::Write, path::PathBuf};

pub struct Hex {
    pub instructions: Vec<u16>,
}

impl Hex {
    pub fn write(&self, mut basepath: PathBuf) {
        basepath.set_extension("hack");
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(basepath.clone())
            .unwrap();
        for instr in &self.instructions {
            file.write(format!("{:0>16b}\n", instr).as_bytes()).unwrap();
        }
        file.flush().unwrap();
        println!("Written output to {}", basepath.to_str().unwrap());
    }
}
