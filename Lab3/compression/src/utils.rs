use std::{
    fs::File,
    io::{prelude::*, BufReader, BufWriter, IoSlice, SeekFrom},
    io::{Error, ErrorKind},
};

use bitvec::vec::BitVec;
pub fn encode(mut input: File, mut output: File) -> std::io::Result<()> {
    {
        let mut chars = String::new();
        input.read_to_string(&mut chars)?;
        let mut reader = BufReader::new(input);

        // write to dst file
        let dst = File::create("tmp")?;
        let mut writer = BufWriter::new(dst);

        // allocate a u8 buffer of size == block_size

        let weights = crate::huff_encode::build_weights(chars.as_str());
        let mut tree = crate::huff_encode::Tree::build_from_weights(weights);
        tree.assign_codes();
        let tree_bin = tree.as_bin();
        let tree_bin_padding = crate::huff_encode::calc_padding_bits(tree_bin.len());
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
    }
    let mut tmp = File::open("tmp")?;
    let mut buffer = [0; 7];
    let mut output_bytes: Vec<u8> = vec![];
    while let Ok(b) = tmp.read(&mut buffer[..7]) {
        // Not sure why f.read doesn't stop?
        //
        // This is definitely a bug and why we are only
        // processing 7168 bytes.
        if b == 0 {
            break;
        }

        let encoded = crate::hamming::encode(buffer);
        for data in encoded {
            output_bytes.push(data);
        }

        // Clear buffer for next block
        buffer = [0; 7];
    }
    output.write(output_bytes.as_slice())?;
    std::fs::remove_file("tmp")?;
    Ok(())
}
fn compress_to_writer<R: Read, W: Write + Seek>(
    reader: &mut R,
    writer: &mut W,
    tree: crate::huff_encode::Tree,
) -> Result<u8, Error> {
    let tree = tree;

    let prev_byte = 0;
    let mut prev_padding = 0;
    /// compress the buffer into CompressData, combining it with
    /// the prev_byte if the prev_padding != 0
    macro_rules! comp_data_from {
        ($buf:expr) => {{
            // get and own the compress data
            let (mut comp_bytes, padding_bits, huff_tree) =
                crate::huff_encode::compress_with_tree($buf, tree.clone())
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
pub fn decode(mut input: File, mut output: File, repair: bool) -> std::io::Result<()> {
    let mut buffer = [0; 8];
    let mut output_bytes: Vec<u8> = vec![];
    while let Ok(b) = input.read(&mut buffer[..8]) {
        // Not sure why f.read doesn't stop?
        //
        // This is definitely a bug and why we are only
        // processing 7168 bytes.
        if b == 0 {
            break;
        }

        //println!("{:?}", buffer);

        let decoded = crate::hamming::decode(buffer, repair);

        if repair {
            for data in decoded.unwrap() {
                output_bytes.push(data);
            }
        }

        // Clear buffer for next block
        buffer = [0; 8];
    }
    if repair {
        let mut tmp = File::create("tmp")?;
        tmp.write(output_bytes.as_slice())?;
    }
    {
        // read from src file
        let src = File::open("tmp")?;
        let mut src_bytes_left = src.metadata().unwrap().len() as usize;
        let reader = BufReader::new(src);

        // write to dst file
        let dst = output;
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
                    "tmp"
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
                format!("{:?} stores invalid header information", "tmp"),
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
                    "tmp"
                ),
            ));
        }
        src_bytes_left -= tree_len;

        // read the HuffTree
        let tree = match crate::huff_tree::Tree::try_from_bin({
            let mut b = BitVec::from_vec(buf[..tree_len].to_vec());
            for _ in 0..tree_padding_bits {
                b.pop();
            }
            b
        }) {
            Ok(tree) => tree,
            Err(err) => return Err(Error::new(ErrorKind::NotFound, format!("{:?}", err))),
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
        std::fs::remove_file("tmp")?;
    }
    Ok(())
}

fn decompress_to_writer<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    reader_bytes_left: &mut usize,
    buf: &mut [u8],
    tree: crate::huff_tree::Tree,
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
