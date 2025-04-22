use mdbook::{
    BookItem,
    book::{Book, Chapter},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

// The BadAscii preprocessor.
pub struct BadAscii;

fn create_svg_html(formal_mode: bool, s: &str) -> String {
    let tb = badascii::TextBuffer::with_text(s);
    let job = if !formal_mode {
        badascii::RenderJob::rough(tb)
    } else {
        badascii::RenderJob::formal(tb)
    };
    // TODO - figure out light vs dark mode for MDBook?
    let svg = badascii::svg::render(&job, "currentColor", "none");
    format!("\n\n<pre>{svg}</pre>\n")
}
impl BadAscii {
    fn process_chapter(formal_mode: bool, chapter: &mut Chapter) {
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
                Some(Event::Html(create_svg_html(formal_mode, &diagram).into()))
            }
            _ => Some(event),
        });
        pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();
        chapter.content = buf;
    }
}

impl Preprocessor for BadAscii {
    fn name(&self) -> &str {
        "badascii"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        let formal_mode = if let Some(nop_cfg) = &ctx.config.get_preprocessor(self.name()) {
            nop_cfg.contains_key("formal")
        } else {
            false
        };
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                Self::process_chapter(formal_mode, chapter);
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
    fn conversion_works() {
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
        BadAscii::process_chapter(false, &mut chapter);
        let expect = expect_test::expect_file!["test.md"];
        expect.assert_eq(&chapter.content);
    }

    #[test]
    fn test_formal_mode_works() {
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
        BadAscii::process_chapter(true, &mut chapter);
        let expect = expect_test::expect_file!["test_formal.md"];
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
        let result = BadAscii.run(&ctx, book);
        assert!(result.is_ok());
        let expected_book = expect_test::expect_file!["test.txt"];
        let actual_book = result.unwrap();
        expected_book.assert_debug_eq(&actual_book);
    }
}
