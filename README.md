# BadAscii 

What is `badascii`?  It's a diagram that is drawn with simple
ASCII characters that can be used to represent simple block
diagrams.  For example:

```
       +-------------+        +-------------+
       | Thing 1     |        | Thing 2     |
       |             |        |             |
  +--->|ins      outs+------->|ins      outs+----+
  |    |             |        |             |    |
  |    |             |        |             |    |
  |    +-------------+        +-------------+    |
  |                                              |
  +----------------------------------------------+
```
and convert them into SVG images like this:

![SVG of diagram](https://github.com/samitbasu/badascii/blob/main/badascii/example.svg)

A set of Rust Crates that implement the BadAscii system for
capturing block diagrams in comments and plain text files
and then rendering them into SVGs.  This repository contains
the following crates:

- `badascii` - contains the pure rendering logic.  This crate
takes a string representation of the block diagram, and generates
the drawing commands needed to render it.  It also provides an
SVG backend that can be used to generate an SVG diagram as a 
string.
- `badascii-cli` - a simple CLI that wraps the library and provides
a way to easily generate diagrams on the command line.  You can
customize the generated output using command line options.
- `badascii-doc` - a couple of proc-macros that allow you to put
`badascii` diagrams into comments in your Rust code, and then
have the rendered diagrams included in the output of your `rustdoc`.
- `badascii-gui` - a GUI application to makes editing `badascii` 
diagrams simple and fun.  Written in `egui`.  You can run this
locally on your machine, or you can use the web version.
- `badascii-mdbook` - a preprocessor plugin for `mdbook` that 
allows you to include `badascii` diagrams in your mdbook, and then
render them to SVG.

The intent of BadAscii is not to replace other, more sophisticated
tools out there.  It's to provide a way to build block diagrams that
are:

- Legible in a code editor
- Editable with no special tools
- Meant to show basic blocks and their relationships
- Flexible enough to do all the things, but not complicated.



TODO

- [ ] Fix background on SVGs
- [ ] Add help link?
