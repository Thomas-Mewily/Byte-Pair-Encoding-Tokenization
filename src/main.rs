/*
Based and inspired by :


https://en.wikipedia.org/wiki/Byte-pair_encoding
https://github.com/openai/tiktoken/tree/main
*/
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::Hash,
    ops::{Deref, DerefMut},
};

mod byte_str;
pub use byte_str::*;

mod span_merger;
pub use span_merger::*;

mod sub_slice;
pub use sub_slice::*;

/*
fn test_input()
{
    type NbMorpheme = usize;
    let text_input =  include_str!("./input/18812.txt"); // French

    let input = ByteStr::from_ref(text_input.as_bytes());
    let mut it = SpanMerger::from_text(input);

    loop
    {
        it.reset_span();
        it.reset_merge_pair();

        while let Some((_morphene, frequency)) = it.next()
        {
            if frequency <= 10 { break; }
        }

        let most_used_morphene = it.morphemes_vec();
        let add_split : Option<(&Morphene, NbMorpheme)> = None;

        for (morphene, frequence) in most_used_morphene
        {
            todo!();

            /*
            match it.input_kind
            {
                InputKind::Text =>
                {
                    for c in input.as_str().unwrap().all_sub_slice()
                    {

                    }
                },
                InputKind::Byte => todo!(),
            }
            for c in morphene
            {

            }
            */
        }
    }

    //let mut seprator_candidate = HashSet::new();
}
*/

/*
fn find_best_separator<'a>(s : SpanMerger<'a>)
{
    s.tokens_map()
}
*/

fn test_tokenization_morphemization() {
    let input = include_str!("./input/100.txt"); // English super long
    //let input = include_str!("./input/18812.txt"); // French
    //let input = include_str!("./input/14741.txt"); // Russian

    //let input = &*std::fs::read_to_string("src/input/rust_std_file_glued_uncomplete.txt").expect("failed to read"); // Rust
    //let input = include_str!("./input/rust_std_file_glued_uncomplete.txt"); // Rust super super long
    //let input = "bonjour le bonbon";
    //let input = "abcabcxbc";
    //let input = "aabaa";
    //let input = "aaa";
    //let input = "ab_ab-ab";
    let mut it = SpanMerger::from_text(input);

    // Force separator to be a morphene on it's own
    // Todo detect separator. Separator tend to lower the total number of token
    // In the most used morphene, split in in different group to isolate separator
    //let separator = [" ", ",", ".", "!", "?", ";", "-"];
    //let separator = ["e"]; // 2910 tokens on french
    let separator = [" "]; // 2865 tokens on french
    //let separator: &[&'static str] = &[]; // 3340 tokens on french
    for sep in separator {
        it.single_morphene
            .insert(sep.as_bytes().iter().copied().collect());
    }
    it.reset_merge_pair(); // since we added some separator
    let mut _nb = 0;

    let mut output = String::new();

    //dbg!(&it);

    while let Some((morphene, nb_merged)) = it.next() {
        if nb_merged <= 20 {
            break;
        }
        _nb += 1;
        output.push_str(&format!("\"{morphene:?}\" x{}\n", nb_merged));

        //println!();
        println!("{_nb} : \"{morphene:?}\" merged x{}", nb_merged);
        //println!("{:?}", it.colored().limit(100));
        //dbg!(&it);
    }

    let ex_len = u16::MAX as usize;
    //let ex_len = 256;

    output.push_str("\n\n");
    output.push_str("\n\n");
    output.push_str("Morphene:");
    output.push_str("\n\n");
    output.push_str("\n\n");

    for (morphene, frequency) in it.morphemes_vec() {
        output.push_str(&format!("\"{:?}\" : x{}\r\n", morphene, frequency));
    }
    output.push_str("\n\n");
    output.push_str("\n\n");

    output.push_str("Exemple of tokenization:");
    output.push_str("\n\n");
    output.push_str(&format!("{:?}", it.colored().limit(ex_len)));

    let path = "./exemple.txt";
    let full_path = std::path::absolute(path).expect("can't absolute path");
    std::fs::write(&full_path, output).expect("failed to save");

    println!("Done at {:?}", full_path);
    println!("{:?}", it.colored().limit(ex_len));

    println!("nb tokens : {}", it.morphemes_map().len());
}

fn _char_frequency() {
    // Can be used to guess if there is any separator (like spacing ?)
    let mut char_frequency: HashMap<char, u64> = HashMap::new();
    let input = include_str!("./input/18812.txt");
    let mut total_use = 0;
    for c in input.chars() {
        *char_frequency.entry(c).or_insert(0) += 1;
        total_use += 1;
    }
    let mut v = Vec::from_iter(char_frequency);
    v.sort_by_key(|(_c, nb)| *nb);
    let mut cumulated = 0.;
    let v: Vec<_> = v
        .into_iter()
        .rev()
        .map(|(c, nb)| {
            let coef = nb as f64 / total_use as f64;
            cumulated += coef;
            (c, nb, coef, cumulated)
        })
        .collect();

    for (c, f, usage, usage_cumul) in v {
        println!(
            "{} : {} / {} => {:.4} %, cumulated: {:.4} %",
            c,
            f,
            total_use,
            usage * 100.,
            usage_cumul * 100.
        );
    }
}

fn main() {
    //char_frequency();
    test_tokenization_morphemization();
    //test_input();
}
