use clap::{Parser, Subcommand, ValueEnum};
use fromansi::{generate_css, parse_ansi, rexpaint_to_ansi};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fromansi")]
#[command(about = "Parse ANSI escape sequences")]
struct Args {
    /// Input file (reads from stdin if not provided) - for terminal output
    input: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate HTML output
    Html {
        /// Input file (reads from stdin if not provided)
        input: Option<PathBuf>,

        /// Output type
        #[arg(short, long, default_value = "fragment")]
        output: HtmlOutputType,

        /// Filter out cells of a specific color (hex format, e.g., #000000)
        #[arg(long)]
        filter: Option<String>,
    },
    /// Convert `RexPaint` file to ANSI text
    Rex {
        /// Input file (reads from stdin if not provided)
        input: Option<PathBuf>,
    },
    /// Generate CSS styles
    Css,
}

#[derive(Clone, ValueEnum)]
enum HtmlOutputType {
    Fragment,
    Standalone,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Handle output
    match args.command {
        None => {
            // Read input
            let input = if let Some(input_path) = &args.input {
                fs::read_to_string(input_path)?
            } else {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            };
            // Terminal output
            print!("{input}");
        }
        Some(Commands::Html {
            input,
            output,
            filter,
        }) => {
            // Read input
            let input = if let Some(input_path) = &input {
                fs::read_to_string(input_path)?
            } else {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            };
            let parsed = parse_ansi(&input);
            let html = parsed.to_html_with_filter(filter.as_deref());
            match output {
                HtmlOutputType::Fragment => {
                    println!("{html}");
                }
                HtmlOutputType::Standalone => {
                    let css = generate_css();
                    let full_html = format!(
                        "<!DOCTYPE html><html><head><style>{css}</style></head><body>{html}</body></html>"
                    );
                    println!("{full_html}");
                }
            }
        }
        Some(Commands::Rex { input }) => {
            // Read RexPaint data
            let data = if let Some(input_path) = &input {
                fs::read(input_path)?
            } else {
                let mut buffer = Vec::new();
                io::stdin().read_to_end(&mut buffer)?;
                buffer
            };
            let ansi = rexpaint_to_ansi(&data)?;
            print!("{ansi}");
        }
        Some(Commands::Css) => {
            let css = generate_css();
            println!("{css}");
            // No debug for CSS since no input parsed
        }
    }

    Ok(())
}
