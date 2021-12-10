use std::env;
use std::fs::File;
use std::io::{Read, Seek};
fn help() {
    println!("check_for_generator.exe In.txt");
}
fn main() -> std::io::Result<()>{
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2=>{
            let mut file_in = File::open(&args[1])?;
            let mut in_matrix = String::new();
            file_in.read_to_string(&mut in_matrix)?;
            let len = in_matrix.chars().count();
            let mut ch_count: Vec<(char,usize)> = Vec::new();
            let mut 
            Ok(())
        },
        _=>
        {
            println!("Provided {} args", args.len() - 1);
            help();
            Ok(())
        }
    }
}
