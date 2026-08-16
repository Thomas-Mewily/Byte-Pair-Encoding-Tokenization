/*
Based and inspired by :


https://en.wikipedia.org/wiki/Byte-pair_encoding
https://github.com/openai/tiktoken/tree/main
*/
use std::{collections::HashMap, fmt::{Debug, Formatter, Result as FmtResult}, hash::Hash, ops::{Deref, DerefMut}};

mod byte_str;
pub use byte_str::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span
{
    begin : usize,
    len   : usize,
}
impl Span
{
    pub fn get<'a>(&self, input : &'a ByteStr) -> &'a ByteStr
    {
        ByteStr::from_ref(&input[self.begin..self.begin+self.len])
    }
}

pub type Frequency = u64;


// Can generalize over dimension (1D, 2D), shape dection, how the testing new shape work (here adjacency only)
pub trait IndividualShapeSpan
{
    fn individual_shape_span(self) -> Vec<Span>;
}
impl<T> IndividualShapeSpan for T where T: Iterator<Item=u8>
{
    fn individual_shape_span(self) -> Vec<Span> {
        self.into_iter().enumerate().map(|(idx, _)| Span { begin: idx, len: 1 }).collect()
    }
} 

pub struct SpanMerger<'a>
{
    pub input : &'a ByteStr,
    pub span: OldNextSpans,
}

pub type OldNextSpans = OldNext<Vec<Span>>;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct OldNext<T>
{
    pub old : T,
    pub next: T,
}

impl<'a> From<&'a ByteStr> for SpanMerger<'a>
{
    fn from(value: &'a ByteStr) -> Self {
        Self { input: value, span: OldNext { old: value.iter().enumerate().map(|(idx, _)| Span { begin: idx, len: 1 }).collect(), next: Vec::new() } }
    }
}

/*
impl<'a> Iterator for SpanMerger<'a>
{
    type Item;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
*/

fn next_span(input : &ByteStr, spans: &mut Vec<Span>, mut new_spans: Vec<Span>) -> Vec<Span>
{
    new_spans.clear();
    //let spans : Vec<Span> = input.iter().enumerate().map(|(idx, _)| Span { begin: idx, len: 1 }).collect();

    let mut frequency : HashMap<&ByteStr, Frequency> = HashMap::new();

    for i in 0..spans.len()-1
    {
        let (prev, next) = (spans[i], spans[i+1]);
        let merged = Span { begin: prev.begin, len: prev.len + next.len };

        let key = merged.get(input);
        frequency.entry(key).and_modify(|frequency| { *frequency += 1 }).or_insert(1);
    }

    let Some((bytes, frequency)) = frequency.iter().max_by_key(|(_bytes, frequency)| **frequency).map(|(b,f)| (*b, *f)) else { return new_spans; };

    new_spans.reserve(spans.len());
    
    let mut i = 0;
    while i <= spans.len()-1
    {
        let (prev, next) = (spans[i], spans[i+1]);
        let len = prev.len+next.len;

        if prev.len + next.len == bytes.len()
            && prev.get(input) == bytes[0..prev.len]
            && next.get(input) == bytes[prev.len.. len]
        {
            // match the bytes
            new_spans.push(Span { begin: prev.begin, len });
            i += 2;
        }else
        {
            new_spans.push(spans[i]);
            i += 1;
        }
    }

    // merge most frequent group

    dbg!(frequency);
    new_spans
}

fn main() {
    let input = include_str!("./input/13704.txt");
    merge_span(ByteStr::from_ref(input.as_bytes()));
}
