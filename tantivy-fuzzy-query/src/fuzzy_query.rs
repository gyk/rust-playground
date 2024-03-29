use tantivy::{DocId, DocSet, Score};

use tantivy::query::{Explanation, Query, Scorer, Weight};
use tantivy::schema::{Field, IndexRecordOption};
use tantivy::Searcher;
use tantivy::SegmentReader;

#[derive(Debug, Clone)]
pub struct SublimeFuzzyQuery {
    pattern: String,
    field: Field,
}

impl SublimeFuzzyQuery {
    /// Creates a new SublimeFuzzyQuery from a given pattern
    pub fn new(pattern: &str, field: Field) -> Self {
        SublimeFuzzyQuery {
            pattern: pattern.to_owned(),
            field,
        }
    }

    fn specialized_weight(&self) -> FuzzyWeight {
        FuzzyWeight {
            pattern: self.pattern.clone(),
            field: self.field,
        }
    }
}

impl Query for SublimeFuzzyQuery {
    fn weight(
        &self,
        _searcher: &Searcher,
        _scoring_enabled: bool,
    ) -> tantivy::Result<Box<dyn Weight>> {
        Ok(Box::new(self.specialized_weight()))
    }
}

pub struct FuzzyWeight {
    pattern: String,
    field: Field,
}

use std::collections::HashMap;

pub struct FuzzyScorer {
    score_list: Vec<(DocId, Score)>,
    curr: usize,
}

impl Weight for FuzzyWeight {
    fn scorer(&self, reader: &SegmentReader, boost: Score) -> tantivy::Result<Box<dyn Scorer>> {
        // let max_doc = reader.max_doc();
        // let mut doc_bitset = BitSet::with_max_value(max_doc);
        let inverted_index = reader.inverted_index(self.field)?;
        let term_dict = inverted_index.terms();
        let mut term_stream = term_dict.stream()?;

        let mut score_map: HashMap<DocId, Score> = HashMap::default();

        while term_stream.advance() {
            let term_info = term_stream.value();
            let term_ord = term_stream.term_ord();

            let mut term_buffer = Vec::with_capacity(64);
            let found = term_dict.ord_to_term(term_ord, &mut term_buffer)?;
            if !found {
                continue;
            }
            let term = unsafe { String::from_utf8_unchecked(term_buffer) };
            let score = match sublime_fuzzy::best_match(&self.pattern, &term) {
                Some(matched) => matched.score() as f32 * boost,
                None => continue,
            };
            println!("{} -> {}", term, score);

            // let mut segment_postings = inverted_index
            //     .read_postings(term, IndexRecordOption::Basic)?;
            // if let Some(segment_postings) = segment_postings {
            //     segment_postings
            // }

            let mut block_segment_postings = inverted_index
                .read_block_postings_from_terminfo(term_info, IndexRecordOption::Basic)?;
            loop {
                let docs = block_segment_postings.docs();
                if docs.is_empty() {
                    break;
                }
                for &doc in docs {
                    println!("    id = {}", doc);
                    score_map.insert(doc, score);
                }
                block_segment_postings.advance();
            }
        }

        let mut score_list = Vec::with_capacity(score_map.len());
        use itertools::Itertools;
        for key in score_map.keys().sorted() {
            score_list.push((*key as u32, score_map[key]));
        }

        println!("score_list len {}", score_list.len());

        Ok(Box::new(FuzzyScorer {
            score_list,
            curr: 0,
        }))
    }

    fn explain(&self, _reader: &SegmentReader, _doc: DocId) -> tantivy::Result<Explanation> {
        Ok(Explanation::new("No comments", 1.0))
    }
}

impl DocSet for FuzzyScorer {
    fn advance(&mut self) -> DocId {
        self.curr = std::cmp::min(self.curr + 1, self.score_list.len());
        self.doc()
    }

    fn doc(&self) -> DocId {
        if self.curr >= self.score_list.len() {
            return tantivy::TERMINATED;
        }
        self.score_list[self.curr].0
    }

    fn seek(&mut self, target: DocId) -> DocId {
        match self.score_list.binary_search_by_key(&target, |(id, _)| *id) {
            Ok(pos) => {
                self.curr = pos;
                self.doc()
            }
            Err(pos) => {
                self.curr = pos;
                tantivy::TERMINATED
            }
        }
    }

    fn size_hint(&self) -> u32 {
        self.score_list.len() as u32
    }
}

impl Scorer for FuzzyScorer {
    fn score(&mut self) -> Score {
        self.score_list[self.curr].1
    }
}
