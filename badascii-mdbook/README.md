# BadAscii-mdbook

A preprocessor for `mdbook` to add `badascii` support.

It turns a badascii diagram like this:

>```badascii
>       +-------------+        +-------------+
>       | Thing 1     |        | Thing 2     |
>       |             |        |             |
>  +--->|ins      outs+------->|ins      outs+----+
>  |    |             |        |             |    |
>  |    |             |        |             |    |
>  |    +-------------+        +-------------+    |
>  |                                              |
>  +----------------------------------------------+
>```

into an embedded SVG like this:

![SVG of diagram](https://raw.githubusercontent.com/samitbasu/badascii/refs/heads/main/badascii/example.svg)


in your book.

## Installation

# From source

To install from source

```shell
cargo install --locked badascii-mdbook
```
This will install the `badascii-mdbook` binary from source.

## Configuration

You need to configure your `mdbook` to use the preprocessor.  This 
requires adding the following to your `book.toml`

```toml
[preprocessor.badascii]
command = "badascii-mdbook"
```

Then you can build your book

```shell
mdbook build path/to/book
```