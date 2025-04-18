//! This is a CLI to turn text into
//! SVG diagrams using the badascii backend
#![doc = badascii!("
       +-------------+        +-------------+
       | Thing 1     |        | Thing 2     |
       |             |        |             |
  +--->|ins      outs+------->|ins      outs+----+
  |    |             |        |             |    |
  |    |             |        |             |    |
  |    +-------------+        +-------------+    |
  |                                              |
  +----------------------------------------------+
")]
use std::{
    io::{Read, Write, stdin, stdout},
    path::PathBuf,
};

use badascii_doc::badascii;

use clap::Parser;

#[derive(Debug, Parser)]
/// BADASCII CLI
///
/// Convert BadAscii diagrams to SVGs at the command line.
///
struct Args {
    /// The file containing the badascii diagram
    /// If unspecified, then `badascii-cli` will
    /// presume that the input comes via `stdin`.
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// The output file to write the SVG to.  If
    /// unspecified, then the output is written to
    /// `stdout`.
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Use the more formal mode, suitable for
    /// gatherings with canapes.
    #[arg(short, long)]
    formal_mode: bool,
    /// Override the default output width (which is
    /// based on the input buffer multiplied by the
    /// arbitrary scale factor of 10.0)
    #[arg(long)]
    width: Option<f32>,
    /// Override the default output height (which is
    /// based on the input buffer multiplied by the
    /// arbitrary scale factor of 15.0)
    #[arg(long)]
    height: Option<f32>,
    /// Override the color used for the stroke of the
    /// SVG.  By default, a bland gray is used that
    /// will at least show up against both light
    /// and dark mode backgrounds.  But you can override
    /// it here.
    #[arg(short, long)]
    color: Option<String>,
    /// Override the color used for the background of the
    /// SVG.  By default, the SVGs render in dark mode.
    #[arg(short, long)]
    background: Option<String>,
}

fn main() {
    let args = Args::parse();
    let input = if let Some(input) = args.input.as_ref() {
        std::fs::read_to_string(input)
            .unwrap_or_else(|_| panic!("Unable to open input {:?} for reading", input))
    } else {
        let mut ret = String::new();
        stdin()
            .read_to_string(&mut ret)
            .expect("Reading from stdin failed");
        ret
    };
    let buffer = badascii::TextBuffer::with_text(&input);
    let mut job = if args.formal_mode {
        badascii::RenderJob::formal(buffer)
    } else {
        badascii::RenderJob::rough(buffer)
    };
    if let Some(width) = args.width {
        job.width = width;
    }
    if let Some(height) = args.height {
        job.height = height;
    }
    let color = args.color.unwrap_or_else(|| "#808080".to_string());
    let background = args.background.unwrap_or_else(|| "#0A0A0A".to_string());
    let svg = badascii::svg::render(&job, &color, &background);
    if let Some(output) = args.output.as_ref() {
        std::fs::write(output, svg)
            .unwrap_or_else(|_| panic!("Unable to write to output file {}", output.display()));
    } else {
        stdout()
            .write_all(svg.as_bytes())
            .unwrap_or_else(|_| panic!("Unable to write to stdout"))
    }
}
