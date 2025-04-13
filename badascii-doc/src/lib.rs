use badascii::{render::RenderJob, text_buffer::TextBuffer};
use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

fn get_text_buffer(input: LitStr) -> TextBuffer {
    let input = input.token().to_string();
    let input_len = input.len();
    let input = input
        .chars()
        .skip(1)
        .take(input_len - 2)
        .collect::<String>();
    TextBuffer::with_text(&input)
}

#[proc_macro]
pub fn badascii_formal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let text_buffer = get_text_buffer(input);
    let job = RenderJob::formal(text_buffer);
    let svg = badascii::svg::render(&job, "currentColor");
    let svg = format!("<p>{svg}</p>");
    quote!(#svg).into()
}

#[proc_macro]
pub fn badascii(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let text_buffer = get_text_buffer(input);
    let job = RenderJob::rough(text_buffer);
    let svg = badascii::svg::render(&job, "currentColor");
    let svg = format!("<p>{svg}</p>");
    quote!(#svg).into()
}
