use super::{Number, Symbol};

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
pub fn modulate_to_string(value: Number) -> String {
    if value == 0 {
        return "010".to_string();
    }

    fn log_2(x: Number) -> u32 {
        const fn num_bits<T>() -> usize {
            std::mem::size_of::<T>() * 8
        }

        assert!(x > 0);
        num_bits::<Number>() as u32 - x.leading_zeros() - 1
    }

    let mut bits: Vec<&str> = Vec::new();

    if value >= 0 {
        bits.push("01");
    } else {
        bits.push("10");
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
}
