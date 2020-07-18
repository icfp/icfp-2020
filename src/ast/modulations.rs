use super::{Number, Symbol};

pub type Modulated = Vec<bool>;

mod modulate_constants {
    pub(super) const ZERO: [bool; 3] = [false, true, false];
    pub(super) const SIGN_POSITIVE: [bool; 2] = [false, true];
    pub(super) const SIGN_NEGATIVE: [bool; 2] = [true, false];
}

///
/// Bits 0..1 define a positive or negative number (and signal width) via a high/low or low/high signal change:
//  01: positive number
//  10: negative number
//
// Bits 2..(n+2) define the width of the following binary-encoded number via a unary-encoded number
// of length n composed of high signals ending with a low signal.
// The number width (in bits) is four times the unary encoding (i.e. 4 * n):
//
//  0: 0 [i.e. the number zero]
//  10: 4-bit number [i.e. 1-15]
//  110: 8-bit number [i.e. 1-255]
//  1110: 12-bit number [i.e. 1-4095]
//
// …
//
// The remaining bits, i.e. (n + 3)..(n + 3 + 4*n - 1), determine the number itself,
// in most-significant-bit first binary notation. Using the examples from this message:
//  0001: 1 <- 4 (4*1)
//  00010000: 16 <- 8 (4*2)
//  000100000000: 256 <- 12 (4*3)
//
fn modulate_number(value: Number) -> Modulated {
    if value == 0 {
        return modulate_constants::ZERO.to_vec();
    }

    fn log_2(x: Number) -> u32 {
        const fn num_bits<T>() -> usize {
            std::mem::size_of::<T>() * 8
        }

        assert!(x > 0);
        num_bits::<Number>() as u32 - x.leading_zeros() - 1
    }

    let mut bits: Vec<bool> = Vec::new();

    if value > 0 {
        bits.extend_from_slice(&modulate_constants::SIGN_POSITIVE);
    } else {
        bits.extend_from_slice(&modulate_constants::SIGN_NEGATIVE);
    }

    let value = value.abs();

    let number_of_bits_for_number = log_2(value) + 1;

    let remainder = if number_of_bits_for_number % 4 != 0 {
        1
    } else {
        0
    };

    let width = (number_of_bits_for_number / 4 + remainder) as usize * 4;

    let ones = vec![true].repeat(width / 4);
    bits.extend_from_slice(&ones);
    bits.push(false);

    if width > 0 {
        let encoded = format!("{:0>width$b}", value, width = width);
        let encoded: Vec<bool> = encoded.bytes().map(|b| b == b'1').collect();
        bits.extend_from_slice(&encoded);
    }

    return bits;
}

pub fn modulate(value: &Symbol) -> Symbol {
    Symbol::Modulated(match value {
        Symbol::Lit(number) => modulate_number(*number),
        _ => unimplemented!("Not implemented for {:?} yet", value),
    })
}

pub fn demodulate(value: Modulated) -> Symbol {
    let slice = value.as_slice();

    let sign = &slice[0..2];

    match sign {
        [true, false] | [false, true] => {
            let sign = if sign == &modulate_constants::SIGN_POSITIVE {
                1
            } else {
                -1
            };

            let width = slice[2..].iter().take_while(|&&b| b).count();

            if width == 0 {
                return Symbol::Lit(0);
            }

            let start = 2 + width + 1;

            let parsed_value = slice[start..(start + (width * 4))]
                .iter()
                .fold(0i64, |num, bit| num << 1 | if *bit { 1 } else { 0 });

            let parsed_value = sign * parsed_value;

            Symbol::Lit(parsed_value)
        }
        [true, true] => unimplemented!("Modulate List not implemented"),
        [false, false] => Symbol::Nil,
        _ => unreachable!("Invalid modulation"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Symbol::*, *};

    #[test]
    fn test_modulate_logic() {
        fn val(s: &str) -> super::Modulated {
            s.bytes().map(|b| b == b'1').collect()
        }

        assert_eq!(modulate_number(0), val("010"));
        assert_eq!(modulate_number(1), val("01100001"));
        assert_eq!(modulate_number(-1), val("10100001"));
        assert_eq!(modulate_number(256), val("011110000100000000"));

        assert_eq!(modulate(&Lit(0)), Modulated(val("010")));
        assert_eq!(modulate(&Lit(1)), Modulated(val("01100001")));
        assert_eq!(modulate(&Lit(-1)), Modulated(val("10100001")));
        assert_eq!(modulate(&Lit(256)), Modulated(val("011110000100000000")));
    }

    #[test]
    fn test_demodulate_logic() {
        assert_eq!(demodulate(modulate_number(0)), Lit(0));
        assert_eq!(demodulate(modulate_number(1)), Lit(1));
        assert_eq!(demodulate(modulate_number(-1)), Lit(-1));
        assert_eq!(demodulate(modulate_number(256)), Lit(256));

        use Symbol::Lit;
        assert_eq!(demodulate(modulate_number(0)), Lit(0));
        assert_eq!(demodulate(modulate_number(1)), Lit(1));
        assert_eq!(demodulate(modulate_number(-1)), Lit(-1));
        assert_eq!(demodulate(modulate_number(256)), Lit(256));
    }
}
