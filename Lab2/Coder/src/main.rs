use bitvec::{bits, bitvec, macros::internal::funty::IsNumber, order::Msb0, prelude::BitVec};
use clap::Parser;
use huff_tree::huff_tree::Tree;
mod huff_tree;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Error, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    vec,
};
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
    let mut src = File::open(opts.input)?;
    let mut chars = String::new();
    src.read_to_string(&mut chars)?;
    let mut reader = BufReader::new(src);

    // write to dst file
    let dst = File::create(opts.output)?;
    let mut writer = BufWriter::new(dst);

    // allocate a u8 buffer of size == block_size

    let weights = huff_tree::huff_tree::build_weights(chars.as_str());
    let mut tree = huff_tree::huff_tree::Tree::build_from_weights(weights);
    tree.assign_codes();
    let tree_bin = tree.as_bin();
    let tree_bin_padding = huff_tree::huff_tree::calc_padding_bits(tree_bin.len());
    let tree_bin_bytes = tree_bin.into_vec();
    // return reader to start
    reader.seek(SeekFrom::Start(0))?;

    // write an empty byte, later to be filled by padding data
    writer.write_all(&[0])?;
    // write the tree_bin_bytes lenght as a 4 byte num
    writer.write_all(&(tree_bin_bytes.len() as u32).to_be_bytes())?;
    // write the HuffTree represented as bytes
    writer.write_all(&tree_bin_bytes)?;
    // compress and write compressed bytes, returning the number of bits used as padding
    let comp_padding = compress_to_writer(&mut reader, &mut writer, tree)?;

    // return to the start of the file and set the padding bits
    writer.seek(SeekFrom::Start(0))?;
    writer.write_all(&[(tree_bin_padding << 4) + comp_padding])?;

    writer.flush()?;
    Ok(())
}
fn compress_to_writer<R: Read, W: Write + Seek>(
    reader: &mut R,
    writer: &mut W,
    tree: Tree,
) -> Result<u8, Error> {
    let tree = tree;

    let prev_byte = 0;
    let mut prev_padding = 0;
    println!("{:#?}", tree);
    /// compress the buffer into CompressData, combining it with
    /// the prev_byte if the prev_padding != 0
    macro_rules! comp_data_from {
        ($buf:expr) => {{
            // get and own the compress data
            let (mut comp_bytes, padding_bits, huff_tree) =
                huff_tree::huff_tree::compress_with_tree($buf, tree.clone())
                    .unwrap()
                    .into_inner();
            // if the previous compress data's padding isn't 0
            // write the comp_bytes minding the padding
            if prev_padding != 0 {
                writer.seek(SeekFrom::Current(-1)).unwrap();

                comp_bytes = offset_bytes(&comp_bytes, prev_padding as usize);
                comp_bytes[0] |= prev_byte
            }

            (comp_bytes, padding_bits, huff_tree)
        }};
    }
    let mut tro = String::new();
    reader.read_to_string(&mut tro)?;
    // try to read exactly buf.len() bytes, compressing them and repeating
    let (comp_bytes, padding_bits, _huff_tree) = comp_data_from!(tro);
    writer.write_all(&comp_bytes)?;
    prev_padding = padding_bits;

    // return the written compressed data's padding bits
    Ok(prev_padding)
}
fn offset_bytes(bytes: &[u8], n: usize) -> Vec<u8> {
    let empty_bytes = n / 8;
    let mut offset_bytes = vec![0; empty_bytes];
    offset_bytes.reserve_exact(bytes.len());

    let mut comp_byte = 0b0000_0000;
    let mut bit_ptr = (7 - n) % 8;
    for byte in bytes {
        for i in 0..8 {
            comp_byte |= (((byte >> (7 - i)) & 1 == 1) as u8) << bit_ptr;

            if bit_ptr == 0 {
                offset_bytes.push(comp_byte);
                comp_byte = 0b0000_0000;
                bit_ptr = 7;
            } else {
                bit_ptr -= 1
            };
        }
    }
    let padding_bits = if bit_ptr == 7 { 0 } else { bit_ptr + 1 };
    if padding_bits != 0 {
        offset_bytes.push(comp_byte);
    }

    offset_bytes
}
