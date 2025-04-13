# BADASCII Command Line Interface

A super simple tool to convert BadAscii diagrams from their text representation
into SVG.  

The usage is straightforward.  If you want to, you can install it with

```shell
cargo install badascii-cli
```

```shell
BADASCII CLI

Convert BadAscii diagrams to SVGs at the command line.

Usage: badascii-cli [OPTIONS]

Options:
  -i, --input <INPUT>
          The file containing the badascii diagram If unspecified, then `badascii-cli` will presume that the input comes via `stdin`

  -o, --output <OUTPUT>
          The output file to write the SVG to.  If unspecified, then the output is written to `stdout`

  -f, --formal-mode
          Use the more formal mode, suitable for gatherings with canapes

      --width <WIDTH>
          Override the default output width (which is based on the input buffer multiplied by the arbitrary scale factor of 10.0)

      --height <HEIGHT>
          Override the default output height (which is based on the input buffer multiplied by the arbitrary scale factor of 15.0)

  -c, --color <COLOR>
          Override the color used for the stroke of the SVG.  By default, a bland gray is used that will at least show up against both light and dark mode backgrounds.  But you can override it here

  -h, --help
          Print help (see a summary with '-h')
```