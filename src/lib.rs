#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[cfg(test)]
mod alphabet_choice;

use alloc::string::String;
use alloc::vec::Vec;

use core::cmp::min;

const ALPHABET: &[u8] = b"123456789BCEFHJLMNOPQRUWXYadktvz";

//       0  1  2  3  4  5  6  7
//
// +0    1  2  3  4  5  6  7  8
// +8    9  B  C  E  F  H  J  L
// +16   M  N  O  P  Q  R  U  W
// +24   X  Y  a  d  k  t  v  z

/*   ASCII from byte 48 to byte 122 inclusive
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9,  :,  ;,  <,  =,  >,  ?,  @,  A,  B,  C,
     D,  E,  F,  G,  H,  I,  J,  K,  L,  M,  N,  O,  P,  Q,  R,  S,  T,  U,  V,  W,
     X,  Y,  Z,  [,  \,  ],  ^,  _,  `,  a,  b,  c,  d,  e,  f,  g,  h,  i,  j,  k,
     l,  m,  n,  o,  p,  q,  r,  s,  t,  u,  v,  w,  x,  y,  z,
*/

#[rustfmt::skip]
const INVERSE_STRICT: [i8; 75] = [
    -1,  0,  1,  2,  3,  4,  5,  6,  7,  8, -1, -1, -1, -1, -1, -1, -1, -1,  9, 10,
    -1, 11, 12, -1, 13, -1, 14, -1, 15, 16, 17, 18, 19, 20, 21, -1, -1, 22, -1, 23,
    24, 25, -1, -1, -1, -1, -1, -1, -1, 26, -1, -1, 27, -1, -1, -1, -1, -1, -1, 28,
    -1, -1, -1, -1, -1, -1, -1, -1, 29, -1, 30, -1, -1, -1, 31,
];

/// This inverse function allows out-of-alphabet characters that might be mistaken for
/// in-alphabet characters, to be automatically mapped to the in-alphabet character
/// they look like. Here are correction mappings (wrong,right):
///   (A,4), (B,8), (I,1), (J,j), (K,k), (M,m), (P,p), (S,5), (W,w),
///   (Y,7), (Z,2), (b,6), (i,1), (l,1), (r,R), (S,5), (x,X), (z,2)
#[rustfmt::skip]
const INVERSE_CORRECTED: [i8; 75] = [
    18,  0,  1,  2,  3,  4,  5,  6,  7,  8, -1, -1, -1, -1, -1, -1, -1, 26,  9, 10,
    27, 11, 12,  5, 13,  0, 14, 28, 15, 16, 17, 18, 19, 20, 21,  4, 29, 22, 30, 23,
    24, 25, 31, -1, -1, -1, -1, -1, -1, 26,  9, 10, 27, 11, 12,  8, 13,  0, 14, 28,
    15, 16, 17, 18, 19, 20, 21,  4, 29, 22, 30, 23, 24, 25, 31,
];

// This seed was just made up. We should test seeds like bech32 did to see which one
// gives us the best error detection.
const XXH32_SEED: u32 = 0x61ccf743;

