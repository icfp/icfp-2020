use super::{Number, Symbol};

pub type Modulated = Vec<bool>;

mod modulate_constants {
    pub(super) const MODULATED_LIST: [bool; 2] = [true, true];
    pub(super) const NIL: [bool; 2] = [false, false];
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

pub fn modulate(value: &Symbol) -> Modulated {
    match value {
        Symbol::Lit(number) => modulate_number(*number),
        Symbol::Nil => modulate_constants::NIL.to_vec(),
        Symbol::List(symbols) => {
            let mut vec = symbols.iter().fold(
                modulate_constants::MODULATED_LIST.to_vec(),
                |mut vec, symbol| {
                    vec.append(&mut modulate(symbol));
                    vec
                },
            );
            vec.extend_from_slice(&modulate_constants::NIL);
            vec
        }
        Symbol::Pair(left, right) => {
            let mut vec = modulate_constants::MODULATED_LIST.to_vec();
            vec.extend_from_slice(&modulate(&left));
            vec.extend_from_slice(&modulate(&right));
            vec
        }
        _ => unimplemented!("Not implemented for {:?} yet", value),
    }
}

pub fn demodulate(value: Modulated) -> Symbol {
    fn demodulate_number(sign: Number, slice: &[bool]) -> (usize, Symbol) {
        let width = slice.iter().take_while(|&&b| b).count();

        if width == 0 {
            return (1, Symbol::Lit(0));
        }

        let width_bits = width + 1;
        let bit_size = width_bits + (width * 4);

        let parsed_value = slice[width_bits..bit_size]
            .iter()
            .fold(0i64, |num, bit| num << 1 | if *bit { 1 } else { 0 });

        let parsed_value = sign * parsed_value;

        (bit_size, Symbol::Lit(parsed_value))
    }

    fn demodulate_slice(slice: &[bool]) -> (usize, Symbol) {
        let prefix = &slice[0..2];

        match prefix {
            [true, false] | [false, true] => {
                let sign = if prefix == &modulate_constants::SIGN_POSITIVE {
                    1
                } else {
                    -1
                };
                let slice = &slice[2..]; // move past prefix

                let (size, symbol) = demodulate_number(sign, slice);
                (size + 2, symbol)
            }
            [true, true] => {
                let slice = &slice[2..]; // move past prefix

                let (first_size, first_symbol) = demodulate_slice(slice);

                let slice = &slice[first_size..]; // move past prefix
                let (second_size, second_symbol) = demodulate_slice(slice);

                (
                    first_size + second_size,
                    Symbol::Pair(first_symbol.into(), second_symbol.into()),
                )
            }
            [false, false] => (2, Symbol::Nil),
            _ => unreachable!("Invalid modulation"),
        }
    }

    let slice = value.as_slice();
    demodulate_slice(slice).1
}

pub fn demodulate_string(s: &str) -> Symbol {
    demodulate(s.bytes().map(|b| b == b'1').collect())
}

pub fn modulate_to_string(symbol: &Symbol) -> String {
    modulate(symbol)
        .iter()
        .map(|&b| if b { "1" } else { "0" })
        .collect::<Vec<&str>>()
        .join("")
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

        assert_eq!(modulate(&Lit(0)), val("010"));
        assert_eq!(modulate(&Lit(1)), val("01100001"));
        assert_eq!(modulate(&Lit(-1)), val("10100001"));
        assert_eq!(modulate(&Lit(256)), val("011110000100000000"));
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

    #[test]
    fn modulate_list_roundtrip() {
        assert_eq!(
            modulate_to_string(&List(vec![Lit(1)])),
            modulate_to_string(&Pair(Lit(1).into(), Nil.into()))
        );
    }

    #[test]
    fn test_demodulate_list() {
        assert_eq!(
            demodulate(vec![
                true, true, // ModulatedList
                false, true, // Positive Int
                true, false, // Width = 4 * 1 (one true)
                false, false, false, true, // One
                false, true, // Positive Int
                true, false, // Width = 4 * 1
                false, false, true, false // Two
            ]),
            Pair(Lit(1).into(), Lit(2).into())
        )
    }

    #[test]
    fn http_responses() {
        assert_eq!(
            demodulate_string("1101000"),
            Pair(Lit(0).into(), Nil.into())
        );

        // 11 - list
        // 01 - +int
        // 10 - 1 width (4 bits)
        // 0001 - one
        // 11 - list
        // 01 - +int
        // 11110 - 4 width 16 bits (4*4)
        // 1111011100101010
        // 00 - Nil
        let response = demodulate_string("1101100001110111110110100111100011000");
        assert_eq!(
            response,
            Pair(Lit(1).into(), Pair(Lit(54214).into(), Nil.into()).into())
        );

        let inc = super::super::eval_instructions(&[
            Symbol::Ap,
            Symbol::Inc,
            Symbol::Ap,
            Symbol::Car,
            response,
        ]);

        dbg!(modulate_to_string(&inc));
    }
}
