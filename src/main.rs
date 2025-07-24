use pulldown_cmark::{html, Options, Parser};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use clap::{Arg, Command as ClapCommand};
use indicatif::{ProgressBar, ProgressStyle};

/// Convert markdown content to HTML string with given options.
fn markdown_to_html(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown_input, options);

    let mut html_body = String::new();
    html::push_html(&mut html_body, parser);
    html_body
}

/// Build full HTML document with optional title and CSS (inline or linked).
fn build_html_document(body: &str, title: &str, css_path: Option<&str>) -> io::Result<String> {
    let mut html_output = String::new();
    html_output.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n");
    html_output.push_str(&format!("<title>{}</title>\n", title));
    if let Some(css_file) = css_path {
        match fs::read_to_string(css_file) {
            Ok(css_content) => {
                html_output.push_str("<style>\n");
                html_output.push_str(&css_content);
                html_output.push_str("\n</style>\n");
            }
            Err(_) => {
                html_output.push_str(&format!("<link rel=\"stylesheet\" href=\"{}\">\n", css_file));
            }
        }
    }
    html_output.push_str("</head>\n<body>\n");
    html_output.push_str(body);
    html_output.push_str("\n</body>\n</html>\n");
    Ok(html_output)
}

/// Write content to file with a progress bar.
fn write_file_with_progress(path: &Path, content: &str) -> io::Result<()> {
    let total_size = content.len() as u64;
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {percent}%")
            .expect("Failed to set progress bar template")
            .progress_chars("=> "),
    );

    let mut file = File::create(path)?;
    let bytes = content.as_bytes();
    let chunk_size = 8192;
    let mut written = 0;

    while written < bytes.len() {
        let end = std::cmp::min(written + chunk_size, bytes.len());
        file.write_all(&bytes[written..end])?;
        written = end;
        pb.set_position(written as u64);
    }
    pb.finish_with_message("Done");
    Ok(())
}

/// Open the file in the default system viewer for live preview.
fn open_in_default_viewer(path: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd").args(["/C", "start", path]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).spawn();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = ClapCommand::new("md2html")
        .version("1.0")
        .author("md2html Author")
        .about("Convert Markdown files to HTML")
        .arg(
            Arg::new("input")
                .help("Input markdown file(s)")
                .required(true)
                .min_values(1),
        )
        .arg(
            Arg::new("output")
                .help("Output HTML file(s)")
                .required(true)
                .min_values(1)
                .last(true),
        )
        .arg(
            Arg::new("css")
                .long("css")
                .takes_value(true)
                .help("CSS file to include (inline or linked)"),
        )
        .arg(
            Arg::new("title")
                .long("title")
                .takes_value(true)
                .help("Title of the HTML document"),
        )
        .arg(
            Arg::new("preview")
                .long("preview")
                .takes_value(false)
                .help("Open the output HTML file(s) in default viewer"),
        )
        .get_matches();

    let input_files: Vec<_> = matches.values_of("input").unwrap().collect();
    let output_files: Vec<_> = matches.values_of("output").unwrap().collect();

    if input_files.len() != output_files.len() {
        eprintln!("Number of input and output files must match.");
        std::process::exit(1);
    }

    let css_path = matches.value_of("css");
    let title = matches.value_of("title");

    let preview = matches.is_present("preview");

    for (input_file, output_file) in input_files.iter().zip(output_files.iter()) {
        let markdown_input = fs::read_to_string(input_file)?;
        let html_body = markdown_to_html(&markdown_input);
        let doc_title = title.unwrap_or(input_file);
        let html_output = build_html_document(&html_body, doc_title, css_path)?;

        write_file_with_progress(Path::new(output_file), &html_output)?;

        println!("Conversion successful: {} -> {}", input_file, output_file);

        if preview {
            open_in_default_viewer(output_file);
        }
    }

    Ok(())
}
