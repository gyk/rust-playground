use pest::Parser;

use crate::PathItem;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct JsonPathParser;

pub fn parse(expression: &str) -> Vec<PathItem> {
    let mut json_path = Vec::new();
    let mut pairs = JsonPathParser::parse(Rule::expression, expression).unwrap();
    let items = pairs.next().unwrap();
    for item in items.into_inner() {
        match item.as_rule() {
            Rule::child |
            Rule::first_child |
            Rule::sub_child |
            Rule::single_quoted_child |
            Rule::double_quoted_child => {
                let identity = item.into_inner().next().unwrap().as_str().to_owned();
                json_path.push(PathItem::Child(identity));
            }

            Rule::indexed_child => {
                let index: isize = item.into_inner().next().unwrap().as_str().parse().unwrap();
                json_path.push(PathItem::Index(index));
            }

            _rule => (),
        }
    }

    json_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() {
        use PathItem::*;

        // smoke
        assert_eq!(
            parse("foo[42]['bar.baz'].qux[quux]"),
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
            parse("[foo].-bar[--baz][+42][-1][-][-100u]"),
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
    }
}
