/*
Based and inspired by :


https://en.wikipedia.org/wiki/Byte-pair_encoding
https://github.com/openai/tiktoken/tree/main
*/
use std::{collections::{HashMap, HashSet}, fmt::{Debug, Formatter, Result as FmtResult}, hash::Hash, ops::{Deref, DerefMut}};

mod byte_str;
pub use byte_str::*;


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span
{
    pub begin : usize,
    pub len   : usize,
}
impl Debug for Span
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}..{}", self.begin, self.end())
    }
}

impl Span
{
    pub fn end(&self) -> usize { self.begin + self.len }
    pub fn set_end(&mut self, end: usize) { self.len = end - self.begin; }
    pub fn with_added_len(mut self, extra_len: usize) -> Self { self.len += extra_len; self }

    pub fn get<'a>(&self, input : &'a ByteStr) -> &'a ByteStr
    {
        ByteStr::from_ref(&input[self.begin..self.begin+self.len])
    }

    pub fn is_empty(&self) -> bool { self.len == 0 }
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
    /// Span are never removed. 
    /// Instead they are replaced by a Span with a 0 len.
    /// All the span are always contiguous (relative to the input) and do not overlap together.
    pub spans: Vec<Span>, //OldNextSpans,
    /// Morphene encoder
    pub pair_frequency : HashMap<&'a Morphene, MorpheneEntry>,
    tmp_span_id : Vec<SpanID>,
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
        let mut rainbow_idx = 0;
        writeln!(f, "{} ", AnsiColorKind::Black.foreground())?;

        for s in self.span.iter()
        {
            if s.is_empty() { continue; }
            if s.end() >= self.input.len() { break; }
            rainbow_idx += 1;
            write!(f, "{}{:?}", rainbow[rainbow_idx % rainbow.len()].background(), s.get(self.input))?;
        }
        writeln!(f, "{}{} ", AnsiColorKind::White.foreground(), AnsiColorKind::Black.background())
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
        self.pair_frequency.clear();

        for i in 0..self.spans.len() - 1
        {
            let prev = self.spans[i];
            let next = self.spans[i+1];
            let pair = Span { begin: prev.begin, len: prev.len + next.len  };
            
            let key = pair.get(self.input);
            let entry = self.pair_frequency.entry(key).or_insert_with(||  MorpheneEntry::default());
            entry.spans_id.insert(i);
        }
        /*
        // individual per spans
        for (idx, s) in self.spans.iter().enumerate()
        {
            let key = s.get(self.input);
            let entry = self.pair_frequency.entry(key).or_insert_with(||  MorpheneEntry::default());
            entry.spans_id.insert(idx);
        }
        */
        self
    }

    pub fn from_bytes<T>(value: T) -> Self 
        where T: Into<&'a ByteStr>
    {
        let value = value.into();
        Self { input: value, spans: value.iter().enumerate().map(|(idx, _)| Span { begin: idx, len: 1 }).collect(), pair_frequency: HashMap::new(), tmp_span_id: Vec::new() }.generate_morphene()
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
            pair_frequency: HashMap::new(),
            tmp_span_id: Vec::new(),
        }.generate_morphene()
    }
}


pub type Morphene = ByteStr;
pub type MorphenePair = ByteStr;
//pub type NextMorpheneEntry<'a, 'entry> = (&'a Morphene, &'entry MorpheneEntry);

