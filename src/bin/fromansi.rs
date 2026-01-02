use clap::Parser;
use fromansi::{OutputType, parse_ansi};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fromansi")]
#[command(about = "Parse ANSI escape sequences")]
struct Args {
    /// Input file (reads from stdin if not provided)
    input: Option<PathBuf>,

    /// Output type
    #[arg(short, long, default_value = "terminal")]
    output: OutputType,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read input
    let input = if let Some(input_path) = &args.input {
        fs::read_to_string(input_path)?
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let output_type = args.output;

    // Handle output
    match output_type {
        OutputType::Terminal => {
            print!("{}", input);
        }
        OutputType::HtmlFragment => {
            let parsed = parse_ansi(&input);
            let html = parsed.to_html();
            println!("{}", html);
        }
        OutputType::HtmlStandalone => {
            let parsed = parse_ansi(&input);
            let html = parsed.to_html();
            let css = fs::read_to_string("static/styles.css")?;
            let full_html = format!("<!DOCTYPE html><html><head><style>{}</style></head><body>{}</body></html>", css, html);
            println!("{}", full_html);
        }
    }

    Ok(())
}
