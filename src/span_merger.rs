use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    pub begin: usize,
    pub len: usize,
}
impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}..{}", self.begin, self.end())
    }
}

impl Span {
    pub fn end(&self) -> usize {
        self.begin + self.len
    }
    pub fn set_end(&mut self, end: usize) {
        self.len = end - self.begin;
    }
    pub fn with_added_len(mut self, extra_len: usize) -> Self {
        self.len += extra_len;
        self
    }

    pub fn get<'a>(&self, input: &'a ByteStr) -> &'a ByteStr {
        ByteStr::from_ref(&input[self.begin..self.begin + self.len])
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub type SpanID = usize;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MorphenePairEntry {
    pub suffix_spans_id: HashSet<SpanID>,
}
impl MorphenePairEntry {
    pub fn nb(&self) -> usize {
        self.suffix_spans_id.len()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum InputKind {
    Text,
    Byte,
}
impl InputKind {
    pub const fn is_text(&self) -> bool {
        matches!(self, InputKind::Text)
    }
    pub const fn is_byte(&self) -> bool {
        matches!(self, InputKind::Byte)
    }
}

#[derive(Debug)]
pub struct SpanMerger<'a> {
    pub input_kind: InputKind,
    pub input: &'a ByteStr,
    /// Span are never removed.
    /// Instead they are replaced by a Span with a 0 len.
    /// All the span are always contiguous (relative to the input) and do not overlap together.
    pub spans: Vec<Span>, //OldNextSpans,
    /// Morphene encoder
    pub pairs: HashMap<&'a MorphenePair, MorphenePairEntry>,
    /// Will not merge these pair / single char.
    /// Stuff like separator / space / dot for ex
    pub single_morphene: HashSet<Vec<u8>>,
    tmp_span_id: Vec<SpanID>,
}

impl<'a> SpanMerger<'a> {
    pub fn colored<'b>(&'b self) -> SpanColored<'b> {
        SpanColored {
            input: self.input,
            span: &self.spans,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpanColored<'a> {
    pub input: &'a ByteStr,
    pub span: &'a [Span],
}
impl<'a> SpanColored<'a> {
    pub fn limit(self, max_byte: usize) -> Self {
        let end_slice_idx = self.span.partition_point(|s| s.end() <= max_byte);
        Self {
            input: &self.input[0..max_byte.min(self.input.len())],
            span: &self.span[0..end_slice_idx],
        }
    }
}
impl Debug for SpanColored<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use hexga_ansi_color::*;
        let rainbow = [
            AnsiColorKind::Blue,
            AnsiColorKind::Red,
            AnsiColorKind::Yellow,
            AnsiColorKind::Green,
            AnsiColorKind::Magenta,
        ];
        let mut rainbow_idx = 0;
        writeln!(f, "{} ", AnsiColorKind::Black.foreground())?;

        for s in self.span.iter() {
            if s.is_empty() {
                continue;
            }
            if s.end() >= self.input.len() {
                break;
            }
            rainbow_idx += 1;
            write!(
                f,
                "{}{:?}",
                rainbow[rainbow_idx % rainbow.len()].background(),
                s.get(self.input)
            )?;
        }
        writeln!(
            f,
            "{}{} ",
            AnsiColorKind::White.foreground(),
            AnsiColorKind::Black.background()
        )
    }
}

impl<'a> SpanMerger<'a> {
    /// Return a sorted vec by frequency of the tokens
    pub fn morphemes_vec(&'a self) -> Vec<(&'a ByteStr, usize)> {
        let mut v: Vec<(&ByteStr, usize)> =
            self.morphemes_map().iter().map(|(m, f)| (*m, *f)).collect();
        v.sort_by_key(|(_m, f)| usize::MAX - *f);
        v
    }

    pub fn morphemes_map_to_owned(&'a self) -> HashMap<Vec<u8>, usize> {
        self.morphemes_map()
            .iter()
            .map(|(k, v)| ((**k).to_owned(), *v))
            .collect()
    }

