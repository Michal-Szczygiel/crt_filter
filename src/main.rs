use image::{
    codecs::png::PngEncoder,
    imageops::{
        blur,
        colorops::{brighten_in_place, contrast_in_place},
        resize, FilterType,
    },
    ColorType, ImageBuffer, ImageEncoder, Rgb,
};
use clap::Parser;
use std::{error::Error, f64::consts::PI, fs::File, io::BufWriter, path::Path};

#[derive(Parser)]
#[clap(about)]
struct Configuration {
    #[clap(short, long)]
    image: String,

    #[clap(short, long)]
    directory: String,

    #[clap(short, long, default_value_t = 2)]
    upsampling: u32,

    #[clap(short, long)]
    pixel: u32,

    #[clap(short, long)]
    scanlines: usize,

    #[clap(short, long)]
    brightness: i32,

    #[clap(short, long)]
    contrast: f32,
}

fn apply_mask(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    pixel_size: u32,
    red_repr: Rgb<u8>,
    green_repr: Rgb<u8>,
    blue_repr: Rgb<u8>,
    amplification: u32,
) {
    let mut offset: u32;
    let one_third = pixel_size / 3;
    let two_thirds = 2 * pixel_size / 3;
    let gap_half = (0.05 * pixel_size as f64).round() as u32;

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        offset = if x % (2 * pixel_size) > pixel_size {
            pixel_size / 2
        } else {
            0
        };

        if x % pixel_size > gap_half
            && x % pixel_size < one_third - gap_half
            && (y + offset) % pixel_size > gap_half
            && (y + offset) % pixel_size < pixel_size - gap_half
        {
            *pixel = Rgb([
                if pixel[0] as u32 * red_repr[0] as u32 / 256 + amplification < 256 {
                    (pixel[0] as u32 * red_repr[0] as u32 / 256) as u8
                } else {
                    255
                },
                if pixel[0] as u32 * red_repr[1] as u32 / 256 + amplification < 256 {
                    (pixel[0] as u32 * red_repr[1] as u32 / 256) as u8
                } else {
                    255
                },
                if pixel[0] as u32 * red_repr[2] as u32 / 256 + amplification < 256 {
                    (pixel[0] as u32 * red_repr[2] as u32 / 256) as u8
                } else {
                    255
                },
            ]);
        } else if x % pixel_size > one_third + gap_half
            && x % pixel_size < two_thirds - gap_half
            && (y + offset) % pixel_size > gap_half
            && (y + offset) % pixel_size < pixel_size - gap_half
        {
            *pixel = Rgb([
                if pixel[1] as u32 * green_repr[0] as u32 / 256 + amplification < 256 {
                    (pixel[1] as u32 * green_repr[0] as u32 / 256) as u8
                } else {
                    255
                },
                if pixel[1] as u32 * green_repr[1] as u32 / 256 + amplification < 256 {
                    (pixel[1] as u32 * green_repr[1] as u32 / 256) as u8
                } else {
                    255
                },
                if pixel[1] as u32 * green_repr[2] as u32 / 256 + amplification < 256 {
                    (pixel[1] as u32 * green_repr[2] as u32 / 256) as u8
                } else {
                    255
                },
            ]);
        } else if x % pixel_size > two_thirds + gap_half
            && x % pixel_size < pixel_size - gap_half
            && (y + offset) % pixel_size > gap_half
            && (y + offset) % pixel_size < pixel_size - gap_half
        {
            *pixel = Rgb([
                if pixel[2] as u32 * blue_repr[0] as u32 / 256 + amplification < 256 {
                    (pixel[2] as u32 * blue_repr[0] as u32 / 256) as u8
                } else {
                    255
                },
                if pixel[2] as u32 * blue_repr[1] as u32 / 256 + amplification < 256 {
                    (pixel[2] as u32 * blue_repr[1] as u32 / 256) as u8
                } else {
                    255
                },
                if pixel[2] as u32 * blue_repr[2] as u32 / 256 + amplification < 256 {
                    (pixel[2] as u32 * blue_repr[2] as u32 / 256) as u8
                } else {
                    255
                },
            ]);
        } else {
            *pixel = Rgb([0, 0, 0]);
        }
    }
}

fn apply_scanlines(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, number: usize) {
    let (_, res_y) = image.dimensions();
    let density = number as f64 / res_y as f64;
    let mut factor: f64;

    for (_, y, pixel) in image.enumerate_pixels_mut() {
        factor = 0.3 * (PI * density * y as f64).sin().powi(2) + 0.7;

        *pixel = Rgb([
            (pixel[0] as f64 * factor) as u8,
            (pixel[1] as f64 * factor) as u8,
            (pixel[2] as f64 * factor) as u8,
        ])
    }
}

fn process_image(
    image_path: &str,
    output_directory: &str,
    upsampling: u32,
    pixel_size: u32,
    red_repr: Rgb<u8>,
    green_repr: Rgb<u8>,
    blue_repr: Rgb<u8>,
    scanlines: usize,
    brightness: i32,
    contrast: f32,
) -> Result<(), Box<dyn Error>> {
    let image_generic = image::open(Path::new(image_path))?;
    let image = image_generic.into_rgb8();
    let (res_x, res_y) = image.dimensions();

    let upsampled_image = resize(
        &image,
        res_x * upsampling,
        res_y * upsampling,
        FilterType::CatmullRom,
    );
    let mut upsampled_image_blurred = blur(&upsampled_image, 2.0 * upsampling as f32);

    apply_mask(
        &mut upsampled_image_blurred,
        pixel_size,
        red_repr,
        green_repr,
        blue_repr,
        40,
    );

    let mut image_with_mask = blur(&upsampled_image_blurred, 2.0 * upsampling as f32);
    apply_scanlines(&mut image_with_mask, scanlines);
    let mut processed_image = resize(&image_with_mask, res_x, res_y, FilterType::CatmullRom);
    brighten_in_place(&mut processed_image, brightness);
    contrast_in_place(&mut processed_image, contrast);

    let file_name = Path::new(&image_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let processed_image_file = File::create(Path::new(&format!(
        "{}/{}.png",
        output_directory, file_name
    )))?;

    let image_encoder = PngEncoder::new(BufWriter::new(processed_image_file));
    image_encoder.write_image(&processed_image, res_x, res_y, ColorType::Rgb8)?;

    return Ok(());
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Configuration::parse();
    process_image(
        &config.image,
        &config.directory,
        config.upsampling,
        config.pixel,
        Rgb([255, 0, 0]),
        Rgb([0, 255, 0]),
        Rgb([0, 0, 255]),
        config.scanlines,
        config.brightness,
        config.contrast,
    )?;

    return Ok(());
}
