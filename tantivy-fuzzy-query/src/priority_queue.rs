use std::cmp::Ordering;
use std::collections::BinaryHeap;

use tantivy::{DocId, DocSet, Score, SegmentId};
use tantivy::postings::SegmentPostings;

pub struct ScoredSegmentPostings {
    segment_id: SegmentId,
    segment_postings: SegmentPostings,
    score: Score,
}

impl ScoredSegmentPostings {
    pub fn new(segment_id: SegmentId, segment_postings: SegmentPostings, score: Score) -> Self {
        ScoredSegmentPostings {
            segment_id,
            segment_postings,
            score,
        }
    }
}

impl PartialEq for ScoredSegmentPostings {
    fn eq(&self, other: &ScoredSegmentPostings) -> bool {
        self.score == other.score && self.segment_id == other.segment_id
    }
}

impl Eq for ScoredSegmentPostings {}

impl PartialOrd for ScoredSegmentPostings {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Use BinaryHeap as a min-heap
        match other.score.partial_cmp(&self.score) {
            Some(Ordering::Equal) => (),
            Some(not_eq) => return Some(not_eq),
            None => unreachable!(), // score cannot be NaN
        }

        self.segment_id.partial_cmp(&other.segment_id)
    }
}

impl Ord for ScoredSegmentPostings {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap_or(Ordering::Equal)
    }
}

pub struct PriorityQueue {
    heap: BinaryHeap<ScoredSegmentPostings>,
    limit: usize,
    offset: usize,
}


impl PriorityQueue {
    pub fn new(limit: usize, offset: usize) -> Self {
        let capacity = offset + limit;
        assert!(capacity > 0);
        PriorityQueue {
            heap: BinaryHeap::with_capacity(capacity),
            limit,
            offset,
        }
    }

    pub fn enqueue(&mut self, x: ScoredSegmentPostings) {
        if self.heap.len() < self.offset + self.limit {
            self.heap.push(x);
        } else {
            match self.heap.peek_mut() {
                Some(min) if x > *min => {
                    *min = x;
                }
                _ => return,
            }
        }
    }

    pub fn into_sorted_vec(self) -> Vec<ScoredSegmentPostings> {
        // let mut v = self.heap.into_sorted_vec();
        // v.reverse();
        // v

        for ScoredSegmentPostings { segment_postings, score, .. } in self.heap.into_iter() {
            // segment_postings.
        }
    }
}

impl DocSet for PriorityQueue {
    fn advance(&mut self) -> DocId {
        self.heap.pop();
        self.doc()
    }

    fn doc(&self) -> DocId {
        match self.heap.peek() {
            Some(min) => min.doc_id,
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
        self.heap.len() as u32
    }
}

impl Scorer for PriorityQueue {
    fn score(&mut self) -> Score {
        match self.heap.peek() {
            Some(min) => min.score,
            None => panic!(),
        }
    }
}
