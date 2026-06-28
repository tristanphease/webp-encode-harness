use std::{
    fmt::Display,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Result as AHResult, anyhow};
use image::DynamicImage;
use image_webp::EncoderParams;

const REFERENCE_PATH: &'static str = "./reference-images";
const OUTPUT_PATH: &'static str = "./output-images";

fn main() {
    let result = run_analysis();

    match result {
        Ok(results) => {
            println!("Finished encoding");
            for result in results {
                println!("{}", result);
            }
        }
        Err(err) => {
            eprintln!("ERROR: {err}");
        }
    }
}

#[derive(Debug)]
struct OutputResult {
    file_name: String,
    libwebp: OutputMetrics,
    image: OutputMetrics,
}

impl Display for OutputResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Output results for {}:", self.file_name)?;
        writeln!(f, "libwebp: [{}]", self.libwebp)?;
        writeln!(f, "image: [{}]", self.image)?;
        Ok(())
    }
}

#[derive(Debug)]
struct OutputMetrics {
    // file size in bytes
    file_size: u64,
    // todo: other metrics could be some automatic metric of accuracy, although might not be great to do
}

impl Display for OutputMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bytes", self.file_size)
    }
}

fn run_analysis() -> AHResult<Vec<OutputResult>> {
    let file_names = load_image_file_names()?;
    println!("Loaded {} images for encoding", file_names.len());

    let mut results = Vec::new();

    create_output_folder()?;
    for file in file_names {
        let image = load_image(&file)?;
        let libwebp_metrics = encode_image_libwebp(&image, &file)?;
        let image_metrics = encode_image_image_webp(&image, &file)?;
        let output_result = OutputResult {
            file_name: file.clone(),
            libwebp: libwebp_metrics,
            image: image_metrics,
        };
        results.push(output_result);
    }
    Ok(results)
}

/// Creates the output folder in case it doesn't exist
fn create_output_folder() -> AHResult<()> {
    if !fs::exists(OUTPUT_PATH)? {
        fs::create_dir(OUTPUT_PATH)?;
    }
    Ok(())
}

/// Loads a png image from the /reference-images folder
fn load_image(path: &str) -> AHResult<DynamicImage> {
    let path_str = format!("{}/{path}", REFERENCE_PATH);
    let path = Path::new(&path_str);
    let loaded_image = image::open(path)?;
    Ok(loaded_image)
}

/// Loads the images from the folder
fn load_image_file_names() -> AHResult<Vec<String>> {
    let file_name_match =
        |name: &str| name.ends_with(".png") || name.ends_with(".webp") || name.ends_with(".jpg");
    let file_names = std::fs::read_dir(REFERENCE_PATH)?
        .filter_map(|file| file.ok())
        .filter(|file| file.file_type().is_ok_and(|f_type| f_type.is_file()))
        .filter(|file| file.file_name().to_str().is_some_and(file_name_match))
        .filter_map(|file| file.file_name().into_string().ok())
        .collect();
    Ok(file_names)
}

/// Creates the output file, overwriting any that are there
fn create_output_file(prefix: &str, file_name: &str) -> AHResult<File> {
    let mut path = PathBuf::from(format!("{}/{prefix}_{file_name}", OUTPUT_PATH));
    path.set_extension("webp");
    let file = File::create(path)?;
    Ok(file)
}

/// Encodes the image using libwebp
fn encode_image_libwebp(image: &DynamicImage, file_name: &str) -> AHResult<OutputMetrics> {
    let encoder = webp::Encoder::from_image(image)
        .map_err(|error_string| anyhow!("Error encoding using libwebp: {}", error_string))?;
    let webp_memory = encoder.encode(100.0); // TODO: consider the quality
    let file = create_output_file("libwebp", file_name)?;
    let mut buf_writer = BufWriter::new(file);
    buf_writer.write_all(&webp_memory)?;

    let metrics = OutputMetrics {
        file_size: webp_memory.len().try_into()?,
    };

    Ok(metrics)
}

/// Encodes the image using the image webp library
fn encode_image_image_webp(image: &DynamicImage, file_name: &str) -> AHResult<OutputMetrics> {
    let file = create_output_file("image-webp", file_name)?;
    let mut encoder = image_webp::WebPEncoder::new(&file);
    let mut encoder_params = EncoderParams::default();
    encoder_params.use_lossy = true;
    encoder_params.lossy_quality = 100;
    encoder.set_params(encoder_params);
    match image {
        DynamicImage::ImageLuma8(luma_bytes) => encoder.encode(
            &*luma_bytes,
            luma_bytes.width(),
            luma_bytes.height(),
            image_webp::ColorType::L8,
        )?,
        DynamicImage::ImageLumaA8(luma_bytes) => encoder.encode(
            &*luma_bytes,
            luma_bytes.width(),
            luma_bytes.height(),
            image_webp::ColorType::La8,
        )?,
        DynamicImage::ImageRgb8(rgb_bytes) => encoder.encode(
            &*rgb_bytes,
            rgb_bytes.width(),
            rgb_bytes.height(),
            image_webp::ColorType::Rgb8,
        )?,
        DynamicImage::ImageRgba8(rgba_bytes) => encoder.encode(
            &*rgba_bytes,
            rgba_bytes.width(),
            rgba_bytes.height(),
            image_webp::ColorType::Rgba8,
        )?,
        _ => return Err(anyhow!("Invalid colour type: {:?}", image.color())),
    }

    let file_size = file.metadata()?.len();
    let metrics = OutputMetrics { file_size };

    Ok(metrics)
}
