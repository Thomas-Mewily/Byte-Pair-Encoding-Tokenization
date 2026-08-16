/*
Based and inspired by :


https://en.wikipedia.org/wiki/Byte-pair_encoding
https://github.com/openai/tiktoken/tree/main
*/
use std::{collections::{HashMap, HashSet}, fmt::{Debug, Formatter, Result as FmtResult}, hash::Hash, ops::{Deref, DerefMut}};

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
    pub fn end(&self) -> usize { self.begin + self.len }

    pub fn get<'a>(&self, input : &'a ByteStr) -> &'a ByteStr
    {
        ByteStr::from_ref(&input[self.begin..self.begin+self.len])
    }

    pub fn is_empty(&self) -> bool { self.len == 0 }
}

pub type Count = u64;


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

pub type SpanID = usize;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MorpheneEntry
{
    //pub prefix : &'a Morphene,
    //pub suffix : &'a Morphene,
    pub spans_id : HashSet<SpanID>,
}
impl MorpheneEntry
{
    pub fn nb(&self) -> usize { self.spans_id.len() }
}

#[derive(Debug)]
pub struct SpanMerger<'a>
{
    pub input : &'a ByteStr,
    /// Span are never removed. Instead they are replaced by a Span with a 0 len
    pub spans: Vec<Span>, //OldNextSpans,
    /// Morphene encoder
    pub glued_morphenes : HashMap<&'a Morphene, MorpheneEntry>,
}
impl<'a> SpanMerger<'a>
{
    pub fn colored<'b>(&'b self) -> SpanColored<'b>
    {
        SpanColored { input: self.input, span: &self.spans }
    }
}

#[derive(Clone, Copy)]
pub struct SpanColored<'a>
{
    pub input : &'a ByteStr,
    pub span : &'a [Span],
}
impl<'a> SpanColored<'a>
{
    pub fn limit(self, max_byte: usize) -> Self 
    {
        let end_slice_idx = self.span.partition_point(|s| s.end() <= max_byte);
        Self { input: &self.input[0.. max_byte.min(self.input.len())], span: &self.span[0..end_slice_idx] }
    }
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
        writeln!(f, "{} ", AnsiColorKind::Black.background())
    }
}

/*
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
*/

impl<'a> SpanMerger<'a> 
{
    pub(crate) fn generate_morphene(mut self) -> Self
    {
        self.glued_morphenes.clear();
        for (idx, s) in self.spans.iter().enumerate()
        {
            let key = s.get(self.input);
            let entry = self.glued_morphenes.entry(key).or_insert_with(||  MorpheneEntry::default());
            entry.spans_id.insert(idx);
        }
        self
    }

    pub fn from_bytes<T>(value: T) -> Self 
        where T: Into<&'a ByteStr>
    {
        let value = value.into();
        Self { input: value, spans: value.iter().enumerate().map(|(idx, _)| Span { begin: idx, len: 1 }).collect(), glued_morphenes: HashMap::new() }.generate_morphene()
    }

    pub fn from_text<T>(value: T) -> Self 
        where T: Into<&'a ByteStr>
    {
        let value = value.into();
        let s = str::from_utf8(value).expect("valid utf8");
        
        // Create spans for each Unicode character (not each byte)
        let spans: Vec<Span> = s.char_indices()
            .map(|(idx, ch)| Span {
                begin: idx,
                len: ch.len_utf8(),  // 1 for ASCII, 2 for ô, 3 for é, etc.
            })
            .collect();
        
        Self {
            input: value,
            spans,
            glued_morphenes: HashMap::new(),
        }.generate_morphene()
    }
}


pub type Morphene = ByteStr;
//pub type NextMorpheneEntry<'a, 'entry> = (&'a Morphene, &'entry MorpheneEntry);

