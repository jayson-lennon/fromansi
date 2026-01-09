use clap::{Parser, Subcommand, ValueEnum};
use error_stack::fmt::ColorMode;
use error_stack::{Report, ResultExt};
use fromansi::{ansi_to_rexpaint, generate_css, parse_ansi, rexpaint_to_ansi};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use wherror::Error;

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
    /// Convert ANSI text to `RexPaint` file
    ToRex {
        /// Input file (reads from stdin if not provided)
        input: Option<PathBuf>,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate CSS styles
    Css,
}

/// The output type for HTML rendering.
#[derive(Clone, ValueEnum)]
enum HtmlOutputType {
    /// Just the <pre> block.
    Fragment,
    /// An entire webpage
    Standalone,
}

/// Top-level application error
#[derive(Debug, Error)]
#[error(debug)]
pub struct AppError;

fn read_text_input(input: Option<PathBuf>) -> Result<String, Report<AppError>> {
    if let Some(input_path) = input {
        fs::read_to_string(&input_path)
            .change_context(AppError)
            .attach_with(|| format!("failed to read input file '{}'", input_path.display()))
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .change_context(AppError)
            .attach("failed to read stdin")?;
        Ok(buffer)
    }
}

fn read_binary_input(input: Option<PathBuf>) -> Result<Vec<u8>, Report<AppError>> {
    if let Some(input_path) = input {
        fs::read(&input_path)
            .change_context(AppError)
            .attach_with(|| format!("failed to read input file '{}'", input_path.display()))
    } else {
        let mut buffer = Vec::new();
        io::stdin()
            .read_to_end(&mut buffer)
            .change_context(AppError)
            .attach("failed to read stdin")?;
        Ok(buffer)
    }
}

fn main() -> Result<(), Report<AppError>> {
    let args = Args::parse();
    Report::set_color_mode(ColorMode::Color);

    // Handle output
    match args.command {
        None => {
            let input = read_text_input(args.input)?;
            print!("{input}");
        }
        Some(Commands::Html {
            input,
            output,
            filter,
        }) => {
            let input = read_text_input(input)?;
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
            let data = read_binary_input(input)?;
            let ansi = rexpaint_to_ansi(&data)
                .change_context(AppError)
                .attach("RexPaint conversion failed")?;
            print!("{ansi}");
        }
        Some(Commands::ToRex { input, output }) => {
            let input_text = read_text_input(input)?;
            let xp_data = ansi_to_rexpaint(&input_text)
                .change_context(AppError)
                .attach("ANSI to RexPaint conversion failed")?;
            fs::write(&output, xp_data)
                .change_context(AppError)
                .attach_with(|| format!("failed to write output file '{}'", output.display()))?;
        }
        Some(Commands::Css) => {
            let css = generate_css();
            println!("{css}");
            // No debug for CSS since no input parsed
        }
    }

    Ok(())
}
