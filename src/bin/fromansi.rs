use clap::Parser;
use fromansi::OutputType;
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
    output: String,
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

    // Parse output type
    let output_type = match args.output.as_str() {
        "terminal" => OutputType::Terminal,
        _ => return Err(format!("Unknown output type: {}", args.output).into()),
    };

    // Handle output
    match output_type {
        OutputType::Terminal => {
            print!("{}", input);
        }
    }

    Ok(())
}
