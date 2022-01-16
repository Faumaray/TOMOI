use std::{fs::File, io::Read};

use clap::Parser;
#[derive(Parser)]
#[clap(version = "1.0", author = "Aleksey S. <siroggi5@gmail.com>")]
struct Opts {
    /// Sets the name of the input file, with characters to code
    #[clap(short, long)]
    one_input: String,
    /// Sets the name of the output file
    #[clap(short, long)]
    two_input: String,
}
fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let mut one = File::open(opts.one_input)?;
    let mut two = File::open(opts.two_input)?;
    println!("First File size: {}", one.metadata().unwrap().len());
    println!("Second File size: {}", two.metadata().unwrap().len());
    let mut string_one = String::new();
    one.read_to_string(&mut string_one)?;
    let mut string_two = String::new();
    two.read_to_string(&mut string_two)?;
    let vc_1 = string_one.chars().collect::<Vec<_>>();
    let vc_2 = string_two.chars().collect::<Vec<_>>();

    let mut err: Vec<usize> = Vec::new();
    for i in 0..vc_1.len() {
        if i < vc_2.len() {
            if vc_1[i] != vc_2[i] {
                err.push(i);
            }
        } else {
            err.push(i);
        }
    }
    if err.len() == 0 {
        println!("All Equal");
    } else {
        /* for i in err {
            if i < vc_2.len() {
                println!("Err index = {} ||{:?}!={:?}", i, vc_1[i], vc_2[i]);
            } else {
                println!(
                    "Err index = {} || {:?} doesn`t contains in second file",
                    i, vc_1[i]
                )
            }
        }*/
        println!("{}", err.len() - 1);
    }
    Ok(())
}
