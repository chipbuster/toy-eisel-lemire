use std::{convert::TryInto};


pub fn parse_float(x: &str) -> Result<f64, std::num::ParseFloatError> {
    parse_float_with_fallback(x)
}

fn parse_float_with_fallback(x: &str) -> Result<f64, std::num::ParseFloatError> {
    let z = parse_float_internal(x);
    match z {
        Some(f) => Ok(f),
        None => x.parse(),
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
struct ManExp10 {
    neg: bool,
    man: u64,
    e10: i16,
}

/// The Eisel-Lemire float-parsing algorithm. If the result is None, the caller
/// should invoke the fallback algorithm.
/* We use the syntax for float literals described at
https://doc.rust-lang.org/stable/reference/tokens.html#floating-point-literals
*/
fn parse_float_internal(input: &str) -> Option<f64> {
    // Step 1: split string into a mantissa and exponent
    let ManExp10 { neg, man, e10 } = parse_man_exp10(input)?;

    // Check zero mantissa
    if man == 0 {
        return Some(0.0);
    }
    // check exp10 range

    // Perform mantissa normalization

    unimplemented!()
}

/*
  This is the first-stage parsing algorithm. Most of the quirks of the floating
  point literal format are dealt with in this function, so we'll document some
  assumptions here. A floating point literal is:

  - A decimal literal followed by a period character U+002E (.). This is
    optionally followed by another decimal literal, with an optional exponent.
  - A single decimal literal followed by an exponent.

  Exponent can either be "e" or "E". Note that any number of underscores `_` are
  allowed in `DEC_LITERAL`s.

  We ignore the floating-point suffix and assume that all literals are to be
  parsed as f64.
*/
fn parse_man_exp10(input: &str) -> Option<ManExp10> {
    let mut inp_iter = input.chars();

    let neg = parse_parts::parse_leading_sign(&mut inp_iter)?;
    let (man, man_exp10, has_exp) = parse_parts::parse_mantissa_base10(&mut inp_iter)?;
    let explicit_exp10 = if has_exp {
        parse_parts::parse_exp10(&mut inp_iter)?
    } else {
        0i16
    };

    let exp10 = man_exp10 + explicit_exp10;
    Some(ManExp10{
        neg, man, e10: exp10
    })
}

mod parse_parts {
use std::str::Chars;
use std::convert::TryInto;

/// Parses the sign of the number (true for negative), advancing the input
/// iterator to the appropriate next point. Returns None if the given stream
/// is unparseable at the current location.
pub fn parse_leading_sign(inp_iter: &mut Chars) -> Option<bool> {
    let my_itr = inp_iter.clone();
    let first_char = *my_itr.peekable().peek()?;
    if ['+', '-'].contains(&first_char) {
        let is_neg_sym = first_char == '-';
        // Advance cur_char to non-sign input. If we get no input, it's not a valid float literal.
        let c = inp_iter.next().unwrap();
        assert!(c == '-' || c == '+');
        Some(is_neg_sym)
    } else {
        Some(false)
    }
}


/// Returns a (u64, i16) pair such that u64 * 10 ** i16 = mantissa
/// Returns None if this input is unparseable, or if the mantissa is longer than 19 digits
pub fn parse_mantissa_base10(inp_iter: &mut Chars) -> Option<(u64, i16, bool)> {
    let mut cur_char = inp_iter.next();

    // Parse the mantissa
    let mut decimal_seen = false;
    let mut digits = 0i16;
    let mut digits_pre_decimal = 0i16;
    let mut mantissa = 0u64;  // Must be 64bit to handle at least 19 decimals
    let mut has_exponent = false;

    while digits < 20 && cur_char.is_some() {
      let c = cur_char.unwrap();
      match c {
        '_' => { 
          // Do nothing: we pretend this character doesn't exist
        },
        '.' => {
          if decimal_seen {
            return None;  // Seeing two decimal in a floating point
          }
          decimal_seen = true;
        }
        'e' | 'E' => {
          // Mantissa is done: this is the start of the exponent
          has_exponent = true;
          break;
        }
        '0'..='9' => {
          mantissa *= 10;
          let d: u64 = c.to_digit(10)?.into();
          mantissa += d;

          digits += 1;
          if !decimal_seen{
              digits_pre_decimal += 1;
          }
        }
        _ => {
          return None; // Non-decimal digit encountered
        }
      };
      cur_char = inp_iter.next();
    }

    // mantissa overflow--revert to fallback
    if digits >= 20 {
        return None
    }

    Some((mantissa, digits_pre_decimal - digits, has_exponent))
}

/// Parses an exponent starting AFTER `e` or `E`.
pub fn parse_exp10(inp_iter: &mut Chars) -> Option<i16> {
    let mut neg = false;
    let mut exp10 = 0i64;

    let mut c = inp_iter.next()?;
    if ['+','-'].contains(&c){
        neg = c == '-';
        c = inp_iter.next()?;
    }

    exp10 = c.to_digit(10)?.into();
    while let Some(c) = inp_iter.next() {
        if c == '_' {
            continue
        }
        exp10 = exp10.checked_mul(10)?;
        let d: i64 = c.to_digit(10)?.into();
        exp10 = exp10.checked_add(d)?;
    }
    let mut exp10: i16 = exp10.try_into().ok()?;
    if neg { exp10 = exp10.checked_mul(-1)? }
    Some(exp10)
}

}

#[cfg(test)]
pub mod tests {
    use crate::elparse::parse_parts::{parse_exp10, parse_leading_sign};

    use super::*;
    use rand::random;

    #[test]
    fn check_parse_exp10(){
        let inputs = vec!["-2639", "+173", "0_00___0", "0+0_0", "999999", ""];
        let outputs = vec![Some(-2639i16), Some(173), Some(0), None, None, None];
        for (i, o) in inputs.into_iter().zip(outputs.into_iter()){
            let testout = parse_exp10(&mut i.chars());
            assert_eq!(testout, o, "Input {} should parse to {:?} but got {:?}", i, o, testout)
        }
    }

    #[test]
    fn check_parse_sign_net(){
        let inputs = vec!["-3","+7","00","90","-",""];
        let outputs = [Some(true), Some(false), Some(false), Some(false), Some(true), None];
        let nexts = [Some('3'),Some('7'),Some('0'),Some('9'), None, None];
        for ((i, o),n) in inputs.into_iter().zip(outputs.into_iter()).zip(nexts.into_iter()){
            let mut itr = i.chars();
            let testout = parse_leading_sign(&mut itr);
            assert_eq!(testout, o.clone(), "Parsing sign of {} should have given {:?} but gave {:?}",i,o,testout);
            let testnext = itr.next();
            assert_eq!(testnext, n.clone(), "Parsing {} should have left {:?} as next in stream, but got {:?}", i, n, testnext )
        }
    }

    #[test]
    fn check_parse_mantissa(){
        unimplemented!()
        // TODO: Finish writing these tests to test parse_mantissa_base10
        let inputs = ["123.45e10"];
    }
    

}
