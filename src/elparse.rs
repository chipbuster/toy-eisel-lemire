use std::convert::TryInto;

pub fn parse_float(x: &str) -> Result<f64, std::num::ParseFloatError>{
    parse_float_with_fallback(x)
}

fn parse_float_with_fallback(x: &str) -> Result<f64, std::num::ParseFloatError> {
    let z= parse_float_internal(x);
    match z {
        Some(f) => Ok(f),
        None => x.parse()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Digits {
  neg: bool,
  digits: Vec<u8>
}

/// The Eisel-Lemire float-parsing algorithm. If the result is None, have the 
/// caller invoke the fallback algorithm.
/// 
/* We use the syntax for float literals described at 
https://doc.rust-lang.org/stable/reference/tokens.html#floating-point-literals:

  - A decimal literal followed by a period character U+002E (.). This is 
    optionally followed by another decimal literal, with an optional exponent.
  - A single decimal literal followed by an exponent.

  Exponent can either be "e" or "E".
*/
fn parse_float_internal(input: &str) -> Option<f64> {
    // Step 1: split string into a mantissa and exponent
    let mantissa = 5;

    // Check zero mantissa
    // Check mantissa range
    // check exp10 range

    // Perform mantissa normalization


    unimplemented!()
}

fn parse_digits(input: &str) -> Option<Digits> {
  let mut sgn_seen = false;
  let mut neg = false;
  let mut point_seen = false;
  let mut digits = Vec::new();
  let mut chars = input.chars().peekable();

  // Examine for negative sign.
  if chars.peek() == Some(&'-') || chars.peek() == Some(&'+') {
    let sgn = chars.next().unwrap();
    neg = sgn == '-';
  }

  for c in chars{
    match c {
      '.' => {
        if point_seen {return None};
        point_seen = true; 
        digits.push(11);
      }
      d@ '0'..='9' => {
        let d = d.to_digit(10).unwrap().try_into().unwrap();
        digits.push(d);
      }
      _ => return None
    }
  }
  Some(Digits{
    neg,
    digits
  })
}

fn split_mantissa_exponent(input: &str) -> Option<(Digits, Digits)> {
  let parts = input.split(&['e','E'][..]).collect::<Vec<_>>();
  let (mantissa, exponent) = if parts.len() == 2 {
    (parse_digits(parts[0]), parse_digits(parts[1]))
  } else {
    (parse_digits(input), Some(Digits{ neg: false, digits: Vec::new()}))
  };

  if mantissa.is_some() && exponent.is_some() {
    Some((mantissa.unwrap(), exponent.unwrap()))
  } else {
    None
  }
}

#[cfg(test)]
pub mod tests {

use super::*;
  use rand::random;

  fn digit_to_u8(x: char) -> u8{
    match x {
      '0'..='9' => x.to_digit(10).unwrap().try_into().unwrap(),
      '.' => 11,
      _ => panic!()
    }
  }

  fn roundtrip_value(val: f64){
    let str_val = &val.to_string()[..];
    let str_val_no_sgn = if val < 0.0 {
      &str_val[1..]
    } else {
      str_val
    };
    let true_digits = Digits {
      neg: val < 0.0,
      digits: str_val_no_sgn.chars().map(digit_to_u8).collect()
    };
    let rt_digits = parse_digits(str_val);
    assert!(&rt_digits.is_some(), "Failed to parse {}", val);
    assert_eq!(*rt_digits.as_ref().unwrap(), true_digits, "Round-tripping {} resulted in {:?}", str_val, rt_digits)
  }

  #[test]
  fn roundtrip_randoms(){
    for _ in 0..100 {
      let val = 10000.0 * (random::<f64>() - 0.5);
      roundtrip_value(val);
    }
  }

  #[test]
  fn parse_digits_1(){
    //let inp = "2.74322964";
    let inp = "-2076.9421866556927";
    let out = parse_digits(inp);
    let canon = Digits{
      neg: true,
      digits: vec![2u8, 0, 7, 6, 11, 9, 4, 2, 1, 8, 6, 6, 5, 5, 6, 9, 2, 7]
    };
    assert_eq!(out.unwrap(), canon);
  }

  #[test]
  fn parse_digits_fail_multiple_decimals(){
    let inp = "2.713.283";
    let out = parse_digits(inp);
    assert!(out.is_none(), "Library successfully parsed {}", inp);
  }

  #[test]
  fn parse_digits_fail_multiple_signs(){
    let inp = "--2713283";
    let out = parse_digits(inp);
    assert!(out.is_none(), "Library successfully parsed {}", inp);
  }
  
  #[test]
  fn parse_digits_fail_sign_embedded(){
    let inp = "2.9302-123";
    let out = parse_digits(inp);
    assert!(out.is_none(), "Library successfully parsed {}", inp);
  }
}