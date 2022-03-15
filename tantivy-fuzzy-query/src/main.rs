use std::io::{stdin, stdout, Write};
use std::time::Instant;

use anyhow::Result;

use fake::faker::name::en::Name;
use fake::Fake;

use tantivy::collector::TopDocs;
use tantivy::schema::{Schema, STORED, STRING};
use tantivy::{doc, Index};

mod fuzzy_query;

use crate::fuzzy_query::*;

fn main() -> Result<()> {
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", STRING | STORED);
    let schema = schema_builder.build();

    let path = r".\tantivy-index";
    let _ = std::fs::create_dir_all(&path);

    let index = match Index::open_in_dir(path) {
        Ok(index) => index,
        Err(..) => {
            let index = Index::create_in_dir(path, schema)?;

            let doc_count = 200;

            {
                let mut index_writer = index.writer(3_000_000)?;
                for _ in 0..doc_count {
                    let doc: String = Name().fake();

                    index_writer.add_document(doc!(
                        title => doc,
                    ));
                }

                index_writer.commit()?;
            }

            index
        }
    };

    let reader = index.reader()?;
    let searcher = reader.searcher();

    loop {
        let mut s = String::new();
        print!("\n\n========\nInput query: ");
        stdout().flush().unwrap();

        let pattern = match stdin().read_line(&mut s) {
            Ok(n) if n > 0 => s.trim_end().to_owned(),
            _ => break,
        };
        println!("Pattern = {:?}", pattern);

        let query = SublimeFuzzyQuery::new(&pattern, title);

        let now = Instant::now();
        let doc_set = searcher.search(&query, &TopDocs::with_limit(20))?;

        println!(
            "Hit count = {}, duration = {}ms",
            doc_set.len(),
            now.elapsed().as_millis()
        );

        for (i, (score, doc_id)) in doc_set.iter().enumerate() {
            if let Ok(doc) = searcher.doc(*doc_id) {
                if let Some(title) = doc.get_first(title) {
                    println!(
                        "Hit #{} (id = {}, score = {}): {:?}",
                        i,
                        doc_id.doc_id,
                        score,
                        title.text()
                    );
                }
            }
        }
    }

    Ok(())
}