    pub fn morphemes_map(&'a self) -> HashMap<&'a ByteStr, usize> {
        let mut tokens = HashMap::new();
        for s in &self.spans {
            if s.is_empty() {
                continue;
            }
            *tokens.entry(s.get(self.input)).or_default() += 1;
        }
        tokens
    }

    pub fn reset_span(&mut self) {
        self.spans.clear();

        match self.input_kind {
            InputKind::Text => {
                let Ok(s) = str::from_utf8(self.input) else {
                    self.input_kind = InputKind::Byte;
                    return self.reset_span();
                };
                self.spans.extend(s.char_indices().map(|(idx, ch)| Span {
                    begin: idx,
                    len: ch.len_utf8(), // 1 for ASCII, 2 for ô, 3 for é, etc.
                }));
            }
            InputKind::Byte => {
                self.spans.extend(
                    self.input
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| Span { begin: idx, len: 1 }),
                );
            }
        }
    }

    pub fn reset_merge_pair(&mut self) {
        self.pairs.clear();

        for i in 0..self.spans.len() - 1 {
            let suffix = self.spans[i];
            let preffix = self.spans[i + 1];

            if self.single_morphene.contains(&**suffix.get(self.input)) {
                continue;
            }
            if self.single_morphene.contains(&**preffix.get(self.input)) {
                continue;
            }

            let merged = Span {
                begin: suffix.begin,
                len: suffix.len + preffix.len,
            };

            let key = merged.get(self.input);
            let entry = self
                .pairs
                .entry(key)
                .or_insert_with(|| MorphenePairEntry::default());
            entry.suffix_spans_id.insert(i);
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
    }

    fn new<T>(value: T, input_kind: InputKind) -> Self
    where
        T: Into<&'a ByteStr>,
    {
        let value = value.into();
        let mut s = Self {
            input: value,
            spans: Vec::new(),
            pairs: HashMap::new(),
            tmp_span_id: Vec::new(),
            single_morphene: HashSet::new(),
            input_kind,
        };
        s.reset_span();
        s.reset_merge_pair();
        s
    }

    pub fn from_bytes<T>(value: T) -> Self
    where
        T: Into<&'a ByteStr>,
    {
        Self::new(value, InputKind::Byte)
    }

    pub fn from_text<T>(value: T) -> Self
    where
        T: Into<&'a ByteStr>,
    {
        let s = Self::new(value, InputKind::Text);
        assert!(s.input_kind.is_text());
        s
    }
}

pub type Morphene = ByteStr;
pub type MorphenePair = ByteStr;
//pub type NextMorpheneEntry<'a, 'entry> = (&'a Morphene, &'entry MorpheneEntry);

impl<'a> SpanMerger<'a> {
    pub fn most_frequent_morphene_pair(&self) -> Option<(&'a MorphenePair, &MorphenePairEntry)> {
        let mut max_so_far = 0;
        let Some((merge_pair, entry)) = self
            .pairs
            .iter()
            .max_by_key(|(merge_pair, entry)| {
                if entry.nb() < max_so_far || self.single_morphene.contains(&****merge_pair) {
                    0
                } else {
                    max_so_far = max_so_far.max(entry.nb());
                    entry.nb()
                }
            })
            .map(|(b, f)| (*b, f))
        else {
            return None;
        };

        Some((merge_pair, entry))
    }

