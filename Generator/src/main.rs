use clap::Parser;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
use std::result::Result;

#[derive(Parser)]
#[clap(version = "1.0", author = "Aleksey S. <siroggi5@gmail.com>")]
struct Opts {
    /// Sets the name of the input file, with the probabilities of occurrence of characters
    #[clap(short, long)]
    input: String,
    /// Sets the name of the output file to which the generated symbols will be written
    #[clap(short, long)]
    output: String,
    /// Sets the range of possible generation of characters for the extended AS╨бI character table
    #[clap(short, long, validator(in_range))]
    range: usize,
    /// Sets how many characters will be generated
    #[clap(short, long, validator(greater_than_zero))]
    count: usize,
}
fn in_range(val: &str) -> Result<(), String> {
    let test = val.parse::<usize>().unwrap();
    if test > 0 && test <= 1024 {
        Ok(())
    } else {
        Err(String::from("Not in range 1..1024"))
    }
}
fn greater_than_zero(val: &str) -> Result<(), String> {
    if val.parse::<usize>().unwrap() > 0 {
        Ok(())
    } else {
        Err(String::from("count needed to be greater than zero"))
    }
}
fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();
    let mut file_in = File::open(opts.input)?;
    let mut out_file = File::create(opts.output)?;
    let range = opts.range;
    let count = opts.count;
    let mut in_matrix = String::new();
    file_in.read_to_string(&mut in_matrix)?;
    println!("------------------------------------------------------------------------------------------------");
    println!("Input from file\n{}", in_matrix);
    println!("------------------------------------------------------------------------------------------------");
    let mut trimmed = string_edit(in_matrix);
    let row_count = trimmed.lines().count();
    let mut column_count = Vec::<usize>::new();
    for _i in 0..row_count {
        column_count.push(0);
    }
    let mut line_index = 0;
    for line in trimmed.lines() {
        for c in line.chars() {
            if c.is_whitespace() {
                column_count[line_index] = column_count[line_index] + 1;
            }
        }
        line_index = line_index + 1;
    }
    println!("String edit\n{}", trimmed);
    println!("------------------------------------------------------------------------------------------------");
    for (i, cl) in column_count.iter().enumerate() {
        if &column_count[0] != &column_count[i] {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "╨Ъ╨╛╨╗-╨▓╨╛ ╤З╨╕╤Б╨╡╨╗ ╨▓ ╤Б╤В╤А╨╛╨║╨░╤Е ╨╜╨╡ ╤А╨░╨▓╨╜╤Л",
            ));
        } else if row_count != 1 {
            println!("col={};row={}", cl, row_count);
            if cl + 1 != row_count {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "╨Ь╨░╤В╤А╨╕╤Ж╨░ ╨╜╨╡ ╨║╨▓╨░╨┤╤А╨░╤В╨╜╨░╤П",
                ));
            }
        }
    }
    let mut symbols = Vec::<char>::new();
    let mut rng = rand::thread_rng();
    for _i in 0..column_count[0] + 1 {
        let t = rng.gen_range(0..range) as u32;
        let ch = char::from_u32(t).unwrap();
        symbols.push(ch);
    }
    println!("Vector of symbols: {:?}", symbols);
    trimmed = normalize_before_dot(trimmed);
    println!("Normalize before dot\n{}", trimmed);
    println!("------------------------------------------------------------------------------------------------");
    let max = find_max(&trimmed);
    trimmed = normalize_after_dot(trimmed, max);
    println!("Normalize after dot\n{}", trimmed);
    println!("------------------------------------------------------------------------------------------------");
    check_sum_in_row(&trimmed)?;
    let matrix = parse_to_array(&trimmed); //G[][] g[] F[][]
    if row_count == 1 {
        //TODO NR(╨╜╨╡ ╤А╨░╨▓╨╜╨╛ ╨▓╨╡╤А╨╛╤П╤В╨╜╤Л╨╡ ╤А╨░╨▒╨╛╤В╨░╤О╤В) RA(╤А╨░╨▓╨╜╨╛ ╨▓╨╡╤А╨╛╤П╤В╨╜╤Л╨╡ ╨╜╨╡ ╤А╨░╨▒╨╛╤В╨░╤О╤В)
        let mut rng = rand::thread_rng();
        let mut incluse: Vec<(i64, i64)> = Vec::new();
        for i in 0..=column_count[0]
        // ╨а╨╛╨╖╨▒╨╕╨╣╨╜╨╕╨║ 2 ╨┐╤А╨╛╨╡╨▒╨░╨╗
        {
            if i == 0 {
                incluse.push((0_i64, (i64::MAX as f64 * matrix[0][i]) as i64));
            } else {
                incluse.push((
                    incluse[i - 1].1,
                    (i64::MAX as f64 * (matrix[0][i] * (i + 1) as f64)) as i64,
                ));
            }
        }
        let mut i = 0;
        while i < count {
            let t: i64 = rng.gen();
            for sym_index in 0..=column_count[0] {
                if (incluse[sym_index].0..incluse[sym_index].1).contains(&t) {
                    let mut b = [0; 4];
                    let res = symbols[sym_index].encode_utf8(&mut b);
                    out_file.write_all(res.as_bytes())?;
                    i = i + 1;
                    break;
                }
            }
        }
        let mut q: f64 = 0.0;
        for i in 0..=column_count[0] {
            if matrix[0][i] > (-9.0_f64).exp() {
                q = q - (matrix[0][i] * matrix[0][i].log10()) / (2.0_f64).log10();
            }
        }
        let x: f64 = 1.0 - q / (((column_count[0] + 1) as f64).log(2.7) / (2.0_f64).log(2.7));
        println!("H(A)={}\nx={}", q, x);
        println!("n/n0={}", 1.0 / (1.0 - x));
    } else {
        let rw = row_count - 1;
        let mut b: Vec<f64> = vec![0.0; rw];
        let mut u: Vec<Vec<f64>> = vec![vec![0.0; rw]; rw];

        for i in 0..rw {
            b[i] = matrix[rw][i];
            for j in 0..rw {
                let mut eq = 0.0;
                if i == j {
                    eq = 1.0;
                }
                u[i][j] = eq + matrix[rw][i] - matrix[j][i];
                print!("{}+{}-{} ", eq, matrix[rw][i], matrix[i][j]);
            }
            println!("{}", matrix[rw][i]);
        }
        let mut p = gaus(u, b, rw);
        p.push(0.0);
        let mut sm = 0.0;
        for i in 0..rw {
            sm = sm + p[i];
        }
        p[rw] = 1.0 - sm;
        let mut r = p.clone();
        for i in 1..=rw {
            r[i] = r[i] + r[i - 1];
        }
        println!("{:?}", p);
        let mut incluse: Vec<(i64, i64)> = Vec::new();
        for i in 0..=rw {
            if i == 0 {
                incluse.push((0_i64, (i64::MAX as f64 * r[i]) as i64));
            } else {
                incluse.push((incluse[i - 1].1, (i64::MAX as f64 * r[i]) as i64));
            }
        }
        let mut rng = rand::thread_rng();
        let mut i = usize::MIN;
        for mut i in 0..rw {
            let t: i64 = rng.gen();
            if (incluse[i].0..incluse[i].1).contains(&t) {
                let mut b = [0; 4];
                let res = symbols[i].encode_utf8(&mut b);
                out_file.write(res.as_bytes())?;
                i = row_count;
                break;
            }
        }
        if i <= row_count {
            let mut b = [0; 4];
            let res = char::from_u32((10 + rw) as u32)
                .unwrap()
                .encode_utf8(&mut b);
            out_file.write(res.as_bytes())?;
        }
        i = 1;
        while i < count {
            let t: i64 = rng.gen();
            for j in 0..=rw {
                if (incluse[j].0..incluse[j].1).contains(&t) {
                    let mut b = [0; 4];
                    let res = symbols[j].encode_utf8(&mut b);
                    out_file.write(res.as_bytes())?;
                    i += 1;
                    break;
                }
            }
        }
        let mut q = 0.0;
        for i in 0..row_count {
            if p[i] > (-9.0_f64).exp() {
                q = q - p[i] * (p[i].log10() / (2.0_f64).log10());
            }
        }
        let x = 1.0 - q / ((row_count as f64).log(2.7) / (2.0_f64).log(2.7));
        let mut q_ = 0.0;

        for i in 0..row_count {
            let mut q = 0.0;
            for j in 0..row_count {
                q = q + matrix[i][j] * (matrix[i][j].log10() / (2.0_f64).log10());
            }
            q_ = q_ - p[i] * q;
        }
        let x_ = 1.0 - q_ / ((row_count as f64).log(2.7) / (2.0_f64).log(2.7));
        println!("H(A)={}\nx(A)={}\nn/n0={}", q, x, 1.0 / (1.0 - x));
        println!("H(A|A^)={}\nx(A|A^)={}\nn_/n0={}", q_, x_, 1.0 / (1.0 - x_));
    }
    Ok(())
}
fn gaus(mut matrix: Vec<Vec<f64>>, mut free_elements: Vec<f64>, count: usize) -> Vec<f64> {
    let mut x = vec![1.0; count];
    //╨Т╤Л╤З╨╕╤В╨░╨╜╨╕╨╡ ╨╕╨╖ ╤Б╤В╤А╨╛╨║ ╨╜╨╡ ╨┤╤А╤Г╨│╨╕╨╡ ╤Б╤В╤А╨╛╨║╨╕ ╤Г╨╝╨╜╨╛╨╢╨╡╨╜╨╜╤Л╨╡ ╨╜╨░ ╤З╨╕╤Б╨╗╨╛
    for i in 0..count - 1 {
        sort_rows(i, &mut matrix, &mut free_elements, count);
        for j in (i + 1)..count {
            if matrix[i][i] != 0.0 {
                let mult_element = matrix[j][i] / matrix[i][i];
                for k in i..count {
                    matrix[j][k] = matrix[j][k] - (matrix[i][k] * mult_element);
                }
                free_elements[j] = free_elements[j] - (free_elements[i] * mult_element);
            }
        }
    }
    for i in (0..=count - 1).rev() {
        x[i] = free_elements[i];
        for j in ((i + 1)..count).rev() {
            x[i] = x[i] - (matrix[i][j] * x[j]);
        }
        x[i] = x[i] / matrix[i][i];
    }
    x
}
//╨б╨╛╤А╤В╨╕╤А╨╛╨▓╨║╨░ ╤Б╤В╤А╨╛╨║╨╕ ╨╝╨░╤В╤А╨╕╤Ж╤Л ╨┐╨╛ ╨▓╨╛╨╖╤А╨░╤Б╤В╨░╨╜╨╕╤О
fn sort_rows(
    sort_index: usize,
    matrix: &mut Vec<Vec<f64>>,
    right_part: &mut Vec<f64>,
    count: usize,
) {
    let mut max_element: f64 = matrix[sort_index][sort_index];
    let mut max_element_index = sort_index;
    for i in (sort_index + 1)..count {
        if matrix[i][sort_index] > max_element {
            max_element = matrix[i][sort_index];
            max_element_index = i;
        }
    }
    if max_element_index > sort_index {
        let mut temp: f64;
        temp = right_part[max_element_index];
        right_part[max_element_index] = right_part[sort_index];
        right_part[sort_index] = temp;
        for i in 0..count {
            temp = matrix[max_element_index][i];
            matrix[max_element_index][i] = matrix[sort_index][i];
            matrix[sort_index][i] = temp;
        }
    }
}
fn string_edit(input: String) -> String {
    let mut output = String::new();
    let mut current = usize::MIN;
    let count = input.lines().count() - 1;
    for line in input.lines() {
        let nline = line.trim_end();
        let mut temp = String::new();
        if !line.is_empty() || nline.len() == 0 {
            for (i, c) in nline.char_indices() {
                if c.is_whitespace()
                    || c == '\t'
                        && (line.chars().nth(i - 1).unwrap().is_whitespace()
                            || line.chars().nth(i - 1).unwrap() == '\t')
                {
                } else if c.is_whitespace() || c == '\t' {
                } else if i != 0
                    && ((line.chars().nth(i - 1).unwrap().is_whitespace()
                        || line.chars().nth(i - 1).unwrap() == '\t')
                        && c.is_numeric())
                {
                    temp.push(' ');
                    temp.push(c);
                } else if c == ',' || c == '.' || c.is_numeric() {
                    if c == ',' {
                        temp.push('.');
                    } else {
                        temp.push(c);
                    }
                }
            }
            temp = temp.trim().to_string();
            if !temp.is_empty() {
                output.push_str(temp.as_str());
            }
            if current != count {
                output.push('\n');
            }
            current = current + 1;
        }
    }
    output
}
fn normalize_before_dot(input: String) -> String {
    let mut output = String::new();
    let mut current = usize::MIN;
    let count = input.lines().count();
    for line in input.lines() {
        for (i, c) in line.char_indices() {
            if line.chars().nth(i + 1).is_some() {
                if c.is_numeric() && line.chars().nth(i + 1).unwrap() != '.' {
                    if i != 0 {
                        if line.chars().nth(i - 1).unwrap() != '.'
                            && !line.chars().nth(i - 1).unwrap().is_numeric()
                        {
                            output.push('0');
                            output.push('.');
                        }
                    } else {
                        output.push('0');
                        output.push('.');
                    }
                }
            }
            output.push(c);
        }
        if current != count {
            current = current + 1;
            output.push('\n');
        }
    }
    output
}
fn find_max(input: &String) -> usize {
    let mut max = usize::MIN;
    for line in input.lines() {
        for (i, c) in line.char_indices() {
            if line.chars().nth(i + 1).is_some() {
                if c == '.' || (c.is_numeric() && line.chars().nth(i + 1).unwrap() == '.') {
                    continue;
                }
            } else if c.is_numeric() {
                max = max + 1;
            }
        }
    }
    max
}
fn normalize_after_dot(input: String, max: usize) -> String {
    let mut output = String::new();
    let count = input.lines().count();
    let mut current_line = usize::MIN;
    let mut current = usize::MIN;
    for line in input.lines() {
        for (i, c) in line.char_indices() {
            if line.chars().nth(i + 1).is_some() {
                if c == '.'
                    || (c.is_numeric() && line.chars().nth(i + 1).unwrap() == '.')
                    || c.is_whitespace()
                {
                    output.push(c);
                } else if c.is_numeric() {
                    current = current + 1;

                    output.push(c);
                    if current < max && line.chars().nth(i + 1).is_some() {
                        if line.chars().nth(i + 1).unwrap().is_whitespace()
                            || line.chars().count() == i + 1
                        {
                            for _i in 1..=(max - current) {
                                output.push('0');
                            }
                            current = 0;
                        }
                    }
                }
            } else {
                output.push(c);
                current = current + 1;
                if current < max {
                    for _i in 1..=(max - current) {
                        output.push('0');
                    }
                    current = 0;
                }
            }
        }
        current = 0;
        if current_line != count {
            current_line = current_line + 1;
            output.push('\n');
        }
    }

    output
}
fn check_sum_in_row(input: &String) -> Result<(), std::io::Error> {
    for line in input.lines() {
        let values = line
            .split(" ")
            .filter_map(|s| s.parse::<f32>().ok())
            .collect::<Vec<_>>();
        let mut summ = 0.0;
        for v in &values {
            summ = summ + v;
        }
        if summ < 0.9999998 {
            return Result::Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("╨Т ╨╛╨┤╨╜╨╛╨╣ ╨╕╨╖ ╤Б╤В╤А╨╛╨║ ╤Б╤Г╨╝╨╝╨░ ╨▓╨╡╤А╨╛╤П╤В╨╜╨╛╤Б╤В╨╡╨╣ ╨╜╨╡ ╤А╨░╨▓╨╜╨░ 1\n sum={}\narray={:?}", &summ,&values),
            ));
        }
    }
    Ok(())
}
fn parse_to_array(input: &String) -> Vec<Vec<f64>> {
    let mut output = Vec::<Vec<f64>>::new();
    for line in input.lines() {
        output.push(
            line.split(" ")
                .filter_map(|s| s.parse::<f64>().ok())
                .collect::<Vec<_>>(),
        );
    }
    output
}
