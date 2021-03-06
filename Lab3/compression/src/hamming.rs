const INDEXES: [usize; 7] = [0, 1, 3, 7, 15, 31, 63];
use bitvec::{prelude as bv, view::BitView};
pub fn encode(mut block: [u8; 7]) -> Vec<u8> {
    let mut next = bv::BitVec::<u8, bv::Msb0>::with_capacity(64);
    let mut bv = block.view_bits_mut::<bv::Msb0>();
    //println!("{:?}", bv);
    let mut prev = 0;
    for index in 0..64 {
        if INDEXES.contains(&index) || index == 2 {
            next.push(false);
        } else {
            next.push(bv[prev]);
            prev += 1;
        }
    }
    let mut values = Vec::<bv::BitVec<u8, bv::Lsb0>>::with_capacity(64);
    for i in 1u8..65u8 {
        values.push(i.view_bits::<bv::Lsb0>().to_bitvec());
    }
    //println!("{}", next);
    let mut r = [0i32; INDEXES.len()];
    for j in 0..INDEXES.len() {
        for i in 0..next.len() {
            let one = *(next.get(i).as_deref().unwrap());
            let two = *(values[i].get(j).as_deref().unwrap());
            if (one == true) && (two == true) {
                r[j] += 1;
            }
        }
    }
    for index in 0..r.len() {
        if r[index].rem_euclid(2) == 1 {
            next.set(INDEXES[index], true);
        } else {
            next.set(INDEXES[index], false);
        }
    }
    //println!("{}", next);
    // We put the parity bits at the top for performance reasons
    // Vec<bool> len u32 == 32
    return next.into();
}
pub fn decode(mut code: [u8; 8], repair: bool, block_count: usize) -> (Option<Vec<u8>>, usize) {
    // We have an error
    let mut values = Vec::<bv::BitVec<u8, bv::Lsb0>>::with_capacity(64);
    for i in 1u8..65u8 {
        values.push(i.view_bits::<bv::Lsb0>().to_bitvec());
    }
    let mut next = code.view_bits_mut::<bv::Msb0>();
    //println!("{}", next);
    let mut error_count = 0;
    let mut has_error = false;
    loop {
        let mut r = [0i32; INDEXES.len()];
        for j in 0..INDEXES.len() {
            for i in 0..next.len() {
                let one = *(next.get(i).as_deref().unwrap());
                let two = *(values[i].get(j).as_deref().unwrap());
                if (one == true) && (two == true) {
                    r[j] += 1;
                }
            }
        }
        let mut error_vec = bv::BitVec::<u8, bv::Lsb0>::with_capacity(r.len());
        for index in 0..r.len() {
            if r[index].rem_euclid(2) == 1 {
                error_vec.push(true);
                has_error = true;
            } else {
                error_vec.push(false);
            }
        }
        //println!("{}", error_vec);
        let error: Vec<u8> = error_vec.clone().into();
        if !has_error {
            break;
        }
        println!("------------------------------------------------------------------------------");
        println!("block ???{}", block_count);
        println!("error at index {}", error[0]);
        error_count += 1;
        let value = *next.get((error[0] - 1) as usize).as_deref().unwrap();
        next.set((error[0] - 1) as usize, !value);
        //println!("after repair:\n{}", next);
        has_error = false;
    }
    if repair {
        let mut out = bv::BitVec::<u8, bv::Msb0>::with_capacity(56);
        for index in 0..64 {
            if !(INDEXES.contains(&index) || index == 2) {
                out.push(*next.get(index).as_deref().unwrap());
            }
        }
        return (Some(out.into()), error_count);
    }
    (None, error_count)
}