/// Encode binary data into hum32 format, optionally specifing
/// a prefix representing the type of data encoded
pub fn encode(plain: &[u8], prefix: Option<&str>) -> Result<String, Error> {
    // First compute the xxHash xxh32 checksum on the data
    let checksum: u32 = xxhash_rust::xxh32::xxh32(plain, XXH32_SEED);
    let checksum_le = checksum.to_le_bytes();

    // Chain the data with this 4 byte checksum (represented in little-endian)
    let input = [plain, checksum_le.as_slice()].concat();

    let prefix_len = match prefix {
        Some(p) => {
            if !p.is_ascii() {
                return Err(Error::NotAscii);
            }
            p.len() + 1
        }
        None => 0,
    };

    let mut output: Vec<u8> = Vec::with_capacity(prefix_len + input.len().div_ceil(4) * 5);

    if let Some(p) = prefix {
        output.extend(p.as_bytes());
        output.push(b'0');
    }

    for chunk in input.chunks(5) {
        let buf = {
            let mut buf = [0u8; 5];
            for (i, &b) in chunk.iter().enumerate() {
                buf[i] = b;
            }
            buf
        };
        output.push(ALPHABET[((buf[0] & 0xF8) >> 3) as usize]);
        output.push(ALPHABET[(((buf[0] & 0x07) << 2) | ((buf[1] & 0xC0) >> 6)) as usize]);
        output.push(ALPHABET[((buf[1] & 0x3E) >> 1) as usize]);
        output.push(ALPHABET[(((buf[1] & 0x01) << 4) | ((buf[2] & 0xF0) >> 4)) as usize]);
        output.push(ALPHABET[(((buf[2] & 0x0F) << 1) | (buf[3] >> 7)) as usize]);
        output.push(ALPHABET[((buf[3] & 0x7C) >> 2) as usize]);
        output.push(ALPHABET[(((buf[3] & 0x03) << 3) | ((buf[4] & 0xE0) >> 5)) as usize]);
        output.push(ALPHABET[(buf[4] & 0x1F) as usize]);
    }

    if input.len() % 5 != 0 {
        let len = output.len();
        let num_extra = 8 - (input.len() % 5 * 8).div_ceil(5);
        output.truncate(len - num_extra);
    }

    Ok(String::from_utf8(output).unwrap())
}

/// Get the prefix, if it has a 0-separated prefix
pub fn prefix(coded: &[u8]) -> Option<&[u8]> {
    coded
        .iter()
        .position(|c| *c == b'0')
        .map(|sep| &coded[0..sep])
}

