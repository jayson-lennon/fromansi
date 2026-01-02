# fromansi

A Rust library and command-line tool for rendering ANSI escape sequences.

## Description

fromansi parses terminal input. It supports:

- Standard 16-color ANSI palette
- 256-color extended palette
- Truecolor (24-bit RGB)
- Text styles: bold, italic, underline, strikethrough, blink, dim, hidden, reverse
- Output to HTML with CSS styling
- Conversion from RexPaint files to ANSI text

## Installation

Arch Linux:

```bash
git clone https://github.com/jayson-lennon/fromansi.git
cd fromansi
makepkg -si
```

Build from source:

```bash
git clone https://github.com/jayson-lennon/fromansi.git
cd fromansi
cargo build --release
```

## Usage

### Command Line

#### Basic usage

```bash
# Parse ANSI from stdin and output to stdout
echo -e "\x1b[31mRed text\x1b[0m" | fromansi

# Parse from file
fromansi input.txt
```

#### Generate HTML

```bash
# HTML fragment
echo -e "\x1b[32mGreen\x1b[0m" | fromansi html

# Standalone HTML page
echo -e "\x1b[32mGreen\x1b[0m" | fromansi html --output standalone
```

#### Convert RexPaint to ANSI

```bash
# Convert to HTML, loading from file.
fromansi rex input.xp | fromansi html

# Convert to HTML, piping from stdin
cat input.xp | fromansi rex | fromansi html
```

#### Generate CSS

When using the "fragment" (default) HTML rendering mode, you can use this command to generate the required CSS for your page.

```bash
fromansi css > styles.css
```

## License

[GPLv3](LICENSE)
