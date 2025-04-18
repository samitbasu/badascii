use mdbook::{
    BookItem,
    book::{Book, Chapter},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

// The BadAscii preprocessor.
pub struct BadAscii;
fn create_svg_html(s: &str) -> String {
    let tb = badascii::TextBuffer::with_text(s);
    let job = badascii::RenderJob::rough(tb);
    // TODO - figure out light vs dark mode for MDBook?
    let svg = badascii::svg::render(&job, "#808080", "#0A0A0A");
    format!("\n\n<pre>{svg}</pre>\n")
}
impl BadAscii {
    fn process_chapter(chapter: &mut Chapter) {
        let parser = pulldown_cmark::Parser::new(&chapter.content);
        let mut buf = String::with_capacity(chapter.content.len() + 128);
        // Inspired by svgbob2 mdbook preprocessor.

        let mut in_block = false;
        let mut diagram = String::new();
        let events = parser.filter_map(|event| match (&event, in_block) {
            (
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("badascii")))),
                false,
            ) => {
                in_block = true;
                diagram.clear();
                None
            }
            (Event::Text(content), true) => {
                diagram.push_str(content);
                None
            }
            (Event::End(TagEnd::CodeBlock), true) => {
                in_block = false;
                Some(Event::Html(create_svg_html(&diagram).into()))
            }
            _ => Some(event),
        });
        pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();
        chapter.content = buf;
    }
}

impl Preprocessor for BadAscii {
    fn name(&self) -> &str {
        "badascii-mdbook"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        if let Some(nop_cfg) = ctx.config.get_preprocessor(self.name()) {
            if nop_cfg.contains_key("blow-up") {
                anyhow::bail!("Boom!!1!");
            }
        }
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                Self::process_chapter(chapter);
            }
        });

        // we *are* a no-op preprocessor after all
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_md_passthrough() {
        let md = r##"
# Chapter 1

Here is a diagram of the mascot.
```badascii
  +----+
  |  OO|
  +----+
```
        "##;
        let mut chapter = Chapter {
            name: "Test".into(),
            content: md.into(),
            number: None,
            sub_items: vec![],
            path: None,
            source_path: None,
            parent_names: vec![],
        };
        BadAscii::process_chapter(&mut chapter);
        let expect = expect_test::expect_file!["test.md"];
        expect.assert_eq(&chapter.content);
    }

    #[test]
    fn badascii_preprocessor_run() {
        let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "nop": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let input_json = input_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let expected_book = book.clone();
        let result = BadAscii.run(&ctx, book);
        assert!(result.is_ok());

        // The nop-preprocessor should not have made any changes to the book content.
        let actual_book = result.unwrap();
        assert_eq!(actual_book, expected_book);
    }
}
