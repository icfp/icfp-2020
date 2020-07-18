use super::{Number, Symbol};

mod modulate_constants {
    pub(super) const ZERO: &str = "010";
    pub(super) const SIGN_POSITIVE: &str = "01";
    pub(super) const SIGN_NEGATIVE: &str = "10";
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
fn modulate_to_string(value: Number) -> String {
    if value == 0 {
        return modulate_constants::ZERO.to_string();
    }

    fn log_2(x: Number) -> u32 {
        const fn num_bits<T>() -> usize {
            std::mem::size_of::<T>() * 8
        }

        assert!(x > 0);
        num_bits::<Number>() as u32 - x.leading_zeros() - 1
    }

    let mut bits: Vec<&str> = Vec::new();

    if value > 0 {
        bits.push(modulate_constants::SIGN_POSITIVE);
    } else {
        bits.push(modulate_constants::SIGN_NEGATIVE);
    }

    let value = value.abs();

    let number_of_bits_for_number = log_2(value) + 1;

    let remainder = if number_of_bits_for_number % 4 != 0 {
        1
    } else {
        0
    };

    let width = (number_of_bits_for_number / 4 + remainder) as usize * 4;

    let ones = "1".repeat(width / 4);
    bits.push(ones.as_str());
    bits.push("0");

    let encoded = format!("{:0>width$b}", value, width = width);
    if width > 0 {
        bits.push(encoded.as_str());
    }

    return bits.join("");
}

pub fn modulate(value: Number) -> Symbol {
    return Symbol::StringValue(modulate_to_string(value));
}

fn demodulate_number(value: String) -> Number {
    let slice = value.as_str();

    let sign = &slice[0..2];
    let sign = if sign == modulate_constants::SIGN_POSITIVE {
        1
    } else {
        -1
    };

    let width = slice[2..].chars().take_while(|c| c == &'1').count();

    if width == 0 {
        return 0;
    }

    let start = 2 + width + 1;

    let binary: &str = &slice[start..(start + (width * 4))];

    let parsed_value = Number::from_str_radix(binary, 2).expect("Unable to parse base 2 number");

    let parsed_value = sign * parsed_value;

    return parsed_value;
}

pub fn demodulate(value: String) -> Symbol {
    return Symbol::Lit(demodulate_number(value));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulate_logic() {
        assert_eq!(modulate_to_string(0), "010");
        assert_eq!(modulate_to_string(1), "01100001");
        assert_eq!(modulate_to_string(-1), "10100001");
        assert_eq!(modulate_to_string(256), "011110000100000000");

        fn val(s: &str) -> Symbol {
            Symbol::StringValue(s.to_string())
        }

        assert_eq!(modulate(0), val("010"));
        assert_eq!(modulate(1), val("01100001"));
        assert_eq!(modulate(-1), val("10100001"));
        assert_eq!(modulate(256), val("011110000100000000"));
    }

    #[test]
    fn test_demodulate_logic() {
        assert_eq!(demodulate_number(modulate_to_string(0)), 0);
        assert_eq!(demodulate_number(modulate_to_string(1)), 1);
        assert_eq!(demodulate_number(modulate_to_string(-1)), -1);
        assert_eq!(demodulate_number(modulate_to_string(256)), 256);

        use Symbol::Lit;
        assert_eq!(demodulate(modulate_to_string(0)), Lit(0));
        assert_eq!(demodulate(modulate_to_string(1)), Lit(1));
        assert_eq!(demodulate(modulate_to_string(-1)), Lit(-1));
        assert_eq!(demodulate(modulate_to_string(256)), Lit(256));
    }
}
