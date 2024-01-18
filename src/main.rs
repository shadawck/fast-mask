use clap::{value_parser, Arg, Command};
use fast_mask::PatchMaskGenerator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs,
    path::{Path, PathBuf},
};

const SUPPORTED_IMAGE_EXTENSIONS: [&str; 16] = [
    "avif", "bmp", "dds", "farbfeld", "gif", "hdr", "ico", "jpeg", "jpg", "png", "pnm", "qoi",
    "tga", "tif", "tiff", "webp",
];

const DEFAULT_RATIO: f32 = 0.2;
const DEFAULT_PATCH_SIZE: u32 = 16;

fn is_supported_image_format(file_path: &Path) -> bool {
    if let Some(extension) = file_path.extension() {
        let extension_str = extension.to_str().unwrap_or("").to_lowercase();
        SUPPORTED_IMAGE_EXTENSIONS.contains(&extension_str.as_str())
    } else {
        false
    }
}
fn process_image(
    input_file: &str,
    input_folder: &str,
    output_folder: &str,
    ratio: f32,
    patch_size: u32,
    progress_bar: &ProgressBar,
) {
    // Check if the image file type is supported
    let input_path = Path::new(input_file);
    if !is_supported_image_format(input_path) {
        progress_bar.inc(1);
        return;
    }

    // Determine relative path from input folder
    let relative_path = input_path
        .strip_prefix(Path::new(input_folder))
        .unwrap_or_else(|_| input_path);

    // Create output subdirectories if they don't exist
    let output_path = Path::new(output_folder).join(relative_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create output subdirectories");
    }

    // Load an existing image
    let image = image::open(input_path).expect("Failed to load image");

    // Create PatchMaskGenerator with specified parameters
    let patchmg = PatchMaskGenerator::new(ratio, patch_size);

    // Transform the image using PatchMaskGenerator
    let masked_image = patchmg.transform(image);

    // Save the transformed image
    let output_file_path = output_path.with_extension("png");
    masked_image
        .save(output_file_path)
        .expect("Failed to save the transformed image");

    progress_bar.inc(1);
}

fn process_folder(
    input_folder: &str,
    output_folder: &str,
    ratio: f32,
    patch_size: u32,
    recursive: bool,
) {
    fs::create_dir_all(output_folder).expect("Failed to create output folder");

    // Process the input based on whether it's a file or a directory
    let input_path_buf = PathBuf::from(input_folder);

    if input_path_buf.is_file() {
        let progress_bar = ProgressBar::new(1);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}")
                .unwrap(),
        );

        let output_path = Path::new(output_folder).join(input_path_buf.file_name().unwrap());
        let output = output_path.to_str().unwrap();

        // Process a single image
        process_image(
            input_folder,
            input_folder,
            output,
            ratio,
            patch_size,
            &progress_bar,
        );
        progress_bar.finish();
    } else if input_path_buf.is_dir() {
        // Determine the number of file for the progess bar
        let file_count = if recursive {
            walkdir::WalkDir::new(input_folder)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .count()
        } else {
            fs::read_dir(input_folder)
                .expect("Failed to read input folder")
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().unwrap().is_file())
                .count()
        };

        // Setup progess bar
        let progress_bar = ProgressBar::new(file_count as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}")
                .unwrap(),
        );

        if recursive {
            walkdir::WalkDir::new(input_folder)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .par_bridge()
                .for_each(|entry| {
                    if entry.file_type().is_file() {
                        let input_path = entry.path();

                        process_image(
                            input_path.to_str().unwrap(),
                            input_folder,
                            output_folder,
                            ratio,
                            patch_size,
                            &progress_bar,
                        );
                    }
                });
        } else {
            fs::read_dir(input_folder)
                .expect("Failed to read input folder")
                .par_bridge()
                .for_each(|entry| {
                    if let Ok(entry) = entry {
                        if entry.file_type().unwrap().is_file() {
                            let input_path = entry.path();

                            process_image(
                                input_path.to_str().unwrap(),
                                input_folder,
                                output_folder,
                                ratio,
                                patch_size,
                                &progress_bar,
                            );
                        }
                    }
                });
        }
        progress_bar.finish();
    } else {
        eprintln!("Invalid input path provided.");
    }
}

fn main() {
    // Define command-line arguments using clap
    let matches = Command::new("Image Transformer")
        .version("1.0")
        .author("Shadawck")
        .about("Transforms images using PatchMaskGenerator")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FOLDER")
                .help("Sets the input folder containing images")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FOLDER")
                .help("Sets the output folder for transformed images")
                .required(true),
        )
        .arg(
            Arg::new("ratio")
                .short('r')
                .long("ratio")
                .value_name("FLOAT")
                .help("Sets the ratio for PatchMaskGenerator")
                .value_parser(value_parser!(f32)),
        )
        .arg(
            Arg::new("patch_size")
                .short('p')
                .long("patch-size")
                .value_name("INTEGER")
                .help("Sets the patch size for PatchMaskGenerator")
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("recursive")
                .short('n')
                .long("recursive")
                .help("Enable recursive handling of nested folders")
                .num_args(0),
        )
        .get_matches();

    let input_path = matches.get_one::<String>("input").unwrap();
    let output_folder = matches.get_one::<String>("output").unwrap();
    let recursive = *matches.get_one::<bool>("recursive").unwrap();

    let ratio: f32 = *matches.get_one::<f32>("ratio").unwrap_or(&DEFAULT_RATIO);
    let patch_size: u32 = *matches
        .get_one::<u32>("patch_size")
        .unwrap_or(&DEFAULT_PATCH_SIZE);

    process_folder(input_path, output_folder, ratio, patch_size, recursive);
}