    /// ```md
    /// prev prefix suffix next
    ///      |||||||||||||
    ///  aka morphene_pair / merged_pair
    /// ```
    pub fn merge_morphene(&mut self, morphene_pair: &MorphenePair) -> Result<NbMerged, ()> {
        // println!("{}", morphene_pair.escape_ascii());
        //dbg!(&morphene_pair.escape_ascii());
        //dbg!(&self.pair_frequency);

        let Some(entry) = self.pairs.remove(morphene_pair) else {
            return Err(());
        };

        let merged_len = morphene_pair.len();

        let mut nb_merged = 0;

        let spans_id = &mut self.tmp_span_id;
        spans_id.clear();
        spans_id.extend(entry.suffix_spans_id.into_iter());
        spans_id.sort(); // Force to merge in a determinist way, left to right

        for prefix_id in spans_id.iter().copied() {
            let prefix_span = self.spans[prefix_id];

            // Was already merged by a nears by morphene in this loop
            if prefix_span.len == 0 {
                continue;
            }

            let prefix_len = prefix_span.len;
            let suffix_len = merged_len - prefix_len;
            let merged_span = prefix_span.with_added_len(suffix_len);

            // TODO : optimize the dead span (len 0) to indicate how far the next non dead span is, thus reducing the nb of iteration for
            // computing the prev_id and next_id

            let prev_id = {
                let mut tmp_prev_idx = prefix_id;
                loop {
                    if tmp_prev_idx == 0 {
                        break None;
                    }
                    tmp_prev_idx -= 1;
                    if self.spans[tmp_prev_idx].len != 0 {
                        break Some(tmp_prev_idx);
                    }
                }
            };
            let (suffix_id, next_id) = {
                let mut suffix_id = None;
                let mut next_id = None;
                let mut tmp_next_idx = prefix_id;
                loop {
                    if tmp_next_idx == self.spans.len() - 1 {
                        break;
                    }
                    tmp_next_idx += 1;

                    let cur = self.spans[tmp_next_idx];
                    if cur.len != 0 {
                        if suffix_id.is_none() {
                            suffix_id = Some(tmp_next_idx);
                        } else {
                            next_id = Some(tmp_next_idx);
                            break;
                        }
                    }
                }
                (suffix_id.expect("no preffix?"), next_id)
            };
            let suffix_span = self.spans[suffix_id];

            if let Some(prev_id) = prev_id {
                // Not first, can merge with prev morphene
                let prev = self.spans[prev_id];
                let prev_pair = prev.with_added_len(prefix_len);
                debug_assert_eq!(prev.end(), prefix_span.begin);

                let prev_is_single_morphene =
                    self.single_morphene.contains(&**prev.get(self.input));

                match self.pairs.get_mut(prev_pair.get(self.input)) {
                    Some(entry) => {
                        entry.suffix_spans_id.remove(&prev_id);
                    }
                    None => {
                        // maybe unreachable() I'm not sure
                        if !prev_is_single_morphene {
                            assert_eq!(prev_pair.get(self.input), morphene_pair);
                        }
                    }
                }

                if !prev_is_single_morphene {
                    let new_prev_pair = prev.with_added_len(merged_len);
                    self.pairs
                        .entry(new_prev_pair.get(self.input))
                        .or_insert_with(|| MorphenePairEntry::default())
                        .suffix_spans_id
                        .insert(prev_id);
                }
            }

            if let Some(next_id) = next_id {
                // Not last, can merge with next morphene
                let next = self.spans[next_id];
                debug_assert_eq!(next.begin, suffix_span.end());
                let suffix_pair = suffix_span.with_added_len(next.len);

                let next_is_single_morphene =
                    self.single_morphene.contains(&**next.get(self.input));

                match self.pairs.get_mut(suffix_pair.get(self.input)) {
                    Some(entry) => {
                        entry.suffix_spans_id.remove(&suffix_id);
                    }
                    None => {
                        if !next_is_single_morphene {
                            assert_eq!(suffix_pair.get(self.input), morphene_pair);
                        }
                    }
                }

                if !next_is_single_morphene {
                    let new_next_pair = merged_span.with_added_len(next.len);
                    self.pairs
                        .entry(new_next_pair.get(self.input))
                        .or_insert_with(|| MorphenePairEntry::default())
                        .suffix_spans_id
                        .insert(prefix_id);
                }
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

impl<'a> Iterator for SpanMerger<'a> {
    type Item = (&'a Morphene, NbMerged);
    fn next(&mut self) -> Option<Self::Item> {
        let (morphene, _entry) = self.most_frequent_morphene_pair()?;
        let nb_merged = self.merge_morphene(morphene).ok()?;
        if nb_merged == 0 {
            return None;
        }
        Some((morphene, nb_merged))
    }
}
