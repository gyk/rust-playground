use tantivy::{DocId, DocSet, Score, Term};
use tantivy::query::Scorer;

pub struct Disjoint<TScorer> {
    scorers: Vec<TScorer>,
    cursor: Option<usize>,
}

// pub struct Disjoint<TDocSet: DocSet> {
//     doc_sets: Vec<TDocSet>,
// }

impl<TScorer: Scorer> Disjoint<TScorer> {
    pub fn new(scorers: Vec<TScorer>) -> Self {
        let total = scorers.iter().map(TScorer::size_hint).sum::<u32>();
        println!("NEW, len = {}, total len = {}", scorers.len(), total);

        for i in 0..100 {
            println!("- {}", scorers[i].doc());
        }

        assert!(!scorers.is_empty());
        Disjoint {
            scorers,
            cursor: None,
        }
    }

    fn peek(&self) -> (usize, DocId) {
        self.scorers.iter().map(TScorer::doc).enumerate().min_by_key(|(_, doc_id)| *doc_id).unwrap()
    }
}

impl<TScorer: Scorer> Scorer for Disjoint<TScorer> {
    fn score(&mut self) -> Score {
        let cursor = match self.cursor {
            Some(cursor) => cursor,
            None => {
                let (i, _) = self.peek();
                self.cursor.replace(i);
                i
            }
        };

        self.scorers[cursor].score()
    }
}

impl<TScorer: Scorer> DocSet for Disjoint<TScorer> {
    fn advance(&mut self) -> DocId {
        let (i, doc_id) = self.peek();
        self.cursor.replace(i);
        if doc_id < tantivy::TERMINATED {
            return self.scorers[i].advance();
        }
        tantivy::TERMINATED
    }

    fn doc(&self) -> DocId {
        let (_, doc_id) = self.peek();
        doc_id
    }

    fn seek(&mut self, target: DocId) -> DocId {
        let mut min_id = tantivy::TERMINATED;
        for (i, scorer) in self.scorers.iter_mut().enumerate() {
            let id = scorer.seek(target);
            if id < min_id {
                min_id = id;
                self.cursor.replace(i);
            }
        }

        min_id
    }

    fn size_hint(&self) -> u32 {
        self.scorers.iter().map(TScorer::size_hint).sum()
    }
}


