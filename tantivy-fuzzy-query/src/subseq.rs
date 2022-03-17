use tantivy_fst::Automaton;

/// An automaton that matches (case-insensitively) if the input contains a specific subsequence.
#[derive(Clone, Debug)]
pub struct Subsequence<'a> {
    subseq: &'a [u8],
}

impl<'a> Subsequence<'a> {
    /// Constructs an automaton that matches input containing the specified subsequence.
    #[inline]
    pub fn new(subsequence: &'a str) -> Subsequence<'a> {
        Subsequence {
            subseq: subsequence.as_bytes(),
        }
    }
}

impl<'a> Automaton for Subsequence<'a> {
    type State = usize;

    #[inline]
    fn start(&self) -> usize {
        0
    }

    #[inline]
    fn is_match(&self, &state: &usize) -> bool {
        state == self.subseq.len()
    }

    #[inline]
    fn can_match(&self, _: &usize) -> bool {
        true
    }

    #[inline]
    fn will_always_match(&self, &state: &usize) -> bool {
        state == self.subseq.len()
    }

    #[inline]
    fn accept(&self, &state: &usize, byte: u8) -> usize {
        if state == self.subseq.len() {
            return state;
        }
        state + (byte.eq_ignore_ascii_case(&self.subseq[state])) as usize
    }
}