impl<'a> SpanMerger<'a>
{
    pub fn most_frequent_morphene_pair(&self) -> Option<(&'a MorphenePair, &MorpheneEntry)>
    {
        let Some((morphene, entry)) = self.pair_frequency.iter().max_by_key(|(_bytes, entry)| entry.nb()).map(|(b,f)| (*b, f)) else { return None; };

        Some((morphene, entry))
    }

    /// ```md
    /// prev prefix suffix next
    ///      |||||||||||||
    ///  aka morphene_pair / merged_pair
    /// ```
    pub fn merge_morphene(&mut self, morphene_pair : &MorphenePair) -> Result<NbMerged, ()>
    {
        // println!("{}", morphene_pair.escape_ascii());
        //dbg!(&morphene_pair.escape_ascii());
        //dbg!(&self.pair_frequency);

        let Some(entry) = self.pair_frequency.remove(morphene_pair) else { return Err(()); };

        let merged_len = morphene_pair.len();

        let mut nb_merged = 0;

        let spans_id = &mut self.tmp_span_id;
        spans_id.clear();
        spans_id.extend(entry.spans_id.into_iter());
        spans_id.sort(); // Force to merge in a determinist way, left to right

        for prefix_id in spans_id.iter().copied()
        {
            let prefix_span = self.spans[prefix_id];

            // Was already merged by a nears by morphene in this loop
            if prefix_span.len == 0 { continue; }

            let prefix_len = prefix_span.len;
            let suffix_len = merged_len - prefix_len;
            let merged_span = prefix_span.with_added_len(suffix_len);

            let prev_id = 
            {
                let mut tmp_prev_idx = prefix_id;
                loop
                {
                    if tmp_prev_idx == 0 
                    {
                        break None;
                    }
                    tmp_prev_idx -= 1;
                    if self.spans[tmp_prev_idx].len != 0 { break Some(tmp_prev_idx); }
                }
            };
            let (suffix_id, next_id) = 
            {
                let mut suffix_id = None;
                let mut next_id = None;
                let mut tmp_next_idx = prefix_id;
                loop
                {
                    if tmp_next_idx == self.spans.len() - 1 
                    {
                        break;
                    }
                    tmp_next_idx += 1;

                    let cur = self.spans[tmp_next_idx];
                    if cur.len != 0
                    { 
                        if suffix_id.is_none()
                        {
                            suffix_id = Some(tmp_next_idx);
                        }else
                        {
                            next_id = Some(tmp_next_idx);
                            break;
                        }
                    }
                }
                (suffix_id.expect("no preffix?"), next_id)
            };
            let suffix_span = self.spans[suffix_id];

            if let Some(prev_id) = prev_id
            {
                // Not first, can merge with prev morphene
                let prev = self.spans[prev_id];
                let prev_pair = prev.with_added_len(prefix_len);
                debug_assert_eq!(prev.end(), prefix_span.begin);

                match self.pair_frequency.get_mut(prev_pair.get(self.input))
                {
                    Some(entry) => { entry.spans_id.remove(&prev_id); },
                    None => { assert_eq!(prev_pair.get(self.input), morphene_pair); /* maybe unreachable() */ },
                }

                let new_prev_pair = prev.with_added_len(merged_len);
                self.pair_frequency.entry(new_prev_pair.get(self.input)).or_insert_with(|| MorpheneEntry::default()).spans_id.insert(prev_id);
            }

            if let Some(next_id) = next_id
            {
                // Not last, can merge with next morphene
                let next = self.spans[next_id];
                debug_assert_eq!(next.begin, suffix_span.end());
                let suffix_pair = suffix_span.with_added_len(next.len);

                // if next_pair.len >= 4
                // {
                //     dbg!(&next.get(self.input));
                //     dbg!(&next_pair.get(self.input));
                // }
                match self.pair_frequency.get_mut(suffix_pair.get(self.input))
                {
                    Some(entry) => { entry.spans_id.remove(&suffix_id); },
                    None => { assert_eq!(suffix_pair.get(self.input), morphene_pair); },
                }

                let new_next_pair = merged_span.with_added_len(next.len);
                self.pair_frequency.entry(new_next_pair.get(self.input)).or_insert_with(|| MorpheneEntry::default()).spans_id.insert(prefix_id);
            }

            // Merge inside the code source
            self.spans[prefix_id] = merged_span;
            self.spans[suffix_id].len = 0;
            nb_merged += 1;

            /*
            let entry: &mut MorpheneEntry = self.pair_frequency.entry(merged_span.get(self.input)).or_insert_with(|| MorpheneEntry::default());
            entry.spans_id.insert(prefix_id);
            */
        }

        //dbg!(&self.pair_frequency);
        //dbg!(&self.spans);

        Ok(nb_merged)
    }
}

pub type NbMerged = usize;

impl<'a> Iterator for SpanMerger<'a>
{
    type Item = (&'a Morphene, NbMerged);
    fn next(&mut self) -> Option<Self::Item> 
    {
        let (morphene, _entry) = self.most_frequent_morphene_pair()?;
        let nb_merged = self.merge_morphene(morphene).ok()?;
        if nb_merged == 0 { return None; }
        Some((morphene, nb_merged))

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


fn tokenize() {
    //let input = include_str!("./input/13704.txt");
    let input = include_str!("./input/18812.txt");
    //let input = "bonjour le bonbon";
    //let input = "abcabcxbc";
    //let input = "aabaa";
    //let input = "aaa";
    //let input = "ab_ab-ab";
    let mut it = SpanMerger::from_text(input);

    let mut _nb = 0;

    let mut output = String::new();

    //dbg!(&it);

    while let Some((morphene, nb_merged)) = it.next()
    {
        if nb_merged <= 10 { break; }
        _nb += 1;
        output.push_str(&format!("\"{morphene:?}\" x{}\n", nb_merged));
        
        //println!();
        //println!("{nb} : \"{morphene:?}\" merged x{}", nb_merged);
        //println!("{:?}", it.colored().limit(100));
        //dbg!(&it);
    }

    let ex_len = u16::MAX as usize;

    output.push_str("\n\n");
    output.push_str("Exemple of tokenization:");
    output.push_str("\n\n");
    output.push_str(&format!("{:?}", it.colored().limit(ex_len)));

    let path = "./exemple.txt";
    let full_path = std::path::absolute(path).expect("can't absolute path");
    std::fs::write(&full_path, output).expect("failed to save");
    println!("Done at {:?}", full_path);
    
    println!("{:?}", it.colored().limit(ex_len));

}


fn _char_frequency()
{
    // Can be used to guess if there is any separator (like spacing ?)
    let mut char_frequency: HashMap<char, u64> = HashMap::new();
    let input = include_str!("./input/18812.txt");
    let mut total_use = 0;
    for c in  input.chars()
    {
        *char_frequency.entry(c).or_insert(0) += 1;
        total_use += 1;
    }
    let mut v = Vec::from_iter(char_frequency);
    v.sort_by_key(|(_c, nb)| *nb);
    let mut cumulated = 0.;
    let v : Vec<_> = v.into_iter().rev().map(|(c, nb)| { let coef = nb as f64 / total_use as f64; cumulated += coef; (c, nb, coef, cumulated) }).collect();

    for (c, f, usage, usage_cumul) in v 
    {
        println!("{} : {} / {} => {:.4} %, cumulated: {:.4} %", c, f, total_use, usage * 100., usage_cumul * 100.);
    }
}

fn main()
{
    //char_frequency();
    tokenize();
}
