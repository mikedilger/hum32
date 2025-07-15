#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::min;

/*
 * The hum32 alphabet was chosen from numbers and both uppercase and lowercase letters, rather
 * than sticking to a single case as most others do (base32, zbase32, bech32). This lets us
 * make the symbols more unique.
 *
 * We avoid more than 1 character from any set of visually similar looking characters. We also
 * avoid similarity groups larger than 2, so that we can map wrong characters to their right
 * character when doing correction.
 *
 * These are the identified character-similarity groups:
 *
 * (a,o,O,0), (A,4), (5,S,s), (b,6), (B,8), (z,Z,2), (i,I,l,1), (u,U,v,V), (r,n), (r,v), (c,C),
 * (j,J), (k,K), (m,M), (p,P), (s,S), (u,U), (v,V), (w,W), (x,X), (y,Y), (z,Z)
 */

const ALPHABET: &[u8] = b"cQ8dEH41kDgqFN2LX69y5hRwepG73mTj";

//       0  1  2  3  4  5  6  7
//
// +0    c  Q  8  d  E  H  4  1
// +8    k  D  g  q  F  N  2  L
// +16   X  6  9  y  5  h  R  w
// +24   e  p  G  7  3  m  T  j

/*   ASCII from byte 48 to byte 122 inclusive
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9,  :,  ;,  <,  =,  >,  ?,  @,  A,  B,  C,
     D,  E,  F,  G,  H,  I,  J,  K,  L,  M,  N,  O,  P,  Q,  R,  S,  T,  U,  V,  W,
     X,  Y,  Z,  [,  \,  ],  ^,  _,  `,  a,  b,  c,  d,  e,  f,  g,  h,  i,  j,  k,
     l,  m,  n,  o,  p,  q,  r,  s,  t,  u,  v,  w,  x,  y,  z,
*/

#[rustfmt::skip]
const INVERSE_STRICT: [i8; 75] = [
    -1,  7, 14, 28,  6, 20, 17, 27,  2, 18, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
     9,  4, 12, 26,  5, -1, -1, -1, 15, -1, 13, -1, -1,  1, 22, -1, 30, -1, -1, -1,
    16, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  0,  3, 24, -1, 10, 21, -1, 31,  8,
    -1, 29, -1, -1, 25, 11, -1, -1, -1, -1, -1, 23, -1, 19, -1,
];

/// This inverse function allows out-of-alphabet characters that might be mistaken for
/// in-alphabet characters, to be automatically mapped to the in-alphabet character
/// they look like. Here are correction mappings (wrong,right):
///   (A,4), (B,8), (I,1), (J,j), (K,k), (M,m), (P,p), (S,5), (W,w),
///   (Y,7), (Z,2), (b,6), (i,1), (l,1), (r,R), (S,5), (x,X), (z,2)
#[rustfmt::skip]
const INVERSE_CORRECTED: [i8; 75] = [
    -1,  7, 14, 28,  6, 20, 17, 27,  2, 18, -1, -1, -1, -1, -1, -1, -1,  6,  2,  0,
     9,  4, 12, 26,  5,  7, 31,  8, 15, 29, 13, -1, 25,  1, 22, 20, 30, -1, -1, 23,
    16, 27, 14, -1, -1, -1, -1, -1, -1, -1, 17,  0,  3, 24, -1, 10, 21,  7, 31,  8,
     7, 29, -1, -1, 25, 11, 22, 20, -1, -1, -1, 23, 16, 19, 14,
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
            (b"".as_slice(), "R9DNRdc"),
            (
                b"The quick brown fox jumps over the lazy dog.".as_slice(),
                "g654gkd62h5R4GpcFDp4Tm72EQy4TTQcNDGRG3dyEQwwFpq9EQG4XpDcN6Xw5TDcF6wR2qXdX3eNR",
            ),
            (b"GM".as_slice(), "km4em3GNy3"),
            (&[246, 11, 226, 142, 73, 141, 43, 201, 119, 153, 142, 112, 11, 216, 255, 247,
              149, 36, 188, 231, 3, 176, 115, 77, 88, 172, 174, 148, 25, 78, 190, 236],
             "TeHTHd9D65h39m3p6pecwR1jTTg9DL11c2e14ygeh9wDk4g2w7R2y6Q235"),
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
        // actual = "g654gkd62h5R4GpcFDp4Tm72EQy4TTQcNDGRG3dyEQwwFpq9EQG4XpDcN6Xw5TDcF6wR2qXdX3eNR";
        let wrong = "g65Agkd6Zh5R4GpCFDp4TmYZEQy4TTQcNDGRG3dyEQwwFpq9EQG4xpDcN6xw5TDcF6wR2qXdX3eNR";
        //              ^    ^      ^      ^^                            ^     ^                   ;
        assert_eq!(
            b"The quick brown fox jumps over the lazy dog.".as_slice(),
            decode(wrong, false).unwrap()
        );

        // actual = "TeHTHd9D65h39m3p6pecwR1jTTg9DL11c2e14ygeh9wDk4g2w7R2y6Q235";
        let wrong = "TeHTHd9D6Sh39m3P6peCwRIjTTg9DLllc2e1Aygeh9wDk4g2w7R2y6Q235";
        //                    ^     ^   ^  ^       ^^    ^                      ;
        assert_eq!(
            [246, 11, 226, 142, 73, 141, 43, 201, 119, 153, 142, 112, 11, 216, 255, 247,
             149, 36, 188, 231, 3, 176, 115, 77, 88, 172, 174, 148, 25, 78, 190, 236].as_slice(),
            decode(wrong, false).unwrap()
        );

    }
}
