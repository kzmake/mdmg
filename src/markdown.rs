use comrak::nodes::{AstNode, NodeValue};
use crate::Result;
use crate::scaffold::Scaffold;
use comrak::{parse_document, Arena, ComrakOptions};

pub fn parse(markdown: &str) -> Result<Vec<Scaffold>> {
    let arena = Arena::new();
    let doc = parse_document(&arena, &markdown, &ComrakOptions::default());

    let mut scaffolds: Vec<Scaffold> = vec![];

    fn iter_nodes<'a, F>(node: &'a AstNode<'a>, scaffolds: &'a mut Vec<Scaffold>, f: &F) -> &'a mut Vec<Scaffold>
    where
        F: Fn(&'a AstNode<'a>, &'a mut Vec<Scaffold>) -> &'a mut Vec<Scaffold>,
    {
        let mut scaffolds = f(node, scaffolds);
        for c in node.children() {
            scaffolds = iter_nodes(c, scaffolds, f);
        }
        scaffolds
    }

    let scaffolds = iter_nodes(doc, &mut scaffolds, &|node, scaffolds| {
        let ast = node.data.clone().into_inner().value;
        match ast {
            NodeValue::Heading(c) if c.level == 2 => {}
            _ => {}
        }

        if let NodeValue::Text(txt_vec) = node.data.clone().into_inner().value {
            if txt_vec.len() == 0 {
                return scaffolds
            }
            let title_txt = String::from_utf8(txt_vec);
            if let Ok(file_name) = title_txt {
                scaffolds.push(Scaffold::Pending { file_name })
            }
        }

        if let NodeValue::CodeBlock(ncb) = node.data.clone().into_inner().value {
            if ncb.literal.len() == 0 {
                return scaffolds
            }

            let code_block_txt = String::from_utf8(ncb.literal);
            let code_block = code_block_txt.expect("code block text is not found");
            let pending_scaffold_position_opt = scaffolds.iter().position(|scaffold| -> bool {
                if let Scaffold::Pending { file_name: _file_name } = scaffold {
                    return true;
                }
                false
            });
            if let Some(pending_scaffold_position) = pending_scaffold_position_opt {
                let pending_scaffold= scaffolds.swap_remove(pending_scaffold_position);
                if let Scaffold::Pending { file_name }  = pending_scaffold {
                    scaffolds.push(Scaffold::Complete {
                        file_name,
                        file_body: code_block
                    })
                }
            }
        }
        return scaffolds
    });

    Ok(scaffolds.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_is_return_to_one_scaffold() {
        let markdown =
r#"
## src/foobar.rs
```rust
use crate::Result;

fn something() -> Result<String> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use suepr::*;

    #[test]
    fn test_something() {
    }
}
```
"#;
        let scaffolds = parse(&markdown).unwrap();
        let file_body = 
r#"use crate::Result;

fn something() -> Result<String> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use suepr::*;

    #[test]
    fn test_something() {
    }
}
"#.to_string();
        assert_eq!(scaffolds, vec![Scaffold::Complete { file_name: "src/foobar.rs".to_string(), file_body }])
    }
}
