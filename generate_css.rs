fn main() {
    println!("/* ANSI Color Styles for fromansi HTML output */");

    // Text styles
    println!(".bold {{ font-weight: bold; }}");
    println!(".italic {{ font-style: italic; }}");
    println!(".underline {{ text-decoration: underline; }}");
    println!(".strikethrough {{ text-decoration: line-through; }}");
    println!(".dim {{ opacity: 0.5; }}");
    println!(".blink {{ animation: blink 1s infinite; }}");
    println!("@keyframes blink {{ 0%, 50% {{ opacity: 1; }} 51%, 100% {{ opacity: 0; }} }}");
    println!(".reverse {{ /* Note: reverse is handled by swapping fg/bg in HTML generation */ }}");
    println!(".hidden {{ visibility: hidden; }}");

    // Standard 16 colors
    let standard_colors = [
        "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0",
        "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
    ];

    for i in 0..16 {
        println!(".fg-{} {{ color: {}; }}", i, standard_colors[i]);
        println!(".bg-{} {{ background-color: {}; }}", i, standard_colors[i]);
    }

    // Color cube 16-231
    for i in 16..232 {
        let r = ((i - 16) / 36) * 51;
        let g = (((i - 16) % 36) / 6) * 51;
        let b = ((i - 16) % 6) * 51;
        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        println!(".fg-{} {{ color: {}; }}", i, hex);
        println!(".bg-{} {{ background-color: {}; }}", i, hex);
    }

    // Grayscale 232-255
    for i in 232..256 {
        let gray = 8 + (i - 232) * 10;
        let hex = format!("#{:02x}{:02x}{:02x}", gray, gray, gray);
        println!(".fg-{} {{ color: {}; }}", i, hex);
        println!(".bg-{} {{ background-color: {}; }}", i, hex);
    }
}