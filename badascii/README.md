# BADASCII

This crate provides a backend library for processing BADASCII strings.
These strings represent simple block diagrams written using basic
ASCII characters. Like this:
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

To use this as a library, the API is pretty simple:

```rust
use badascii::{RenderJob, TextBuffer};

// Create a text buffer from the string
let text_buffer = TextBuffer::with_text(" +---> ");
// Create a render job
let job = RenderJob::rough(text_buffer);
// Render it with a white text color
let svg = badascii::svg::render(&job, "white");
// Rejoice!  `svg` contains a string with the SVG
```

## Other stuff

You can use `badascii` in your RustDoc generated comments 
using the `badascii-doc` crate.  You can also edit `badascii`
diagrams using the GUI editor at `badascii-gui`.  And
eventually at the website I plan to set up.  Last but not
least, there is a mdbook preprocessor at `badascii-mdbook`.

## Alternates

There are many.  The most sophisticated might be `svgbob`.  

