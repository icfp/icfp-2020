// From: https://github.com/pankdm/pegovka-messages
// Cargo.toml:
// [dependencies]
// image = "0.23.6"
// lazy_static = "1.4.0"
//
// Usage
// decode input file:
//    cargo run input_file output_file
// show all supported symbols:
//    cargo run -- --show-all

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;

use image::Rgb;

#[macro_use]
extern crate lazy_static;

const ZOOM: usize = 8;
const SHIFT: usize = 2;

lazy_static! {
    static ref SYMBOLS: HashMap<i32, &'static str> = [
        (0, "ap"),
        (12, "=="),
        (417, "inc"),
        (401, "dec"),
        (365, "sum"),
    ]
    .iter()
    .copied()
    .collect();
}

struct Svg {
    file: File,
}

#[allow(unused)]
impl Svg {
    pub fn new(output_file: &String, width: usize, height: usize) -> Svg {
        let mut file = File::create(output_file).unwrap();
        file.write_all(
            format!(
                "<svg xmlns='http://www.w3.org/2000/svg' version='1.1' width='{}' height='{}'>\n",
                width * ZOOM,
                height * ZOOM,
            )
            .as_bytes(),
        );
        file.write_all("<rect width='100%' height='100%' style='fill:black'/>\n".as_bytes());
        Svg { file: file }
    }

    pub fn close(&mut self) {
        self.file.write_all("</svg>".as_bytes());
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: String) {
        self.file.write_all(
            format!(
                "<rect x='{}' y='{}' width='7' height='7' style='fill:{}'/>\n",
                x * ZOOM,
                y * ZOOM,
                color
            )
            .as_bytes(),
        );
    }
    pub fn add_raw_annotation(
        &mut self,
        x: usize,
        y: usize,
        dx: usize,
        dy: usize,
        text: &String,
        control_bit: bool,
    ) {
        let color = if control_bit { "yellow" } else { "green" };
        self.file.write_all(
            format!(
                "<rect x='{}' y='{}' width='{}' height='{}' style='fill:{};opacity:0.5'/>\n",
                x * ZOOM - SHIFT,
                y * ZOOM - SHIFT,
                dx * ZOOM + 2 * SHIFT,
                dy * ZOOM + 2 * SHIFT,
                color,
            )
            .as_bytes(),
        );
        let options = "dominant-baseline='middle' text-anchor='middle' fill='white'";
        let style_options = [
            "paint-order: stroke;",
            "fill: white;",
            "stroke: black;",
            "stroke-width: 2px;",
            "font: 18px bold sans;",
        ]
        .join(" ");

        self.file.write_all(
            format!(
                "<text x='{}' y='{}' {} style='{}'>{}</text>\n",
                x * ZOOM + (dx / 2) * ZOOM,
                y * ZOOM + (dy / 2) * ZOOM,
                options,
                style_options,
                text,
            )
            .as_bytes(),
        );
    }

    pub fn add_annotation(
        &mut self,
        x: usize,
        y: usize,
        dx: usize,
        dy: usize,
        value: i32,
        control_bit: bool,
    ) {
        let text = Svg::annotation_text(control_bit, value);
        self.add_raw_annotation(x, y, dx, dy, &text, control_bit);
    }

    fn annotation_text(control_bit: bool, value: i32) -> String {
        if !control_bit {
            return value.to_string();
        }
        let text = SYMBOLS.get(&value).unwrap_or(&"");
        return if text == &"" {
            format!(":{}", value)
        } else {
            text.to_string()
        };
    }
}

fn value_to_svg_color(value: u8) -> String {
    match value {
        0 => "#333333".to_string(),
        1 => "white".to_string(),
        _ => {
            panic!("Unexpected pixel: {:?}", value);
        }
    }
}

fn rgb_to_value(pixel: &Rgb<u8>) -> u8 {
    match pixel {
        Rgb([0, 0, 0]) => 0,
        Rgb([255, 255, 255]) => 1,
        _ => {
            panic!("Unexpected pixel: {:?}", pixel);
        }
    }
}

type Image = Vec<Vec<u8>>;
type BooleanGrid = Vec<Vec<bool>>;

struct ImageWrapper {
    image: Image,
    height: usize,
    width: usize,
}

enum ParseResult {
    None,
    Glyph {
        dx: usize,
        dy: usize,
        value: i32,
        control_bit: bool,
    },
}

fn try_parse_symbol(iw: &ImageWrapper, x: usize, y: usize) -> ParseResult {
    let image = &iw.image;
    if image[x + 1][y] == 1 && image[x][y + 1] == 1 && image[x - 1][y] == 0 && image[x][y - 1] == 0
    {
        // control bit
        let control_bit = image[x][y] == 1;

        // Find proper delta
        let mut delta = 1;
        // 10 as limit should be good enough
        while delta < 10 {
            if x + delta >= iw.width || y + delta >= iw.height {
                break;
            }
            if image[x + delta][y] != 1 || image[x][y + delta] != 1 {
                break;
            }
            delta += 1;
        }

        // Calculate overall value
        let mut value = 0 as i32;
        for cy in 0..(delta - 1) {
            for cx in 0..(delta - 1) {
                if image[x + 1 + cx][y + 1 + cy] == 1 {
                    value += 1 << (cy * (delta - 1) + cx) as i32;
                }
            }
        }
        let extra_bit = image[x][y + delta];
        // extra bit indicate negative numbers
        if extra_bit == 1 {
            value = -value;
        }

        return ParseResult::Glyph {
            dx: delta,
            dy: delta + extra_bit as usize,
            value: value,
            control_bit: control_bit,
        };
    } else {
        return ParseResult::None;
    }
}

