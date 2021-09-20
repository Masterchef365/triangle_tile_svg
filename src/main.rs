use anyhow::{bail, Context, Result};
use std::io::Read;
use std::path::{Path, PathBuf};
use svg::node::element::{path::Data as SvgData, Path as SvgPath};
use svg::Node;

fn main() -> Result<()> {
    // Arg parsing
    let mut args = std::env::args();
    let program_name = args.next().unwrap();
    let usage = || {
        format!(
            "Usage: {} <image path> <# vertical triangles (30)> <triangle height (0.1)> <out path>",
            program_name
        )
    };

    let image_path: PathBuf = args.next().with_context(usage)?.into();

    let n_vertical_tris: usize = args
        .next()
        .unwrap_or("30".to_string())
        .parse()
        .context("# of vertical triangles")?;

    let triangle_height: f32 = args
        .next()
        .unwrap_or("0.1".to_string())
        .parse()
        .context("Triangle height")?;

    let svg_path = args
        .next()
        .unwrap_or("out.svg".to_string());

    // Load image
    let (image_width, image_data) = load_png_from_path(image_path).context("Loading image")?;
    let image_height = image_data.len() / (image_width * 3);
    
    if image_data.is_empty() {
        bail!("Empty image");
    }

    // Ratio of half the base of a triangle to it's height
    let sqrt_3 = (3.0_f32).sqrt();

    // Number of triangles horizontally
    let n_horiz_tris = (image_width * n_vertical_tris) / image_height;
    let n_horiz_tris = (n_horiz_tris as f32 * sqrt_3) as usize;

    // Half of the width of the base of a triangle. Useful for stepping along the grid
    let half_triangle_width = triangle_height / sqrt_3;

    // Generate triangles
    let mut document = svg::Document::new().set(
        "viewBox",
        (
            0,
            0,
            n_horiz_tris as f32 * half_triangle_width,
            n_vertical_tris as f32 * triangle_height,
        ),
    );

    let mut y = 0.0;
    for row in 0..n_vertical_tris {
        let mut x = 0.0;
        for col in 0..=n_horiz_tris {
            let img_y = ((row * image_height) / n_vertical_tris).min(image_height-1);
            let img_x = ((col * image_width) / n_horiz_tris).min(image_width-1);
            let img_idx = img_x + img_y * image_width;
            let subpixel_idx = img_idx*3;

            let rgb = [
                image_data[subpixel_idx+0],
                image_data[subpixel_idx+1],
                image_data[subpixel_idx+2],
            ];

            let points_up = (row & 1 == 0) != (col & 1 == 0);

            let color = encode_color(rgb);

            document.append(triangle_at(x, y, half_triangle_width, triangle_height, points_up, &color));
            
            x += half_triangle_width;
        }
        y += triangle_height;
    }

    svg::save(svg_path, &document).context("Saving document")?;

    Ok(())
}

fn triangle_at(x: f32, y: f32, half_width: f32, height: f32, points_up: bool, color: &str) -> SvgPath {
    let data = if points_up {
        SvgData::new()
            .move_to((x, y))
            .line_by((-half_width, height))
            .line_by((half_width * 2., 0.))
    } else {
        SvgData::new()
            .move_to((x, y + height))
            .line_by((-half_width, -height))
            .line_by((half_width * 2., 0.))
    }
    .close();

    SvgPath::new()
        .set("fill", color)
        .set("stroke", "none")
        .set("stroke-width", 0.001)
        .set("d", data)
}

fn encode_color([r, g, b]: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn load_png_from_path<P: AsRef<Path>>(path: P) -> Result<(usize, Vec<u8>)> {
    let file = std::fs::File::open(path).context("Opening file")?;
    let reader = std::io::BufReader::new(file);
    load_png_rgb(reader)
}

/// Returns (width, rgb data) for the given PNG image reader
fn load_png_rgb<R: Read>(r: R) -> Result<(usize, Vec<u8>)> {
    let decoder = png::Decoder::new(r);
    let mut reader = decoder.read_info().context("Creating reader")?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).context("Reading frame")?;

    if info.bit_depth != png::BitDepth::Eight {
        bail!("Bit depth {:?} unsupported!", info.bit_depth);
    }

    buf.truncate(info.buffer_size());

    let buf: Vec<u8> = match info.color_type {
        png::ColorType::Rgb => buf,
        png::ColorType::Rgba => buf
            .chunks_exact(4)
            .map(|px| [px[0], px[1], px[2]])
            .flatten()
            .collect(),
        png::ColorType::Grayscale => buf.iter().map(|&px| [px; 3]).flatten().collect(),
        png::ColorType::GrayscaleAlpha => {
            buf.chunks_exact(2).map(|px| [px[0]; 3]).flatten().collect()
        }
        other => bail!("Images with color type {:?} are unsupported", other),
    };

    Ok((info.width as usize, buf))
}