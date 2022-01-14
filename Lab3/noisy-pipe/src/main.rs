use clap::Parser;
use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser)]
#[clap(version = "1.0", author = "Aleksey S. <siroggi5@gmail.com>")]
struct Opts {
    /// Sets the name of the input file, with characters to code
    #[clap(short, long)]
    input: String,
    /// Вероятность ошибки
    #[clap(short, long)]
    prob: f64,
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let mut input = File::open(&opts.input)?;
    let mut output = File::create(format!("{}.dmg", &opts.input))?;
    let mut buffer = [0; 8];
    let mut rng = rand::thread_rng();
    while let Ok(b) = input.read(&mut buffer[..8]) {
        // Not sure why f.read doesn't stop?
        //
        // This is definitely a bug and why we are only
        // processing 7168 bytes.
        if b == 0 {
            break;
        }
        let mut original = u64::from_be_bytes(buffer);
        if rng.gen_bool(opts.prob) {
            let invalid_bit = rng.gen_range(0..56);
            let mask: u64 = 0b1 << invalid_bit;
            println!("flipping bit: {}", invalid_bit);

            // Toggle that specific bit
            original ^= mask;
            output.write(&original.to_be_bytes())?;
        } else {
            output.write(&original.to_be_bytes())?;
        }
        //println!("{:?}", buffer);

        // Clear buffer for next block
        buffer = [0; 8];
    }

    Ok(())
}
