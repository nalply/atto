use range_set::RangeSet;
use regex::Regex;

fn main() {
    //    let all = pred_range(|_| true);
    let valid = pred_range_rx(r"[^\pC\pZ]|[\p{Cf}]");
    //    let x = subtract(&all, &valid);
    //    print_character_sets(&x);

    //    let alphabetic = pred_range(|c| c.is_alphabetic());
    //   let x = subtract(&valid, &alphabetic);

    print_character_sets(&valid);
}

fn subtract(target: &Ranges, other: &Ranges) -> Ranges {
    let mut target = target.clone();

    for range in other.as_ref().iter() {
        target.remove_range(range.clone());
    }

    target
}

fn pred_range_rx(rx: &str) -> Ranges {
    fn encode_utf8(c: char) -> String {
        let mut buf = [0u8; 4];
        c.encode_utf8(buf.as_mut_slice()).to_owned()
    }

    let rx = Regex::new(rx).unwrap();
    pred_range(|c| rx.is_match(&encode_utf8(c)))
}

fn pred_range<F: Fn(char) -> bool>(predicate: F) -> Ranges {
    let mut range_set = Ranges::new();
    for u in 0..=char::MAX as u32 {
        if predicate(char::from_u32(u).unwrap_or_default()) {
            range_set.insert(u);
        }
    }

    range_set
}

type Ranges = RangeSet<[std::ops::RangeInclusive<u32>; 1]>;

fn print_character_sets(ranges: &Ranges) {
    ranges.as_ref().iter().for_each(|range| {
        let start = range.start();
        let end = range.end();
        if start == end {
            print!("'\\u{{{start:02x}}}' ");
        } else {
            print!("'\\u{{{start:02x}}}'-'\\u{{{end:02x}}}' ");
        }
    });
    println!();
}
