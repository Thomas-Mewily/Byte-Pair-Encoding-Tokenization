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

mod language_analysis;
pub use language_analysis::*;

//mod sub_slice;
//pub use sub_slice::*;

