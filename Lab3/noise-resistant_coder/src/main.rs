use std::fs::File;

use clap::Parser;
use compression::utils::encode;

#[derive(Parser)]
#[clap(version = "1.0", author = "Aleksey S. <siroggi5@gmail.com>")]
struct Opts {
    /// Sets the name of the input file, with characters to code
    #[clap(short, long)]
    input: String,
    /// Sets the name of the output file
    #[clap(short, long)]
    output: String,
}
fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let mut input = File::open(opts.input)?;
    let mut output = File::create(opts.output)?;
    encode(input, output)?;
    Ok(())
}
