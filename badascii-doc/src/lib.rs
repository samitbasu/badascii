use badascii::{render::RenderJob, text_buffer::TextBuffer};
use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

fn strip_outer(x: &str) -> String {
    let x = x
        .chars()
        .skip_while(|&c| c != '"')
        .skip(1)
        .collect::<String>();
    let x = x
        .chars()
        .rev()
        .skip_while(|&c| c != '"')
        .skip(1)
        .collect::<String>();
    x.chars().rev().collect()
}

fn get_text_buffer(input: LitStr) -> TextBuffer {
    let input = input.token().to_string();
    let input = strip_outer(&input);
    TextBuffer::with_text(&input)
}

#[proc_macro]
pub fn badascii_formal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let text_buffer = get_text_buffer(input);
    let job = RenderJob::formal(text_buffer);
    let svg = badascii::svg::render(&job, "currentColor", "none");
    let svg = format!("<p></p><div style=\"text-align:center;\">{svg}</div><p></p>");
    quote!(#svg).into()
}

#[proc_macro]
pub fn badascii(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let text_buffer = get_text_buffer(input);
    let job = RenderJob::rough(text_buffer);
    let svg = badascii::svg::render(&job, "currentColor", "none");
    let svg = format!("<p></p><div style=\"text-align:center;\">{svg}</div><p></p>");
    quote!(#svg).into()
}
