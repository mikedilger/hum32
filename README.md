# hum32

hum32 is a 5-bit (32 character) encoding scheme that attempts to:

* Output human writable characters that are hard to mistake for other characters
* Correct some mistakes automatically
* Detect errors via a checksum

In that vein it is similar to base32, zbase32, and bech32. We compare these as follows:

|Encoding|Chars Avoided       |Padding   |Checksum     |
|--------|--------------------|----------|-------------|
|base32  |0,1,8,9  uppercase  |yes       |no           |
|zbase32 |0,l,v,2, uppercase  |no        |no           |
|bech32  |1,b,i,o, uppercase  |no        |custom, poor |
|bech32m |1,b,i,o, uppercase  |no        |custom       |
|hum32   |g,i,o,s, samecase   |no        |xxHash       |

## Alphabet Choice

Note that hum32 uses a mixture of uppercase and lowercase characters, but never allows
both the uppercase and lowercase of the same character. We choose the case representation
with the least visual ambiguity, for example we use uppercase 'L' because lower case 'l'
looks like a '1' (and an 'i' or 'I').

The full alphabet is: 123456789aBCdEFHJkLMNOPQRtUvWXYz

We wrote code to determine this alphabet, it is available at `src/alphabet_choice.rs`
and can be run with `cargo test choose_character_set -- --nocapture`
We just swapped 'O' for '0'.

## Checksum

Prior to hum32, only bech32 and it's fixed bech32m provide a checksum.

We provide a 32-bit checksum using xxHash's xxh32 function which is performed on and appended
to the data prior to encoding. This may not be optimal for the kinds of errors humans make,
but it is easy and very effective.

## Automatic Correction

Unlike any of the prior algorithms, we detect and correct bad input. This usually only happens
when the case of the character is wrong. But other out-of-alphabet characters have substitutions
too such as 'G' probably was a '6', etc.

## Padding

We do not pad.

## Prefix support

Like bech32 we support prefixes, separated in our case with a '0'.

The crate is `#[no_std]`

## API

```
pub fn encode(plain: &[u8], prefix: Option<&str>) -> Result<String, Error>
pub fn prefix(coded: &[u8]) -> Option<&[u8]> {
pub fn decode(coded: &str, strict: bool) -> Result<Vec<u8>, Error>
```

## Example

```
input = [246, 11, 226, 142, 73, 141, 43, 201, 119, 153, 142, 112, 11, 216, 255, 247,
         149, 36, 188, 231, 3, 176, 115, 77, 88, 172, 174, 148, 25, 78, 190, 236]
output = TeHTHd9D65h39m3p6pecwR1jTTg9DL11c2e14ygeh9wDk4g2w7R2y6Q235
```

However if you mistype some of those characters like this:

```
output = TeHTHd9D6Sh39m3P6peCwRIjTTg9DLllc2e1Aygeh9wDk4g2w7R2y6Q235
```

You still get back the correct original input.

Our character set is layed out as follows:

```
       0  1  2  3  4  5  6  7

 +0    c  Q  8  d  E  H  4  1
 +8    k  D  g  q  F  N  2  L
 +16   X  6  9  y  5  h  R  w
 +24   e  p  G  7  3  m  T  j
```

