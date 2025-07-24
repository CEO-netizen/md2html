use pulldown_cmark::{html, Options, Parser};
use std::process::Command;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

// Function to print the progress bar
fn print_progress_bar(completed: usize, total: usize) {
    let bar_width = 40;
    let progress = completed * bar_width / total;
    let bar = "█".repeat(progress) + &" ".repeat(bar_width - progress);
    print!("\r[{}] {}%", bar, (completed * 100 / total).min(100));
    let _ = io::stdout().flush();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input_markdown_file(s).md> <output_html_file(s).html> [--css <css_file>] [--title <title>] [--preview]", args[0]);
        return Ok(());
    }

    // Parse optional arguments
    let mut css_path: Option<String> = None;
    let mut title: Option<String> = None;
    let mut preview = false;
    let mut input_files = Vec::new();
    let mut output_files = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--css" => {
                if i + 1 < args.len() {
                    css_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Missing value for --css");
                    return Ok(());
                }
            }
            "--title" => {
                if i + 1 < args.len() {
                    title = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Missing value for --title");
                    return Ok(());
                }
            }
            "--preview" => {
                preview = true;
                i += 1;
            }
            s if s.ends_with(".md") => {
                input_files.push(s.to_string());
                if i + 1 < args.len() && args[i + 1].ends_with(".html") {
                    output_files.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Missing output HTML file for input: {}", s);
                    return Ok(());
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    if input_files.is_empty() || output_files.is_empty() || input_files.len() != output_files.len() {
        eprintln!("Provide matching input and output files.");
        return Ok(());
    }

    for (input_file_path_str, output_file_path_str) in input_files.iter().zip(output_files.iter()) {
        let input_file_path = Path::new(input_file_path_str);
        let output_file_path = Path::new(output_file_path_str);
        let markdown_input = fs::read_to_string(input_file_path)?;

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&markdown_input, options);

        let mut html_body = String::new();
        html::push_html(&mut html_body, parser);

        // Build HTML boilerplate
        let doc_title = title.clone().unwrap_or_else(|| input_file_path_str.clone());
        let mut html_output = String::new();
        html_output.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n");
        html_output.push_str(&format!("<title>{}</title>\n", doc_title));
        if let Some(css_file) = &css_path {
            if let Ok(css_content) = fs::read_to_string(css_file) {
                html_output.push_str("<style>\n");
                html_output.push_str(&css_content);
                html_output.push_str("\n</style>\n");
            } else {
                html_output.push_str(&format!("<link rel=\"stylesheet\" href=\"{}\">\n", css_file));
            }
        }
        html_output.push_str("</head>\n<body>\n");
        html_output.push_str(&html_body);
        html_output.push_str("\n</body>\n</html>\n");

        let total_steps = html_output.len().max(1);
        let mut file = File::create(output_file_path)?;
        let chunk_size = total_steps / 100.max(1);
        for (i, chunk) in html_output.as_bytes().chunks(chunk_size.max(1)).enumerate() {
            file.write_all(chunk)?;
            print_progress_bar((i + 1) * chunk_size, total_steps);
        }
        println!("\nConversion successful: {} -> {}", input_file_path_str, output_file_path_str);

        // Live preview
        if preview {
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("xdg-open").arg(output_file_path_str).spawn();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("cmd").args(["/C", "start", output_file_path_str]).spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open").arg(output_file_path_str).spawn();
            }
        }
    }
    Ok(())
}

