// This part was coded by an LLM :
// It's currently not used yet

pub trait AllSubSlice {
    fn all_sub_slice(&self) -> Vec<&Self>;
}
// For &[T] - returns Vec<&[T]>
impl<T> AllSubSlice for [T] {
    fn all_sub_slice(&self) -> Vec<&[T]> {
        let len = self.len();
        let mut result = Vec::with_capacity(len * (len + 1) / 2 - 1);

        // Iterate by length first (shorter to longer)
        for length in 1..len {
            for start in 0..=(len - length) {
                let end = start + length;
                // Skip the full slice (length == len)
                if length == len {
                    continue;
                }
                result.push(&self[start..end]);
            }
        }

        result
    }
}

// For &str - uses char indices for proper UTF-8 handling
impl AllSubSlice for str {
    fn all_sub_slice(&self) -> Vec<&str> {
        let chars: Vec<char> = self.chars().collect();
        let char_count = chars.len();

        if char_count == 0 {
            return Vec::new();
        }

        // Pre-calculate byte positions for each char index
        let mut byte_positions = Vec::with_capacity(char_count + 1);
        let mut byte_offset = 0;
        byte_positions.push(0);

        for c in &chars {
            byte_offset += c.len_utf8();
            byte_positions.push(byte_offset);
        }

        let mut result = Vec::with_capacity(char_count * (char_count + 1) / 2 - 1);

        // Iterate by character length first (shorter to longer)
        for length in 1..=char_count {
            // Skip the full slice
            if length == char_count {
                continue;
            }

            for start_char in 0..=(char_count - length) {
                let end_char = start_char + length;
                let byte_start = byte_positions[start_char];
                let byte_end = byte_positions[end_char];

                // SAFETY: byte_start and byte_end are at UTF-8 character boundaries
                unsafe {
                    result.push(self.get_unchecked(byte_start..byte_end));
                }
            }
        }

        result
    }
}
