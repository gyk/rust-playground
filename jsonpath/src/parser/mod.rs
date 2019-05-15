use pest::Parser;

use crate::PathItem;
use crate::error::{Error, Result};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct JsonPathParser;

pub fn parse(expression: &str) -> Result<Vec<PathItem>> {
    let mut json_path = Vec::new();
    let mut pairs = JsonPathParser::parse(Rule::expression, expression)?;
    let items = pairs.next().ok_or_else(|| Error::PairsNextItemError)?;
    for item in items.into_inner() {
        match item.as_rule() {
            Rule::child |
            Rule::first_child |
            Rule::sub_child |
            Rule::single_quoted_child |
            Rule::double_quoted_child => {
                let identity = item
                    .into_inner()
                    .next()
                    .ok_or_else(|| Error::PairsNextItemError)?
                    .as_str()
                    .to_owned();
                json_path.push(PathItem::Child(identity));
            }

            Rule::indexed_child => {
                let index: isize = item
                    .into_inner()
                    .next()
                    .ok_or_else(|| Error::PairsNextItemError)?
                    .as_str()
                    .parse()?;
                json_path.push(PathItem::Index(index));
            }

            _rule => (),
        }
    }

    Ok(json_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() -> Result<()> {
        use PathItem::*;

        // smoke
        assert_eq!(
            parse("foo[42]['bar.baz'].qux[quux]")?,
            &[
                Child("foo".into()),
                Index(42),
                Child("bar.baz".into()),
                Child("qux".into()),
                Child("quux".into())
            ][..]
        );

        // leading brackets, negative index, index with explicit '+' sign
        assert_eq!(
            parse("[foo].-bar[--baz][+42][-1][-][-100u]")?,
            &[
                Child("foo".into()),
                Child("-bar".into()),
                Child("--baz".into()),
                Index(42),
                Index(-1),
                Child("-".into()),
                Child("-100u".into()),
            ][..]
        );

        Ok(())
    }
}
