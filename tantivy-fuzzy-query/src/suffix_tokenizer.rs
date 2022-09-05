use std::str::CharIndices;

use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

#[derive(Clone, Copy)]
pub struct SuffixTokenizer;

impl Tokenizer for SuffixTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        From::from(SuffixTokenStream {
            text,
            chars: text.char_indices(),
            token: Token::default(),
            state: TokenizingState::Start,
            bitset: 0,
        })
    }
}

enum TokenizingState {
    Start,
    Boundary,
    Upper { pos: Option<usize> },
    Lower,
}

pub struct SuffixTokenStream<'a> {
    text: &'a str,
    chars: CharIndices<'a>,
    token: Token,
    state: TokenizingState,
    bitset: u32,
}

impl<'a> SuffixTokenStream<'a> {
    fn transit(&mut self, next_char: (usize, char)) -> Option<usize> {
        let (offset, ch) = next_char;
        match self.state {
            TokenizingState::Start => {
                self.state = if !ch.is_ascii_alphabetic() {
                    TokenizingState::Boundary
                } else if ch.is_ascii_uppercase() {
                    TokenizingState::Upper { pos: None }
                } else {
                    debug_assert!(ch.is_ascii_lowercase());
                    TokenizingState::Lower
                };

                Some(offset)
            }
            TokenizingState::Boundary => {
                if !ch.is_ascii_alphabetic() {
                    None
                } else if ch.is_ascii_uppercase() {
                    self.state = TokenizingState::Upper { pos: None };
                    Some(offset)
                } else {
                    debug_assert!(ch.is_ascii_lowercase());
                    self.state = TokenizingState::Lower;
                    Some(offset)
                }
            }
            TokenizingState::Upper { pos } => {
                if !ch.is_ascii_alphabetic() {
                    self.state = TokenizingState::Boundary;
                    pos
                } else if ch.is_ascii_uppercase() {
                    self.state = TokenizingState::Upper { pos: Some(offset) };
                    None
                } else {
                    debug_assert!(ch.is_ascii_lowercase());
                    self.state = TokenizingState::Lower;
                    pos
                }
            }
            TokenizingState::Lower => {
                if !ch.is_ascii_alphabetic() {
                    self.state = TokenizingState::Boundary;
                    None
                } else if ch.is_ascii_uppercase() {
                    self.state = TokenizingState::Upper { pos: None };
                    Some(offset)
                } else {
                    debug_assert!(ch.is_ascii_lowercase());
                    None
                }
            }
        }
    }
}

impl<'a> TokenStream for SuffixTokenStream<'a> {
    fn advance(&mut self) -> bool {
        while let Some(next_char) = self.chars.next() {
            if let Some(offset) = self.transit(next_char) {
                let ascii_char = self.text.as_bytes().get(offset).unwrap();
                // It must be an ASCII character except the first one.
                if ascii_char.is_ascii_alphabetic() {
                    let mask = 1 << (ascii_char.to_ascii_lowercase() - b'a');
                    if self.bitset & mask != 0 {
                        continue;
                    }
                    self.bitset |= mask;
                }

                self.token.position = 0;
                self.token.offset_from = offset;
                self.token.offset_to = self.text.len();
                self.token.text.clear();
                self.token.text.push_str(&self.text[offset..]);
                return true;
            }
        }
        false
    }

    fn token(&self) -> &Token {
        &self.token
    }
    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suffix_tokenizer() {
        let s = "O'Reilly Media's RESTful web services, 1st Edition (May 18, 2007)";
        let tokenizer = SuffixTokenizer;
        let mut stream = tokenizer.token_stream(s);

        let mut expected = |s| dbg!(&stream.next().unwrap().text).starts_with(s);

        assert!(expected("O'Reilly"));
        assert!(expected("Reilly"));
        assert!(expected("Media"));
        assert!(expected("s"));
        assert!(expected("Tful"));
        assert!(expected("web"));
        assert!(expected("Edition"));
        assert!(stream.next().is_none());
    }

    #[test]
    fn foo() {
        let s = "Building RESTful Web Services with Spring 5";
        let tokenizer = SuffixTokenizer;
        let mut stream = tokenizer.token_stream(s);

        while let Some(s) = stream.next() {
            println!("~ {}", s.text);
        }

    }
}
