use std::time::Instant;

use anyhow::Result;

use fake::faker::lorem::en::Sentence;
use fake::Fake;

use tantivy::collector::{DocSetCollector};
use tantivy::query::RegexQuery;
use tantivy::schema::{Schema, STORED, STRING};
use tantivy::{doc, Index};

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

            {
                let mut index_writer = index.writer(3_000_000)?;
                for _ in 0..1_000_000 {
                    let doc: String = Sentence(5..15).fake();
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

    use std::io::{stdin, stdout, Write};

    loop {
        let mut s = String::new();
        print!("\n\n========\nInput query: ");
        stdout().flush().unwrap();

        let pattern = match stdin().read_line(&mut s) {
            Ok(n) if n > 0 => s.trim_end().to_owned(),
            _ => break,
        };
        println!("Pattern = {:?}", pattern);

        let query = RegexQuery::from_pattern(&pattern, title)?;

        let now = Instant::now();
        let doc_set = searcher.search(&query, &DocSetCollector)?;

        println!(
            "Hit count = {}, duration = {}ms",
            doc_set.len(),
            now.elapsed().as_millis()
        );

        for (i, &doc_id) in doc_set.iter().take(20).enumerate() {
            if let Ok(doc) = searcher.doc(doc_id) {
                if let Some(title) = doc.get_first(title) {
                    println!("Hit #{}: {:?}", i, title.text());
                }
            }
        }
    }

    Ok(())
}
