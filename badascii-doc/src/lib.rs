use badascii::{Options, render::RenderJob, text_buffer::TextBuffer};
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
pub fn badascii_smooth(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let text_buffer = get_text_buffer(input);
    let size = text_buffer.size();
    let options = Options {
        disable_multi_stroke: Some(true),
        max_randomness_offset: Some(0.0),
        roughness: Some(0.0),
        ..Default::default()
    };
    let job = RenderJob {
        width: (size.num_cols * 10) as f32,
        height: (size.num_rows * 15) as f32,
        num_cols: size.num_cols,
        num_rows: size.num_rows,
        text: text_buffer,
        options,
        x0: 0.0,
        y0: 0.0,
    };
    let svg = badascii::svg::render(&job, "currentColor");
    let svg = format!("<p>{svg}</p>");
    quote!(#svg).into()
}

#[proc_macro]
pub fn badascii(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let text_buffer = get_text_buffer(input);
    let size = text_buffer.size();
    let job = RenderJob {
        width: (size.num_cols * 10) as f32,
        height: (size.num_rows * 15) as f32,
        num_cols: size.num_cols,
        num_rows: size.num_rows,
        text: text_buffer,
        options: Options::default(),
        x0: 0.0,
        y0: 0.0,
    };
    let svg = badascii::svg::render(&job, "currentColor");
    let svg = format!("<p>{svg}</p>");
    quote!(#svg).into()
}
