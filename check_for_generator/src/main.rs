use std::fs::File;
use std::io::{Read, Seek};
use std::{collections::HashMap, env};
fn help() {
    println!("check_for_generator.exe In.txt");
}
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            let mut file_in = File::open(&args[1])?;
            let mut in_matrix = String::new();
            file_in.read_to_string(&mut in_matrix)?;
            let len = in_matrix.chars().count();
            let vc = in_matrix.chars().collect::<Vec<_>>();
            let mut basic = build_weights(vc);
            println!("{:#?}", basic);
            Ok(())
        }
        _ => {
            println!("Provided {} args", args.len() - 1);
            help();
            Ok(())
        }
    }
}
pub fn build_weights(s: Vec<char>) -> HashMap<String, usize> {
    let mut h = HashMap::new();
    for i in 0..s.len() {
        let counter = h.entry(format!("{}", s[i])).or_insert(0);
        *counter += 1;
        if i != 0 {
            let counter = h.entry(format!("{}|{}", s[i - 1], s[i])).or_insert(0);
            *counter += 1;
        }
    }
    h
}
pub fn build_second(s: &str) -> HashMap<String, usize> {
    let mut h = HashMap::new();
    for (index, ch) in s.char_indices() {}

    h
}
