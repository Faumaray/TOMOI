/// Ration a `Vec<T>` into ration_count sized rations.
/// 
/// Edge cases:
/// * If cannot ration equally -> dumps the remainder into the last ration.
/// * If `Vec<T>` is to small -> returns just one ration with the whole `Vec<T>` in it.
pub fn ration_vec<T: Clone>(vec: &[T], ration_count: usize) -> Vec<Vec<T>>{
    let mut elements_left = vec.len();
    let elements_per_ration = elements_left / ration_count;
    let mut current_element = 0;

    let mut rations: Vec<Vec<T>> = Vec::with_capacity(ration_count);
    if elements_per_ration == 0{
        rations.push(vec[..].to_vec());
    }
    else{
        for i in 0..ration_count{
            if i == ration_count - 1{
                rations.push(vec[current_element..].to_vec());
                break;
            }
        
            rations.push(vec[current_element..current_element + elements_per_ration].to_vec());
            current_element += elements_per_ration;
            elements_left -= elements_per_ration;
        }
    }
    rations
}

/// Returns the size of the type in bits
pub fn size_of_bits<T>() -> usize{
    std::mem::size_of::<T>() * 8
}

/// Return how many bits will be used as padding
/// with given the bit_count.
pub fn calc_padding_bits(bit_count: usize) -> u8{
    let n = (8 - bit_count % 8) as u8; 
    match n{8 => 0, _ => n}
}
