use image::{ImageReader,Rgb, RgbImage, imageops};
use std::{io::{Write, stdin, stdout}, path::PathBuf};
use imageproc::{self, drawing::draw_text};
use ab_glyph::{FontRef, PxScale};

mod source;
mod shader;

use source::input;
use shader::effects;
use crate::source::input::ColorMode;

fn main() {
    // get file path
    let mut file_path_name: String = String::new();
    print!("Enter the file path: ");
    let _=stdout().flush();
    stdin().read_line(&mut file_path_name).expect("File string invalid");

    let path = PathBuf::from(&file_path_name.trim());

    if let Err(e) = input::validify_file_path(&path) {
        eprintln!("Error with file path: {}", e);
        return;
    }

    // get color mode
    let mut color: String = String::new();
    print!("Colored (may take significantly longer) y/N: ");
    let _=stdout().flush();
    stdin().read_line(&mut color).expect("Invalid string, defaulting to non-colored");

    let color_mode: ColorMode = match color.trim().to_lowercase().as_str() {
        "y" | "yes" => ColorMode::Rgb,
        _ => ColorMode::Luma
    };

    let img_type = input::open_image(path, color_mode)
    .unwrap_or_else(|e| {
        eprintln!("Failed to open image: {}", e);
        std::process::exit(1);
    });

    // get down scale level
    let mut ds_detail_level = String::new();
    print!("Enter down scale level (8 = 1/8 of original reosolution): ");
    let _=stdout().flush();
    stdin().read_line(&mut ds_detail_level).expect("Invalid string");

    let down_scale: f32 = match ds_detail_level.trim().parse::<f32>() {
        Ok(value) => value,
        Err(e)=>{
            println!("Not a valid numerical down scale level. {}\nDefaulting to 8", e);
            8.0
        }
    };

    // ascii config
    let ascii_config =  effects::AsciiConfig {
        charset: String::from(" .-/+=[&$%#@"),
        invert: false,
        scale: down_scale,
        font: FontRef::try_from_slice(include_bytes!("../fonts/Bescii-Mono.ttf"))
            .expect("failed to load font"),
    };

    let output_img = match img_type {
        input::LoadedImageType::Rgb(img) => effects::apply_ascii_rgb(&img, &ascii_config),
        input::LoadedImageType::Luma(img) => effects::apply_ascii_luma(&img, &ascii_config)
    };

    output_img.save("images/test.png").expect("Failed to save image.");
    print!("Finished");
}