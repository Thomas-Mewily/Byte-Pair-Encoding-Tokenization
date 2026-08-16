/*
Based and inspired by :


https://en.wikipedia.org/wiki/Byte-pair_encoding
https://github.com/openai/tiktoken/tree/main
*/
use std::{collections::HashMap, fmt::{Debug, Formatter, Result as FmtResult}, hash::Hash, mem, ops::{Deref, DerefMut}};

mod byte_str;
pub use byte_str::*;
use hexga_ansi_color::AnsiColor;


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
impl<'a> SpanMerger<'a>
{
    pub fn colored<'b>(&'b self) -> SpanColored<'b>
    {
        SpanColored { input: self.input, span: &self.span.current }
    }
}

#[derive(Clone, Copy)]
pub struct SpanColored<'a>
{
    pub input : &'a ByteStr,
    pub span : &'a [Span],
}
impl Debug for SpanColored<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult 
    {
        use hexga_ansi_color::*;
        let rainbow = [AnsiColorKind::Blue, AnsiColorKind::Red, AnsiColorKind::Yellow, AnsiColorKind::Green, AnsiColorKind::Magenta];

        for (idx, s) in self.span.iter().enumerate()
        {
            write!(f, "{}{:?}", rainbow[idx % rainbow.len()].background(), s.get(self.input))?;
        }
        write!(f, "{}", AnsiColorKind::Black.background())
    }
}

pub type OldNextSpans = OldNext<Vec<Span>>;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct OldNext<T>
{
    pub current : T,
    pub next: T,
}
impl<T> OldNext<T>
{
    pub fn swap(&mut self)
    {
        mem::swap(&mut self.current, &mut self.next);
    }
}

impl<'a,T> From<T> for SpanMerger<'a> where T: Into<&'a ByteStr>
{
    fn from(value: T) -> Self {
        let value = value.into();
        Self { input: value, span: OldNext { current: value.iter().enumerate().map(|(idx, _)| Span { begin: idx, len: 1 }).collect(), next: Vec::new() } }
    }
}

impl<'a> Iterator for SpanMerger<'a>
{
    type Item = HashMap<&'a ByteStr, Frequency>;
    fn next(&mut self) -> Option<Self::Item> {
        let new_spans = &mut self.span.next;
        let spans = self.span.current.deref();
        let input = self.input;

        new_spans.clear();

        let mut frequency : HashMap<&ByteStr, Frequency> = HashMap::new();
        for i in 0..spans.len()-1
        {
            let (prev, next) = (spans[i], spans[i+1]);
            debug_assert_eq!(prev.begin + prev.len, next.begin);
            let merged = Span { begin: prev.begin, len: prev.len + next.len };

            let key = merged.get(input);
            frequency.entry(key).and_modify(|frequency| { *frequency += 1 }).or_insert(1);
        }

        let Some((bytes, _frequency)) = frequency.iter().max_by_key(|(_bytes, frequency)| **frequency).map(|(b,f)| (*b, *f)) else { return None; };

        assert!(spans.len() >= 1);
        
        // Merge most common shape
        new_spans.reserve(spans.len());
        let mut i = 0;
        while i <= spans.len()-2
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
                new_spans.push(prev);
                i += 1;
            }
        }
        if i == spans.len() - 1
        {
            new_spans.push(spans.last().copied().unwrap());
        }

        // dbg!(frequency);

        self.span.swap();
        Some(frequency)
    }
}


fn main() {
    let input = include_str!("./input/13704.txt");
    //let input = "bonjour le bonbon";
    //let input = "aabaa";
    let mut it = SpanMerger::from(input);

    loop
    {
        let Some(best) = it.next() else { break; };

        {
            let mut morphene_frequency : Vec<_> = best.iter().collect();
            morphene_frequency.sort_by_key(|(_b,f)| u64::MAX - **f);
            let top : Vec<_> = morphene_frequency.iter().take(10).collect();
            println!("{:?}", top);
        }

        println!("{:?}", it.colored());
        println!();

        if best.len() <= 30 { break; }
    }
}