/// Decode hum32 back into binary. If the input has a substitution errors from letters
/// outside of the alphabet, some of these these can be tolerated with strict=false.
///
/// Returns an error if some character outside of the alphabet was found and either strict
/// is true, or the character doesn't have an obvious replacement inside the alphabet.
/// Also returns error if the checksum did not match.
pub fn decode(coded: &str, strict: bool) -> Result<Vec<u8>, Error> {
    if !coded.is_ascii() {
        return Err(Error::NotAscii);
    }

    let alphabet = if strict {
        INVERSE_STRICT
    } else {
        INVERSE_CORRECTED
    };

    let start = match coded.chars().position(|c| c == '0') {
        Some(p) => p + 1,
        None => 0,
    };

    let data = &coded.as_bytes()[start..];

    let mut unpadded_data_length = data.len();
    for i in 1..min(6, data.len()) + 1 {
        if data[data.len() - i] != b'=' {
            break;
        }
        unpadded_data_length -= 1;
    }
    let output_length = unpadded_data_length * 5 / 8;
    let mut ret = Vec::with_capacity(output_length.div_ceil(5) * 5);
    for chunk in data.chunks(8) {
        let buf = {
            let mut buf = [0u8; 8];
            for (i, &c) in chunk.iter().enumerate() {
                match alphabet.get(c.wrapping_sub(b'0') as usize) {
                    Some(&-1) | None => return Err(Error::InvalidCharacter(c as char)),
                    Some(&value) => buf[i] = value as u8,
                };
            }
            buf
        };
        ret.push((buf[0] << 3) | (buf[1] >> 2));
        ret.push((buf[1] << 6) | (buf[2] << 1) | (buf[3] >> 4));
        ret.push((buf[3] << 4) | (buf[4] >> 1));
        ret.push((buf[4] << 7) | (buf[5] << 2) | (buf[6] >> 3));
        ret.push((buf[6] << 5) | buf[7]);
    }
    ret.truncate(output_length);

    // SKIP the checksum momentarily...
    // Verify the checksum (last 4 bytes)
    let checksum: u32 = xxhash_rust::xxh32::xxh32(&ret[..ret.len() - 4], XXH32_SEED);
    let checksum_le = checksum.to_le_bytes();
    if *checksum_le.as_slice() != ret[ret.len() - 4..] {
        return Err(Error::InvalidChecksum);
    }

    ret.truncate(ret.len() - 4);
    Ok(ret)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    NotAscii,
    InvalidCharacter(char),
    InvalidChecksum,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_prefix() {
        assert_eq!(prefix(b"mopub0lkj234"), Some(b"mopub".as_slice()));
        assert_eq!(
            prefix(b"thisisalonger12345string0mopub0lkj234"),
            Some(b"thisisalonger12345string".as_slice())
        );
    }

    #[test]
    fn test_every_byte() {
        // A vector of all bytes
        let mut input: [u8; 256] = [0; 256];
        for i in 0..256 {
            input[i] = i as u8;
        }

        let encoded = encode(input.as_slice(), Some("bytes")).unwrap();
        println!("{}", encoded);
        let decoded = decode(&*encoded, true).unwrap();
        assert_eq!(input.as_slice(), &*decoded);
    }

    #[test]
    fn test_vectors() {
        let vectors = [
            (b"".as_slice(), "UOBHU41"),
            (
                b"The quick brown fox jumps over the lazy dog.".as_slice(),
                "CNQ7C94NJRQU7aY1FBY7vtdJ52P7vv21HBaUak4P52WWFYEO52a7MYB1HNMWQvB1FNWUJEM4MkXHU",
            ),
            (b"GM".as_slice(), "9t7XtkaHPk"),
            (&[246, 11, 226, 142, 73, 141, 43, 201, 119, 153, 142, 112, 11, 216, 255, 247,
              149, 36, 188, 231, 3, 176, 115, 77, 88, 172, 174, 148, 25, 78, 190, 236],
             "vX6v64OBNQRkOtkYNYX1WU8zvvCOBL881JX87PCXROWB97CJWdUJPN2JkQ"),
        ];

        for (input, answer) in vectors.iter() {
            let encoded = encode(*input, None).unwrap();

            // Verify it is what we expect
            assert_eq!(&encoded, *answer);

            // Verify that it decodes
            assert_eq!(*input, decode(&encoded, true).unwrap());
        }
    }

    #[test]
    fn test_corrections() {
        // actual = "vX6v64OBNQRkOtkYNYX1WU8zvvCOBL881JX87PCXROWB97CJWdUJPN2JkQ";
        let wrongcase = "Vx6V64obnqrKoTKynyx1wu8ZVVcobl881jx87pcxrowb97cjwDujpn2jKq";

        assert_eq!(
            [246, 11, 226, 142, 73, 141, 43, 201, 119, 153, 142, 112, 11, 216, 255, 247,
             149, 36, 188, 231, 3, 176, 115, 77, 88, 172, 174, 148, 25, 78, 190, 236].as_slice(),
            decode(wrongcase, false).unwrap()
        );
        // 0 -> O
        // G -> 6 <0-00 no 6
        // S -> 5
        // i -> 1
        // s -> 5


        //     actual = "prefix0CtQ7CdN1H6W31t49FQM77ddRJBYUC94LFXM7MtEHF6W31YEUFRW89kY1L6WWC94JFROU994252W7CtY17QUU5aEQ52OUkXdLFNQUkYY1JHNUMYEHFQU31vELJQM7QtEPJMM7aXEEFQM7vdP66Xa5CUUk"
        let wrongsubs = "prefix0CtQ7CdNiHGW3it49FQM77ddRJBYUC94LFXM7MtEHFGW3iYEUFRW89kYiLGWWC94JFR0U9942S2W7CtYi7QUUsaEQS20UkXdLFNQUkYYiJHNUMYEHFQU3ivELJQM7QtEPJMM7aXEEFQM7vdPGGXasCUUk";

        assert_eq!(
            b"When in the course of human events you need a new 5-bit encoding scheme, you just make one.".as_slice(),
            decode(wrongsubs, false).unwrap()
        );
    }
}
