use std::collections::BTreeSet;

// symbol similarity matrix.
// (symbol, strong matches, weak matches, aural matches)
//
// The symbols are in preferred order (numbers, uppercase, lowercase)
//
// If you edit this, run `cargo test --features=charset test_symmetry -- --nocapture`
// to verify symmetry.  That is, if X is similar to Y, then Y must be similar to X.

/// Similar alphanumeric character pairs, ordered by their visual similarity.
/// (We don't include uppercase-lowercase pairs of the same letter, which we exclude
///  simply for aural reasons)
const VISUAL_PAIRS: &[(char, char)] = &[
    ('1','l'),
    ('0','O'),
    ('1','i'),
    ('1','I'),
    ('i','l'),
    ('l','I'),
    ('0','o'),
    ('5','S'),
    ('2','Z'),
    ('6','G'),
    ('9','g'),
    ('6','b'),
    ('U','V'),
    ('u','v'),
    ('5','s'),
    ('2','z'),
    ('I','L'),
    ('u','V'),
    ('v','U'),
    ('8','B'),
    ('4','A'),
    ('7','T'),
    ('D','O'),
    ('D','0'),
    ('G','b'),
    ('a','o'),
    ('b','G'),
    ('n','r'),
    ('o','a'),
    ('r','v'),
    ('7','K'),
];

/// Similar alphanumeric character pairs, ordered by their aural similarity
/// We don't include uppercase, the lowercase represents.
const AURAL_PAIRS: &[(char, char)] = &[
    ('b','p'),
    ('b','d'),
    ('m','n'),
    ('6','x'),
    ('8','a'),
];

const FULL_ALPHABET: &str  = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";


// run with `cargo test --features=charset choose_character_set -- --nocapture`
#[test]
fn choose_character_set() {
    let mut chosen: BTreeSet<char> = BTreeSet::new();

    // A closure to print our results so far
    let sofar = |chosen: &BTreeSet<char>, which: &str| {
        eprintln!("We got {} characters so far using {which}", chosen.len());
        eprint!("    ");
        for c in chosen.iter() {
            eprint!("{c}");
        }
        eprintln!();
    };

    // First pass: Choose symbols, excluding symbols that have any kind of
    // similarity at all
    'pass1:
    for c in FULL_ALPHABET.chars() {
        for already in chosen.iter() {
            if is_same_letter(c, *already) {
                continue 'pass1;
            }
            if visual_similarity(c, *already) > 0 {
                continue 'pass1;
            }
            if aural_similarity(c, *already) > 0 {
                continue 'pass1;
            }
        }
        chosen.insert(c);
    }
    sofar(&chosen, "No similarity");

    let mut alphabet = FULL_ALPHABET.chars().collect::<Vec<char>>();
    alphabet.retain(|c| !chosen.contains(c));

    // Second pass: allow aural similarity
    'pass2:
    for &c in alphabet.iter() {
        for already in chosen.iter() {
            if is_same_letter(c, *already) {
                continue 'pass2;
            }
            if visual_similarity(c, *already) > 0 {
                continue 'pass2;
            }
        }
        chosen.insert(c);
    }
    sofar(&chosen, "Aural similarity");

    // Third pass: choose the weakest visual similarity
    let mut max_similarity = 1;
    while chosen.len() < 32 {
        alphabet.retain(|c| !chosen.contains(c));

        'passn:
        for &c in alphabet.iter() {
            for already in chosen.iter() {
                if is_same_letter(c, *already) {
                    continue 'passn;
                }
                if visual_similarity(c, *already) > max_similarity {
                    continue 'passn;
                }
            }
            chosen.insert(c);
        }
        sofar(&chosen, &*format!("Max similarity {max_similarity}"));

        max_similarity += 1;

        if max_similarity > VISUAL_PAIRS.len() {
            panic!("No Solution Found");
        }
    }

    sofar(&chosen, "Optimal Alphabet");
}

fn visual_similarity(a: char, b: char) -> usize {
    for (i, (x, y)) in VISUAL_PAIRS.iter().rev().enumerate() {
        if (a==*x && b==*y) || (a==*y && b==*x) {
            return i+1;
        }
    }
    0
}

fn aural_similarity(a: char, b: char) -> usize {
    for (i, (x, y)) in AURAL_PAIRS.iter().rev().enumerate() {
        let xx = x.to_uppercase().to_string();
        let yy = y.to_uppercase().to_string();
        let aa = a.to_uppercase().to_string();
        let bb = b.to_uppercase().to_string();

        if (aa==xx && bb==yy) || (aa==yy && bb==xx) {
            return i+1;
        }
    }
    0
}

fn is_same_letter(a: char, b: char) -> bool {
    a==b ||
        a.to_uppercase().to_string() == b.to_uppercase().to_string()
}