fn mark_parsed(parsed: &mut BooleanGrid, x: usize, y: usize, dx: usize, dy: usize) {
    for cdx in 0..dx {
        for cdy in 0..dy {
            parsed[x + cdx][y + cdy] = true;
        }
    }
}

fn parse_image(iw: &ImageWrapper, parsed: &mut BooleanGrid, svg: &mut Svg) {
    println!("Parsing image...");
    // skip boundaries
    for y in 1..(iw.height - 2) {
        for x in 1..(iw.width - 2) {
            if parsed[x][y] {
                continue;
            }
            let parse_result = try_parse_symbol(iw, x, y);
            match parse_result {
                ParseResult::None => continue,
                ParseResult::Glyph {
                    dx,
                    dy,
                    value,
                    control_bit,
                } => {
                    mark_parsed(parsed, x, y, dx, dy);
                    svg.add_annotation(x, y, dx, dy, value, control_bit);
                }
            }
        }
    }
    println!("Done");
}

fn empty_image(width: usize, height: usize) -> Image {
    let mut image = Vec::new();
    for _ in 0..width {
        image.push(vec![0; height]);
    }
    image
}

fn encode_symbol(value: i32) -> Image {
    assert!(value >= 0);
    // println!("Encoding {}", value);
    let ln = if value == 0 {
        1.0
    } else {
        (value as f32).log2().ceil()
    };
    let d = ln.sqrt().ceil() as usize;
    // println!("  d = {}", d);
    let mut image = empty_image(d + 1, d + 1);
    for i in 0..=d {
        image[i][0] = 1;
        image[0][i] = 1;
    }

    for cy in 0..d {
        for cx in 0..d {
            let bit = 1 << (cx + cy * d);
            // println!("    checking against {}", bit);
            if (value & bit) > 0 {
                image[1 + cx][1 + cy] = 1;
            }
        }
    }

    image
}

fn show_all_symbols() {
    let mut images = Vec::new();
    let mut keys = SYMBOLS.keys().collect::<Vec<&i32>>();
    keys.sort();

    let mut max_dx = 0;
    let offset = 2;
    let mut total_dy = offset;
    for key in keys.iter() {
        let image = encode_symbol(**key);
        max_dx = max_dx.max(image.len());
        total_dy += image[0].len() + 4;
        images.push(image);
    }

    let svg_width = 3 * (max_dx + 4) as usize;
    let svg_height = total_dy as usize;
    let mut svg = Svg::new(&"all_symbols.svg".to_string(), svg_width, svg_height);
    // initally set all image to black
    for x in 0..svg_width {
        for y in 0..svg_height {
            svg.set_pixel(x, y, value_to_svg_color(0));
        }
    }

    let mut y0 = offset;
    for index in 0..images.len() {
        let image = &images[index];
        for repeat in 0..3 {
            let x0 = offset + (max_dx + 4) * repeat;
            for dx in 0..image.len() {
                for dy in 0..image[0].len() {
                    if image[dx][dy] == 1 {
                        svg.set_pixel(x0 + dx, y0 + dy, value_to_svg_color(1));
                    }
                }
            }

            if repeat == 1 {
                let text = format!("{}", keys[index]);
                svg.add_raw_annotation(x0, y0, image.len(), image[0].len(), &text, true);
            }
            if repeat == 2 {
                svg.add_annotation(x0, y0, image.len(), image[0].len(), *keys[index], true);
            }
        }
        y0 += image[0].len() + 4;
    }

    svg.close();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Running {:?}", args);
    assert!(args.len() >= 2);
    if args[1] == "--show-all" {
        show_all_symbols();
        return;
    }

    assert!(args.len() >= 3);
    let input_file = args[1].to_string();
    let output_file = args[2].to_string();

    let img = image::open(input_file).unwrap().to_rgb();
    println!("Img dimensions: {:?}", img.dimensions());
    let scale = 4;
    let width = img.dimensions().0 / scale;
    let height = img.dimensions().1 / scale;

    let mut svg = Svg::new(&output_file, width as usize, height as usize);

    // initialize empty data structures
    let mut parsed = Vec::new();
    let mut image = empty_image(width as usize, height as usize);
    for _ in 0..width {
        parsed.push(vec![false; height as usize]);
    }

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(scale * x, scale * y);
            let value = rgb_to_value(pixel);
            image[x as usize][y as usize] = value;

            let color = value_to_svg_color(value);
            svg.set_pixel(x as usize, y as usize, color);
        }
    }

    let iw = ImageWrapper {
        image: image,
        height: height as usize,
        width: width as usize,
    };

    parse_image(&iw, &mut parsed, &mut svg);

    svg.close()
}
