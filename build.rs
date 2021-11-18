use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::convert::{TryInto, From};
use std::fs::File;
use std::{env, path};
use std::io::Write;
use std::num::Wrapping;

struct LUTEntry {
    m128_hi: u64,
    m128_lo: u64,
    widebiased_e2: i32
}

// Magic numbers for Eisel-Lemire table generation

// N is large enough so that (1<<N) is easily bigger than 1e310.
const N: i32 = 2048;

// 1214 is 1023 + 191. 1023 is the bias for IEEE 754 double-precision floating
// point. 191 is ((3 * 64) - 1) and we work with multiples-of-64-bit mantissas.
const BIAS: i32 = 1214;

/// Generates the 128-bit lookup table for Eisel-Lemire
fn gen_lookup_table(min_exponent: i16, max_exponent:i16) -> Vec<LUTEntry> {
    let two128: BigUint = One::one();
    let two128: BigUint = two128 << 128;
    assert!(two128.bits() == 129);
    (min_exponent..=max_exponent).into_iter().map(|e10| gen_lut_entry(e10, &two128)).collect()
}

fn gen_lut_entry(e10: i16, two128: &BigUint) -> LUTEntry {
    assert!((-310i16..=310i16).contains(&e10), "E10 is out of range!");
    let mut z: BigUint = One::one();
    z <<= N;  // Exp is now larger than 10^e10 for sure

    // Multiply z by 10^e10 using integer arithmetic. Since we can't actually
    // do 10^(negative) with integer arithmetic, implement as 10^(abs(e10))
    // followed by either multiply or divide.
    if e10 >= 0 {
        let e10:u32 = e10.abs().try_into().unwrap();
        let mult_val = BigUint::from(10u8).pow(e10);
        z = z * mult_val;
    } else {
        let e10:u32 = e10.abs().try_into().unwrap();
        let div_val = BigUint::from(10u8).pow(e10);
        assert!(div_val != Zero::zero(), "Division value is zero on input of {}", e10);
        z = z / div_val;
    }

    // Pow2 exponent
    let mut e2 = -N;
    while &z >= two128 {
        z >>= 1;
        e2 += 1;
    }
    assert!(z.bits() == 128, "Invalid representation of M128: wrong number of bits for 10^{}: {}!", e10, z.bits());

    // Check validity of exponent
    let approx_n = ((Wrapping(217706u64) * Wrapping(e10 as u64)).0 >> 16) + 1087;
    let approx_n = approx_n as u32;
    let bias_n = e2 + BIAS;
    assert!(approx_n == bias_n.try_into().unwrap(), "Approxmiated exponent {} does not match biased exponent {}!", approx_n, bias_n);


    let digits = z.iter_u64_digits().collect::<Vec<_>>();
    let m128_lo = digits[0];
    let m128_hi = digits[1];
    let widebiased_e2 = bias_n;

    LUTEntry {
        m128_hi,
        m128_lo,
        widebiased_e2
    }
}

/// Formats a generated lookup table into a string that can be included into 
/// Rust source code as a static LUT. Also generates appropriate methods.
fn format_lookup_table(luts: Vec<LUTEntry>, min_exponent: i16) -> String {
    let mut lines = Vec::new();

    // Generate constants
    lines.push(format!("const EL_POW10_LUT_MIN: i16 = {};", min_exponent));
    lines.push(format!("const EL_POW10_LUT: [(u64, u64, u16); {}] = [", luts.len()));
    for entry in luts {
        lines.push(format!("({:#x}, {:#x}, {}), // pow2 = {} ", entry.m128_hi, entry.m128_lo, entry.widebiased_e2, entry.widebiased_e2 - BIAS));
    }
    lines.push("];".to_string());

    return lines.join("\n");
}

fn main(){
    println!("cargo:rerun-if-changed=build.rs");
    let min_exponent = -307;
    let max_exponent = 288;
    let table = gen_lookup_table(min_exponent, max_exponent);
    let lut_str = format_lookup_table(table, min_exponent);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = path::Path::new(&out_dir).join("el_lookup_table.rs");
    let mut f = File::create(dest_path).expect("Could not create LUT output file.");
    f.write_all(lut_str.as_bytes()).unwrap();
}