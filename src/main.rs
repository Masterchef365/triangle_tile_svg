use anyhow::{bail, Context, Result};
use std::io::Read;
use std::path::Path;

fn main() -> Result<()> {
    // Arg parsing
    let mut args = std::env::args();
    let program_name = args.next().unwrap();
    let usage = || {
        format!(
            "Usage: {} <image path> <# vertical triangles (30)> <triangle height (0.1)>",
            program_name
        )
    };
    let image_path = args.next().with_context(usage)?;
    //let image_path = args.next().with_context(usage);

    let (width, rgb_data) = load_png_from_path(image_path).context("Loading image")?;
    let height = rgb_data.len() / (width * 3);

    dbg!(width, height);

    Ok(())
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