use std::cmp::Ordering;
use std::collections::BinaryHeap;

use tantivy::{DocId, DocSet, Score};
use tantivy::query::Scorer;

#[derive(Clone, Copy)]
pub struct ScoredDocId {
    score: Score,
    doc_id: DocId,
}

impl ScoredDocId {
    pub fn new(doc_id: DocId, score: Score) -> Self {
        ScoredDocId {
            score,
            doc_id,
        }
    }
}

impl PartialEq for ScoredDocId {
    fn eq(&self, other: &ScoredDocId) -> bool {
        self.doc_id == other.doc_id && self.score == other.score
    }
}

impl Eq for ScoredDocId {}

impl PartialOrd for ScoredDocId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Use BinaryHeap as a min-heap
        match other.score.partial_cmp(&self.score) {
            Some(Ordering::Equal) => (),
            Some(not_eq) => return Some(not_eq),
            None => unreachable!(), // score cannot be NaN
        }

        self.doc_id.partial_cmp(&other.doc_id)
    }
}

impl Ord for ScoredDocId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap_or(Ordering::Equal)
    }
}

pub struct HeapCollector {
    heap: BinaryHeap<ScoredDocId>,
    limit: usize,
    offset: usize,
}

impl HeapCollector {
    pub fn new(limit: usize, offset: usize) -> Self {
        let capacity = offset + limit;
        assert!(capacity > 0);
        HeapCollector {
            heap: BinaryHeap::with_capacity(capacity),
            limit,
            offset,
        }
    }

    pub fn append<TScorer: Scorer>(&mut self, scorer: &mut TScorer) {
        loop {
            let doc_id = scorer.doc();
            if doc_id == tantivy::TERMINATED {
                break;
            }
            let score = scorer.score();
            self.push(ScoredDocId::new(doc_id, score));
            scorer.advance();
        }
    }

    pub fn push(&mut self, scored_doc_id: ScoredDocId) {
        if self.heap.len() < self.offset + self.limit {
            self.heap.push(scored_doc_id);
        } else {
            match self.heap.peek_mut() {
                Some(mut min) if scored_doc_id.score > min.score => {
                    *min = scored_doc_id;
                }
                _ => return,
            }
        }
    }

    pub fn into_scorer(mut self) -> HeapScorer {
        for _ in 0..self.offset {
            self.heap.pop();
        }

        let mut queue: Vec<_> = self.heap.into_iter().skip(self.offset).take(self.limit).map(|x| (x.doc_id, x.score))
            .collect();
        queue.sort_by_key(|x| x.0);

        HeapScorer {
            queue: VecDeque::from(queue),
        }
    }
}

///////////////////////////////////////////////////////////////////////////


use std::collections::VecDeque;

pub struct HeapScorer {
    queue: VecDeque<(DocId, Score)>,
}

impl DocSet for HeapScorer {
    fn advance(&mut self) -> DocId {
        if self.queue.is_empty() {
            return tantivy::TERMINATED;
        }

        self.queue.pop_front();
        self.doc()
    }

    fn doc(&self) -> DocId {
        match self.queue.front() {
            Some(head) => head.0,
            None => tantivy::TERMINATED,
        }
    }

    // fn seek(&mut self, target: DocId) -> DocId {
    //     match self.score_list.binary_search_by_key(&target, |(id, _)| *id) {
    //         Ok(pos) => {
    //             self.curr = pos;
    //             self.doc()
    //         }
    //         Err(pos) => {
    //             self.curr = pos;
    //             tantivy::TERMINATED
    //         }
    //     }
    // }

    fn size_hint(&self) -> u32 {
        self.queue.len() as u32
    }
}

impl Scorer for HeapScorer {
    fn score(&mut self) -> Score {
        match self.queue.front() {
            Some(head) => head.1,
            None => panic!(),
        }
    }
}
