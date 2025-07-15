# hum32

hum32 is a 32-bit encoding scheme that attempts to output human writable characters that
are hard to mistake for other characters, correct mistakes, and detect errors via checksumming.
In that vein it is similar to base32, zbase32, and bech32.

Unlike those former solutions to this problem, we chose to use both uppercase and lowercase
symbols, giving us we think more visual separation between characters.

Unlike all but bech32, we add a checksum. Ours is 32 bits long using xxHash's xxh32 function
and performed on (and appended to) the data prior to encoding.

We also choose to optionally detect and automatically correct out-of-alphabet single-character
mistakes based on visual similarity.

Unlike base32, but like zbase32 and bech32, we do not waste space with padding characters.

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

## Visual similarity

These character groups were used for visual similarity:

```
(a,o,O,0), (A,4), (5,S,s), (b,6), (B,8), (z,Z,2), (i,I,l,1), (u,U,v,V), (r,n), (r,v), (c,C),
(j,J), (k,K), (m,M), (p,P), (s,S), (u,U), (v,V), (w,W), (x,X), (y,Y), (z,Z)
```

## Automatic corrections

These corrections are applied automatically (wrong, corrected):

```
(A,4), (B,8), (I,1), (J,j), (K,k), (M,m), (P,p), (S,5), (W,w),
(Y,7), (Z,2), (b,6), (i,1), (l,1), (r,R), (S,5), (x,X), (z,2)
```
