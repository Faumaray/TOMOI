use bitvec::{bits, bitvec, macros::internal::funty::IsNumber, order::Msb0, prelude::BitVec};
use clap::Parser;
use huff_tree::huff_tree::Tree;
mod huff_tree;
use std::{
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{BufReader, BufWriter, Error, ErrorKind, Read, Seek, SeekFrom, Write},
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
    // read from src file
    let src = File::open(&opts.input)?;
    let mut src_bytes_left = src.metadata().unwrap().len() as usize;
    let reader = BufReader::new(src);

    // write to dst file
    let dst = File::create(opts.output)?;
    let mut writer = BufWriter::new(dst);

    // allocate a u8 buffer of size == block_size
    let mut buf = vec![0; 1024];

    // read only first 5 bytes
    let mut reader = reader.take(5);
    let bytes_read = reader.read(&mut buf)?;
    if bytes_read < 5 {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "{:?} too short to decompress, missing header information",
                &opts.input
            ),
        ));
    }
    src_bytes_left -= 5;

    // read padding info from the first byte
    let padding = buf[0];
    let tree_padding_bits = padding >> 4;
    let data_padding_bits = padding & 0b0000_1111;
    if tree_padding_bits > 7 || data_padding_bits > 7 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("{:?} stores invalid header information", opts.input),
        ));
    }
    // read tree_bin's length
    let tree_len = u32::from_be_bytes(buf[1..5].try_into().unwrap()) as usize;

    // read only next tree_len bytes
    reader.set_limit(tree_len as u64);
    let bytes_read = reader.read(&mut buf)?;
    if bytes_read < tree_len {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "{:?} too short to decompress, missing header information",
                &opts.input
            ),
        ));
    }
    src_bytes_left -= tree_len;

    // read the HuffTree
    let tree = match Tree::try_from_bin({
        let mut b = BitVec::from_vec(buf[..tree_len].to_vec());
        for _ in 0..tree_padding_bits {
            b.pop();
        }
        b
    }) {
        Ok(tree) => tree,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "{:?} too short to decompress, missing header information",
                    &opts.input
                ),
            ))
        }
    };
    // decompress the remaining bytes
    let mut reader = reader.into_inner();
    decompress_to_writer(
        &mut reader,
        &mut writer,
        &mut src_bytes_left,
        &mut buf,
        tree,
        data_padding_bits,
    )?;

    writer.flush()?;
    Ok(())
}
fn decompress_to_writer<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    reader_bytes_left: &mut usize,
    buf: &mut [u8],
    tree: Tree,
    padding_bits: u8,
) -> Result<(), Error> {
    // do pretty much the same thing as in huff_coding::comp::decompress
    // see it's docs for an explanation
    let mut decomp_buf = Vec::new();
    let mut current_branch = tree.root();
    macro_rules! read_codes_in_byte {
        ($byte: expr;[$bitrange:expr]) => {
            for bit_ptr in $bitrange {
                if current_branch.has_children() {
                    match ($byte >> (7 - bit_ptr)) & 1 == 1 {
                        true => {
                            current_branch = current_branch.right_child().unwrap();
                        }
                        false => {
                            current_branch = current_branch.left_child().unwrap();
                        }
                    }
                }
                if !current_branch.has_children() {
                    decomp_buf.push(current_branch.letter().unwrap().clone());
                    current_branch = tree.root();
                }
            }
        };
    }
    // try to read exactly buf.len() bytes, decompressing them and writing
    while reader.read_exact(buf).is_ok() {
        for byte in &buf[..] {
            read_codes_in_byte!(byte;[0..8]);
        }
        let tmp = &decomp_buf.iter().collect::<String>();
        writer.write_all(tmp.as_bytes())?;
        decomp_buf.clear();
        *reader_bytes_left -= buf.len();
    }
    // if couldn't read exactly buf.len() bytes and there are some bytes left,
    // decompress them minding the padding bits
    if *reader_bytes_left > 0 {
        for byte in &buf[..*reader_bytes_left - 1] {
            read_codes_in_byte!(byte;[0..8]);
        }
        read_codes_in_byte!(buf[*reader_bytes_left - 1];[0..8 - padding_bits]);
        let tmp = &decomp_buf.iter().collect::<String>();
        writer.write_all(tmp.as_bytes())?;
    }
    Ok(())
}
