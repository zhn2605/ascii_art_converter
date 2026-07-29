pub mod effects {
    use ab_glyph::{FontRef, PxScale};
    use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage, imageops};
    use imageproc::{self, drawing::draw_text};
    use std::time::Instant;

    pub struct AsciiConfig {
        pub charset: String,
        pub invert: bool,
        pub scale: f32,
        pub font: FontRef<'static>
    }

    fn calculate_dimensions(width: u32, height: u32, scale: f32) -> (u32, u32) {
        let nwidth = ((width as f32) / scale).floor().max(1.0) as u32;
        let nheight = ((height as f32) / scale).floor().max(1.0) as u32;
        (nwidth, nheight)
    }

    // lookup table to map luminance to ascii chars
    fn build_luminance_lookup_table(gradient: &[u8]) -> [char; 256] {
        let mut table = [' '; 256];
        let gradient_len = gradient.len() as f32;
        for lum in 0..256 {
            let idx = ((lum as f32 / 255.0) * (gradient_len - 1.0)) as usize;
            table[lum] = gradient[idx] as char;
        }
        table
    }

    pub fn apply_ascii_luma(img: &image::GrayImage, config: &AsciiConfig) -> DynamicImage {
        let gradient: &[u8] = config.charset.as_bytes();
        
        // pre-compute lookup table
        let lum_table = build_luminance_lookup_table(gradient);
        
        let font_scale = PxScale { x: config.scale, y: config.scale};
        let font = &config.font;
        let (nwidth, nheight) = calculate_dimensions(img.width(), img.height(), config.scale);

        // resize old img, create new one
        let resized = imageops::resize(img, nwidth, nheight, imageops::FilterType::Nearest);
        let mut output_img = GrayImage::from_pixel(
            img.width(), 
            img.height(), 
            Luma([0])
        );

        let text_color = Luma([255]);    
        let mut char_batch = String::with_capacity(nwidth as usize);

        // process row by row
        let start = Instant::now();
        for y in 0..nheight {
            char_batch.clear();
            
            for x in 0..nwidth {
                let pixel = resized.get_pixel(x, y);
                let luminance = pixel.0[0];
                char_batch.push(lum_table[luminance as usize]);
            }
            
            output_img = draw_text(
                &output_img, 
                text_color, 
                0, 
                (y as f32 * config.scale) as i32, 
                font_scale, 
                font, 
                &char_batch
            );
        }

        println!("ASCII conversion took: {:?}", start.elapsed());
        DynamicImage::ImageLuma8(output_img)
    }

    pub fn apply_ascii_rgb(img: &image::RgbImage, config: &AsciiConfig) -> DynamicImage {
        let gradient: &[u8] = config.charset.as_bytes();
        
        // pre-compute lookup table
        let lum_table = build_luminance_lookup_table(gradient);
        
        let font_scale = PxScale { x: config.scale, y: config.scale};
        let font = &config.font;
        let (nwidth, nheight) = calculate_dimensions(img.width(), img.height(), config.scale);

        // resize images once
        let resized_rgb = imageops::resize(img, nwidth, nheight, imageops::FilterType::Nearest);
        let resized_gray = imageops::colorops::grayscale(&resized_rgb);
        
        let mut output_img = RgbImage::from_pixel(
            img.width(), 
            img.height(), 
            Rgb([0, 0, 0])
        );

        let mut char_batch = String::with_capacity(nwidth as usize);
        
        let draw_text_mut = |output: &mut RgbImage,
                             batch: &str,
                             color: Rgb<u8>,
                             x: i32,
                             y: i32| {
            if !batch.is_empty() {
                *output = draw_text(output, color, x, y, font_scale, font, batch);
            }
        };
        
        // process row by row
        let start = Instant::now();
        for y in 0..nheight {
            char_batch.clear();
            let mut batch_start_x = 0i32;
            let mut batch_color = Rgb([0u8, 0u8, 0u8]);
            let mut batch_empty = true;
            
            for x in 0..nwidth {
                let pixel = resized_rgb.get_pixel(x, y);
                let luminance = resized_gray.get_pixel(x, y).0[0];
                let ch = lum_table[luminance as usize];
                let current_color = *pixel;
                
                // skip spaces
                if ch.is_whitespace() {
                    if !batch_empty {
                        draw_text_mut(
                            &mut output_img,
                            &char_batch,
                            batch_color,
                            batch_start_x,
                            (y as f32 * config.scale) as i32
                        );
                        char_batch.clear();
                        batch_empty = true;
                    }
                    continue;
                }

                // flush batch on color change
                if !batch_empty && current_color != batch_color {
                    draw_text_mut(
                        &mut output_img,
                        &char_batch,
                        batch_color,
                        batch_start_x,
                        (y as f32 * config.scale) as i32
                    );
                    char_batch.clear();
                    batch_empty = true;
                }

                // start new batch
                if batch_empty {
                    batch_color = current_color;
                    batch_start_x = (x as f32 * config.scale) as i32;
                    batch_empty = false;
                }
                
                char_batch.push(ch);
            }
            
            // flush remaining batch for this row
            if !batch_empty {
                draw_text_mut(
                    &mut output_img,
                    &char_batch,
                    batch_color,
                    batch_start_x,
                    (y as f32 * config.scale) as i32
                );
            }
        }

        println!("ASCII conversion took: {:?}", start.elapsed());
        DynamicImage::ImageRgb8(output_img)
    }

}
