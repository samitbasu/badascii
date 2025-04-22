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
This will install the `mdbook-badascii` binary from source.

## Configuration

You need to configure your `mdbook` to use the preprocessor.  This 
requires adding the following to your `book.toml`

```toml
[preprocessor.badascii]
```

If you also want the formal mode for diagrams, you can include a config
flag in the `book.toml`.

```toml
[preprocessor.badascii]
formal = true
```

This will convert all of the diagrams using `formal` mode.

Then you can build your book

```shell
mdbook build path/to/book
```
