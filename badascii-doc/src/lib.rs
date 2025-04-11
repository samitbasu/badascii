use badascii_backend::{Options, render::RenderJob, tc::TextCoordinate, text_buffer::TextBuffer};
use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let input = input.token().to_string();
    let input_len = input.len();
    let input = input
        .chars()
        .skip(1)
        .take(input_len - 2)
        .collect::<String>();
    let mut text_buffer = TextBuffer::new(60, 150);
    text_buffer.paste(&input, TextCoordinate { x: 1, y: 1 });
    let job = RenderJob {
        width: 1500.0,
        height: 900.0,
        num_cols: 150,
        num_rows: 60,
        text: text_buffer,
        options: Options::default(),
        x0: 0.0,
        y0: 0.0,
    };
    let svg = badascii_backend::svg::render(&job);
    quote!(#svg).into()
}

#[cfg(test)]
mod tests {
    use syn::parse2;

    use super::*;

    #[test]
    fn it_works() {
        let input = quote! {"
+-----+
      |
      |<-----o 
      |
+-----+
"};
        let svg = my_macro(input.into());
        eprintln!("{}", svg);
    }
}
