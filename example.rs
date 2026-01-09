//! wrap text into a .xp file
//! ```bash
//! cargo run --example textwrap /path/to/text test.xp 16 8
//! ```
extern crate rexpaint;
extern crate textwrap;

use codepage_437::CP437_WINGDINGS;
use rexpaint::{XpColor, XpFile};
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    // TODO: support some markup language for colorful text !
    let args: Vec<String> = env::args().collect();
    let infile_name = args.get(1).expect("expecting input file name");
    let outfile_name = args.get(2).expect("expecting output file name");
    let width = args
        .get(3)
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(80);
    let height = args
        .get(4)
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(60);
    let mut xp = XpFile::new(width, height);

    println!(
        "generating {}×{} {} from {}",
        width, height, outfile_name, infile_name
    );

    let mut f = File::open(infile_name).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();

    let fgcolor = XpColor::new(255, 255, 255);
    let bgcolor = XpColor::new(0, 0, 0);
    for (y, line) in textwrap::wrap(&contents, width).iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == '\n' {
                continue;
            }
            if let Some(cell) = xp.layers[0].get_mut(x, y) {
                // clip everything that falls outside image
                cell.ch = u32::from(CP437_WINGDINGS.encode(ch).unwrap_or(254));
                cell.fg = fgcolor;
                cell.bg = bgcolor;
            }
        }
    }

    let mut f = File::create(outfile_name).unwrap();
    xp.write(&mut f).unwrap();
}
