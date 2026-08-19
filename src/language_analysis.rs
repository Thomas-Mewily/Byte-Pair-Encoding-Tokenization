use super::*;

pub type Coef = f64;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Morpheme
{
    data : MorpheneData,
    nb   : usize,
    coef : Coef,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum MorpheneData<L=MorphemeLetter,C=MorphemeCombinaison>
{
    Letter(L),
    Combined(C)
}

pub type MorphemeLetter = u8;
pub type MorphemeCombinaison = Vec<MorphemeID>;
pub type MorphemeID = usize;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Morphemes
{
    /// Order = ID. Stored by most frequent.
    morpheme : Vec<Morpheme>,
    morpheme_encoder : HashMap<Vec<u8>, MorphemeID>,
}

//impl Extend<(&Morph)

impl Morphemes
{
    pub fn new() -> Self { Self::default() }
}

pub type Corpus = Vec<TrainingData>;


/*
pub struct Learning
{
    pub language : LanguageData,
    pub corpus : Corpus,
}

impl Deref for Learning
{
    type Target = LanguageData;
    fn deref(&self) -> &Self::Target {
        &self.language
    }
}
impl DerefMut for Learning
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.language
    }
}
*/


#[derive(Clone, PartialEq, Debug)]
pub struct TrainingData
{
    pub data : Vec<u8>,
    pub credit : Credits,
} 

#[derive(Clone, PartialEq, Debug)]
pub struct Credits
{
    author : Option<String>,
    name   : Option<String>,
    years  : Option<i32>,
    url    : Option<String>
}


/*

Token/Morpheme same idea here

Todo : for each tokens
- assign some ID. Unknow token ID.
- find the most related and unreladed token nears by


french

Détokenizer ↑ / Tokeniser ↓ :


- r e m e r c i e r e m e n t
- re merci er ement
- remerciement

C h a t
Chat

le morphemelettre "C" est similaire au "c". Donc "Chat" similaire à "chat" 

*/