impl<'a> SpanMerger<'a>
{
    pub fn most_frequent_morphene(&self) -> Option<(&'a Morphene, &MorpheneEntry)>
    {
        let Some((morphene, entry)) = self.glued_morphenes.iter().max_by_key(|(_bytes, entry)| entry.nb()).map(|(b,f)| (*b, f)) else { return None; };

        Some((morphene, entry))
    }

    pub fn merge_morphene(&mut self, morphene : &Morphene) -> Result<usize, ()>
    {
        let Some(entry) = self.glued_morphenes.remove(morphene) else { return Err(()); };

        let mut nb_merged = 0;

        for span_id in entry.spans_id
        {
            let span = self.spans[span_id];

            let have_prev = span_id >= 1;
            let have_next = span_id + 1 < self.spans.len();

            if have_prev || have_next
            {
                // Dead, no longer used
                self.spans[span_id].len = 0;
            }

            if !have_prev && !have_next
            {
                // Don't have any neighbor
                let entry = self.glued_morphenes.entry(span.get(self.input)).or_insert_with(|| MorpheneEntry::default());
                entry.spans_id.insert(span_id);
            }

            if have_prev
            {
                // Not first, can merge with prev morphene
                let prev_id = span_id-1;
                let mut prev = self.spans[prev_id];
                self.glued_morphenes.get_mut(prev.get(self.input)).ok_or(())?.spans_id.remove(&prev_id);

                prev.len += span.len;
                self.spans[prev_id] = prev;
                
                let entry = self.glued_morphenes.entry(prev.get(self.input)).or_insert_with(|| MorpheneEntry::default());
                entry.spans_id.insert(prev_id);

                nb_merged += 1;
            }

            if have_next
            {
                // Not last, can merge with next morphene
                let next_id = span_id+1;
                let mut next = self.spans[next_id];
                self.glued_morphenes.get_mut(next.get(self.input)).ok_or(())?.spans_id.remove(&next_id);

                next.begin = span.begin;
                next.len += span.len;
                self.spans[next_id] = next;
                
                let entry = self.glued_morphenes.entry(next.get(self.input)).or_insert_with(|| MorpheneEntry::default());
                entry.spans_id.insert(next_id);

                nb_merged += 1;
            }
        }

        Ok(nb_merged)
    }
}

impl<'a> Iterator for SpanMerger<'a>
{
    type Item = &'a Morphene;
    fn next(&mut self) -> Option<Self::Item> 
    {
        let (morphene, _entry) = self.most_frequent_morphene()?;
        let _ = self.merge_morphene(morphene);
        Some(morphene)

        /*
        let new_spans = &mut self.span.next;
        let spans = self.span.current.deref();
        let input = self.input;

        let morphene = &mut self.morphenes;

        morphene.clear();
        new_spans.clear();

        for i in 0..spans.len()-1
        {
            let (prev, next) = (spans[i], spans[i+1]);
            debug_assert_eq!(prev.end(), next.begin);
            let merged = Span { begin: prev.begin, len: prev.len + next.len };

            let key = merged.get(input);
            morphene.entry(key).and_modify(|frequency| { *frequency += 1 }).or_insert(1);
        }

        let Some((bytes, frequency)) = morphene.iter().max_by_key(|(_bytes, frequency)| **frequency).map(|(b,f)| (*b, *f)) else { return None; };

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
        Some((bytes, frequency))
        */
    }
}


fn main() {
    //let input = include_str!("./input/13704.txt");
    //let input = include_str!("./input/18812.txt");
    //let input = "bonjour le bonbon";
    //let input = "aabaa";
    //let input = "aabaa";
    let input = "a";
    let mut it = SpanMerger::from_text(input);
    let mut nb = 0;

    dbg!(&it);

    while let Some(morphene) = it.next()
    {
        nb += 1;


        dbg!(&morphene);
        dbg!(&it);

        let entry = &it.glued_morphenes[morphene];
        println!();
        println!("{nb} : \"{morphene:?}\" {:?}", entry);
        println!("{:?}", it.colored().limit(100));

        //let wait = std::io::stdin().read_line(&mut String::new());
        
        //if entry.nb() <= 1 || morphene.len() >= 32 { break; }
        //if it.morphene.len() <= 200 { break;}
        /*
        if let Some(m) = morphene.as_ref()
        {
            let mut morphene_frequency : Vec<_> = m.iter().collect();
            morphene_frequency.sort_by_key(|(_b,f)| u64::MAX - **f);
            let top : Vec<_> = morphene_frequency.iter().take(10).collect();
            println!("{:?}", top);
        }

        println!("{:?}", it.colored());
        println!();
        */

        //if morphene.len() <= 30 { break; }
    }

    //println!("{:?}", it.glued_morphenes);
    //println!("{:?}", it.colored());
}
