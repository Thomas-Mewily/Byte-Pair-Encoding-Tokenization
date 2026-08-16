use super::*;
use std::{fmt::Display, ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive}};


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteStr(pub [u8]);



impl ByteStr 
{
    pub fn from_ref(slice : &[u8]) -> &Self { <&ByteStr>::from(slice) }
    pub fn from_mut(slice : &mut [u8]) -> &mut Self { <&mut ByteStr>::from(slice) }
}
impl From<&str> for &ByteStr
{
    fn from(value: &str) -> Self {
        Self::from(value.as_bytes())
    }
}
impl From<&[u8]> for &ByteStr
{
    fn from(value: &[u8]) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
impl From<&mut [u8]> for &mut ByteStr
{
    fn from(value: &mut [u8]) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
impl<T: AsRef<[u8]>> PartialEq<T> for ByteStr {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other.as_ref()
    }
}
impl Debug for &ByteStr
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult 
    {
        match str::from_utf8(&self.0)
        {
            //Ok(str) => Display::fmt(&str.escape_default(), f),
            Ok(str) => Display::fmt(str, f),
            Err(_byte) => Debug::fmt(&self.0, f),
        }
    }
}
impl Deref for ByteStr
{
    type Target=[u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ByteStr
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl Index<usize> for ByteStr {
    type Output = u8;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for ByteStr {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

// Slice indexing - returns &ByteStr
impl Index<Range<usize>> for ByteStr {
    type Output = ByteStr;
    
    fn index(&self, range: Range<usize>) -> &Self::Output {
        ByteStr::from_ref(&self.0[range])
    }
}

impl IndexMut<Range<usize>> for ByteStr {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        ByteStr::from_mut(&mut self.0[range])
    }
}

// RangeFrom (e.g., &bytes[5..])
impl Index<RangeFrom<usize>> for ByteStr {
    type Output = ByteStr;
    
    fn index(&self, range: RangeFrom<usize>) -> &Self::Output {
        ByteStr::from_ref(&self.0[range])
    }
}

impl IndexMut<RangeFrom<usize>> for ByteStr {
    fn index_mut(&mut self, range: RangeFrom<usize>) -> &mut Self::Output {
        ByteStr::from_mut(&mut self.0[range])
    }
}

// RangeTo (e.g., &bytes[..5])
impl Index<RangeTo<usize>> for ByteStr {
    type Output = ByteStr;
    
    fn index(&self, range: RangeTo<usize>) -> &Self::Output {
        ByteStr::from_ref(&self.0[range])
    }
}

impl IndexMut<RangeTo<usize>> for ByteStr {
    fn index_mut(&mut self, range: RangeTo<usize>) -> &mut Self::Output {
        ByteStr::from_mut(&mut self.0[range])
    }
}

// RangeFull (e.g., &bytes[..])
impl Index<RangeFull> for ByteStr {
    type Output = ByteStr;
    
    fn index(&self, _range: RangeFull) -> &Self::Output {
        self
    }
}

impl IndexMut<RangeFull> for ByteStr {
    fn index_mut(&mut self, _range: RangeFull) -> &mut Self::Output {
        self
    }
}

// RangeInclusive (e.g., &bytes[2..=5])
impl Index<RangeInclusive<usize>> for ByteStr {
    type Output = ByteStr;
    
    fn index(&self, range: RangeInclusive<usize>) -> &Self::Output {
        ByteStr::from_ref(&self.0[range])
    }
}

impl IndexMut<RangeInclusive<usize>> for ByteStr {
    fn index_mut(&mut self, range: RangeInclusive<usize>) -> &mut Self::Output {
        ByteStr::from_mut(&mut self.0[range])
    }
}

// RangeToInclusive (e.g., &bytes[..=5])
impl Index<RangeToInclusive<usize>> for ByteStr {
    type Output = ByteStr;
    
    fn index(&self, range: RangeToInclusive<usize>) -> &Self::Output {
        ByteStr::from_ref(&self.0[range])
    }
}

impl IndexMut<RangeToInclusive<usize>> for ByteStr {
    fn index_mut(&mut self, range: RangeToInclusive<usize>) -> &mut Self::Output {
        ByteStr::from_mut(&mut self.0[range])
    }
}



// Allow comparing &[u8] with &ByteStr
impl PartialEq<ByteStr> for [u8] {
    fn eq(&self, other: &ByteStr) -> bool {
        self == &other.0
    }
}

impl PartialEq<[u8]> for ByteStr {
    fn eq(&self, other: &[u8]) -> bool {
        &self.0 == other
    }
}

// Also handle references
impl PartialEq<&ByteStr> for [u8] {
    fn eq(&self, other: &&ByteStr) -> bool {
        self == &other.0
    }
}

impl PartialEq<[u8]> for &ByteStr {
    fn eq(&self, other: &[u8]) -> bool {
        &self.0 == other
    }
}

impl PartialEq<&ByteStr> for ByteStr {
    fn eq(&self, other: &&ByteStr) -> bool {
        &self.0 == &other.0
    }
}

impl PartialEq<ByteStr> for &ByteStr {
    fn eq(&self, other: &ByteStr) -> bool {
        &self.0 == &other.0
    }
}

// Compare ByteStr with &str
impl PartialEq<str> for ByteStr {
    fn eq(&self, other: &str) -> bool {
        &self.0 == other.as_bytes()
    }
}

impl PartialEq<ByteStr> for str {
    fn eq(&self, other: &ByteStr) -> bool {
        self.as_bytes() == &other.0
    }
}

impl PartialEq<&ByteStr> for str {
    fn eq(&self, other: &&ByteStr) -> bool {
        self.as_bytes() == &other.0
    }
}

impl PartialEq<ByteStr> for String {
    fn eq(&self, other: &ByteStr) -> bool {
        self.as_bytes() == &other.0
    }
}