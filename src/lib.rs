#![allow(unused_variables)]
#![allow(dead_code)]

extern crate encoding;

use encoding::types::DecoderTrap;
use encoding::types::Encoding;

struct CharCounts {
    sp: u32,
    nl: u32,
    tot: u32,
}

struct CharStats {
    utf8: CharCounts,
    utf16be: CharCounts,
    utf16le: CharCounts,
    utf32: CharCounts
}

impl CharCounts {
    fn new() -> CharCounts {
        CharCounts { sp:0, nl: 0, tot: 0 } 
    }
}

impl CharStats {
    fn new() -> CharStats {
        CharStats {
            utf8: CharCounts::new(),
            utf16be: CharCounts::new(),
            utf16le: CharCounts::new(),
            utf32: CharCounts::new(),
        }
    }
}

fn count_chars(cc: &mut CharCounts, prev: &String, curr: &String, next: &String) {
    match &curr[0..] {
        "" => (),
        c @ _ => {
            match (&prev[0..], c, &next[0..]) {
                ("", _, "") => (),
                (_, "\n", _) => cc.nl += 1,
                (_, " ", _)  => cc.sp += 1,
                _ => (),
            }
            
            cc.tot += 1;
        }
    }
}

fn decode(decoder: &Encoding, input: &[u8]) -> Option<String> {
    match decoder.decode(input, DecoderTrap::Strict) {
                Result::Ok(s) => Some(s),
                Result::Err(..) => None
    }
}

fn count_stream(cc: &mut CharCounts, chars: &[u8], i: usize, enc_len: usize, encoding: &Encoding) {
    let j : usize = i / enc_len;
    let len : usize = chars.len() / enc_len;

    if i % enc_len == 0 {
        let prev : Option<String> = match j-1 {
            x if j == 0 => None,
            x if x < len => decode(encoding, &chars[x*enc_len..(x+1)*enc_len]),
            _ => None
        };

        let curr : Option<String> = match j {
            x if j == 0 => None,
            x if x < len => decode(encoding, &chars[x*enc_len..(x+1)*enc_len]), 
            _ => None
        };

        let next : Option<String> = match j+1 {
            x if j == 0 => None,
            x if x < len => decode(encoding, &chars[x*enc_len..(x+1)*enc_len]), 
            _ => None
        };

        match (prev, curr, next) {
            (Some(p), Some(c), Some(n)) => {
                count_chars(cc, &p, &c, &n);
            },
            _ => ()
        }
    }
}

fn count_utf8(cs: &mut CharStats, chars: &[u8], i: usize) {
    use encoding::all::UTF_8;
    count_stream(&mut cs.utf8, chars, i, 1, UTF_8);
}

fn count_utf16(cs: &mut CharStats, chars: &[u8], i: usize) {
    use encoding::all::UTF_16LE;
    use encoding::all::UTF_16BE;
    count_stream(&mut cs.utf16le, chars, i, 2, UTF_16LE);
    count_stream(&mut cs.utf16be, chars, i, 2, UTF_16BE);
}

fn count_utf32(cs: &mut CharStats, chars: &[u8], i: usize) {
    ()
}



fn get_char_stats(chars: &[u8]) -> CharStats {
    let mut cs = CharStats::new();

    for i in 0..chars.len() {
        count_utf8(&mut cs, chars, i);
        count_utf16(&mut cs, chars, i);
        count_utf32(&mut cs, chars, i);
    }

    cs
}

#[cfg(test)]
mod tests {
    use super::CharCounts;
    use super::CharStats;

    fn assert_char_counts(cc: &CharCounts, sp: u32, nl: u32, tot: u32) {
        assert!(cc.sp == sp);
        assert!(cc.nl == nl);
        assert!(cc.tot == tot);
    }

    fn to_utf8_bytes(s: &str) -> Vec<u8>
    {
        s.as_bytes().iter().map(|&x| x).collect::<Vec<u8>>()
    }

    fn to_utf16le_bytes(s: &str) -> Vec<u8>
    {
        let mut o : Vec<u8> = vec![];

        for b in s.as_bytes().iter() {
            o.push(*b);
            o.push(0);
        }

        o
    }

    #[allow(unstable)]
    #[test]
    fn test_count() {
        use super::count_chars;

        let cc = &mut CharCounts::new();
        assert_char_counts(cc, 0, 0, 0);

        count_chars(cc, &" ".to_string(), &" ".to_string(), &" ".to_string());
        assert_char_counts(cc, 1, 0, 1);
        
        count_chars(cc, &" ".to_string(), &"\n".to_string(), &" ".to_string());
        assert_char_counts(cc, 1, 1, 2);
    }

    #[test]
    fn test_count_utf8() {
        use super::count_utf8;
        
        let chars : &[u8]   = &to_utf8_bytes(" a \nb\n")[0..]; 

        let cs = & mut CharStats::new();
        assert_char_counts(&cs.utf8, 0, 0, 0);

        count_utf8(cs, chars, 0); // edge space
        assert_char_counts(&cs.utf8, 0, 0, 0);
        
        count_utf8(cs, chars, 1); // a
        assert_char_counts(&cs.utf8, 0, 0, 1);
        
        count_utf8(cs, chars, 2); // space 
        assert_char_counts(&cs.utf8, 1, 0, 2);
        
        count_utf8(cs, chars, 3); // \n 
        assert_char_counts(&cs.utf8, 1, 1, 3);
        
        count_utf8(cs, chars, 4); // b 
        assert_char_counts(&cs.utf8, 1, 1, 4);

        count_utf8(cs, chars, 5); // edge \n
        assert_char_counts(&cs.utf8, 1, 1, 4);
    }

    #[test]
    fn test_count_utf16le() {
        use super::count_utf16;
        
        let chars : &[u8]   = &to_utf16le_bytes(" a \nb\n")[0..]; 

        let cs = & mut CharStats::new();
        assert_char_counts(&cs.utf16le, 0, 0, 0);

        count_utf16(cs, chars, 0); // edge space
        assert_char_counts(&cs.utf16le, 0, 0, 0);

        count_utf16(cs, chars, 2); // a
        assert_char_counts(&cs.utf16le, 0, 0, 1);

        count_utf16(cs, chars, 3); // invalid utf-16 position
        assert_char_counts(&cs.utf16le, 0, 0, 1);

        count_utf16(cs, chars, 4); // space 
        assert_char_counts(&cs.utf16le, 1, 0, 2);

        count_utf16(cs, chars, 5); // invalid utf-16 position 
        assert_char_counts(&cs.utf16le, 1, 0, 2);

        count_utf16(cs, chars, 6); // \n 
        assert_char_counts(&cs.utf16le, 1, 1, 3);

        count_utf16(cs, chars, 8); // b
        assert_char_counts(&cs.utf16le, 1, 1, 4);

        //count_utf16(cs, chars, 10); // edge \n
        //assert_char_counts(&cs.utf16le, 1, 1, 4);
    }
}



