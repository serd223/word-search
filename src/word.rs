
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// The letters hashmap contains a character and the 'weight' of that character in the string.
pub struct Word<'a> {
    pub str_repr: &'a str,
    pub letters: HashMap<char, f32> 
}

use std::fmt;
impl fmt::Display for Word<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str_repr)
    }
}

impl<'a> Word<'a> {
    pub fn from_str(s: &'a str) -> Self {
        let mut letters = HashMap::new();
        let one_div_len = 1. / s.chars().count() as f32;
        for c in s.chars() {
            if c == ' ' {
                continue;
            }

            match letters.get_mut(&c) {
                Some(v) => *v += one_div_len,
                None => { letters.insert(c, one_div_len); }
            }
        }
        
        Self {
            str_repr: s,
            letters: letters
        }
    }
}