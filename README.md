# md2html

Version 0.1.0

A command-line tool written in Rust to convert Markdown files to HTML files. Supports optional CSS styling, custom HTML title, and live preview of the generated HTML.

## Usage

```bash
md2html <input_markdown_file(s).md> <output_html_file(s).html> [--css <css_file>] [--title <title>] [--preview]
```

- Convert one or more Markdown files to HTML.
- Optionally specify a CSS file to style the output.
- Optionally specify a custom HTML title.
- Use `--preview` to open the generated HTML file in the default browser.

## Dependencies

- [clap](https://crates.io/crates/clap) for command-line argument parsing.
- [pulldown-cmark](https://crates.io/crates/pulldown-cmark) for Markdown parsing.

## License

This project is licensed under the GPL 3 License. See the LICENSE file for details.
