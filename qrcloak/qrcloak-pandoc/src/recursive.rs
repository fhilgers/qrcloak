use crate::Code;

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub inline: &'a mut pandoc_ast::Inline,
    pub code: Code,
}

pub fn codes<'a>(blcks: impl Iterator<Item = &'a mut pandoc_ast::Block>) -> Vec<CodeBlock<'a>> {
    let flattened = blocks(blcks);

    let mut v = Vec::new();

    for block in flattened {
        match block {
            pandoc_ast::Block::CodeBlock(attrs, data) => {
                if let Some(code) = Code::new_if_marked(attrs, data) {
                    *block = pandoc_ast::Block::Plain(vec![pandoc_ast::Inline::SoftBreak]);

                    let inline = match block {
                        pandoc_ast::Block::Plain(inline) => &mut inline[0],
                        _ => panic!("should be a para"),
                    };

                    v.push(CodeBlock { inline, code });
                }
            }
            other_block => {
                let flattened_inlines = inlines(other_block);

                for inline in flattened_inlines {
                    if let pandoc_ast::Inline::Code(attrs, data) = inline {
                        if let Some(code) = Code::new_if_marked(attrs, data) {
                            v.push(CodeBlock { inline, code });
                        }
                    }
                }
            }
        };
    }

    v
}

pub fn inlines(block: &mut pandoc_ast::Block) -> Vec<&mut pandoc_ast::Inline> {
    let mut v = Vec::new();
    match block {
        pandoc_ast::Block::Plain(inline) => v.extend(inline),
        pandoc_ast::Block::Para(inline) => v.extend(inline),
        pandoc_ast::Block::LineBlock(inline) => v.extend(inline.iter_mut().flatten()),
        pandoc_ast::Block::CodeBlock(_, _) => {}
        pandoc_ast::Block::RawBlock(_, _) => {}
        pandoc_ast::Block::BlockQuote(_) => {}
        pandoc_ast::Block::OrderedList(_, _) => {}
        pandoc_ast::Block::BulletList(_) => {}
        pandoc_ast::Block::DefinitionList(abc) => v.extend(abc.iter_mut().flat_map(|(i, _)| i)),
        pandoc_ast::Block::Figure(_, _, _) => {}
        pandoc_ast::Block::Header(_, _, inline) => v.extend(inline),
        pandoc_ast::Block::HorizontalRule => {}
        pandoc_ast::Block::Table(_, _, _, _, _, _) => {} // TODO: handle table
        pandoc_ast::Block::Div(_, _) => {}
        pandoc_ast::Block::Null => {}
    };

    v
}

pub fn blocks<'a>(
    blcks: impl Iterator<Item = &'a mut pandoc_ast::Block>,
) -> impl Iterator<Item = &'a mut pandoc_ast::Block> {
    blcks.flat_map(|block| {
        let mut v = Vec::new();

        match block {
            pandoc_ast::Block::Plain(_) => v.push(block),
            pandoc_ast::Block::Para(_) => v.push(block),
            pandoc_ast::Block::LineBlock(_) => v.push(block),
            pandoc_ast::Block::CodeBlock(_, _) => v.push(block),
            pandoc_ast::Block::RawBlock(_, _) => v.push(block),
            pandoc_ast::Block::BlockQuote(b) => v.extend(b),
            pandoc_ast::Block::OrderedList(_, lb) => v.extend(lb.iter_mut().flatten()),
            pandoc_ast::Block::BulletList(lb) => v.extend(lb.iter_mut().flatten()),
            pandoc_ast::Block::DefinitionList(abc) => {
                v.extend(abc.iter_mut().flat_map(|(_, b)| b).flatten())
            }
            pandoc_ast::Block::Figure(_, _, b) => v.extend(b),
            pandoc_ast::Block::Header(_, _, _) => v.push(block),
            pandoc_ast::Block::HorizontalRule => v.push(block),
            pandoc_ast::Block::Table(_, _, _, _, _, _) => v.push(block), // TODO: handle table
            pandoc_ast::Block::Div(_, b) => v.extend(b),
            pandoc_ast::Block::Null => v.push(block),
        };
        v
    })
}
