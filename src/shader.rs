pub mod effects {
    use ab_glyph::{FontRef, PxScale};
    use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage, imageops};
    use imageproc::{self, drawing::draw_text};

    pub struct AsciiConfig {
        pub charset: String,
        pub invert: bool,
        pub scale: f32,
        pub font: FontRef<'static>
    }

    pub fn apply_ascii_luma(img: &image::GrayImage, config: &AsciiConfig) -> DynamicImage {
        let gradient: &[u8] = config.charset.as_bytes();
        let mut old_y = 0;
        let mut char_batch = String::new();
        let font_scale = PxScale { x: config.scale, y: config.scale};
        let font = &config.font;
        let nwidth: u32 =  ((img.width() as f32) / config.scale).floor().max(1.0) as u32;
        let nheight: u32 =  ((img.height() as f32) / config.scale).floor().max(1.0) as u32;

        // resize old img, create new one, and define ascii ramp
        let resized = imageops::resize(img, nwidth, nheight, imageops::FilterType::Nearest);
        let mut output_img = GrayImage::from_pixel(
            img.width(), 
            img.height(), 
            Luma([0])
        );

        // test text color
        let text_color = Luma([255]);

        // loop through pixels
        for (_, y, pixel) in resized.enumerate_pixels() {
            if y != old_y {
                output_img = draw_text(
                    &output_img, 
                    text_color, 
                    0, 
                    (old_y as f32 * config.scale) as i32, 
                    font_scale, 
                    font, 
                    &char_batch
                );

                char_batch.clear();
                old_y = y;
            }
            let [luminance] = pixel.0;
            let idx = (luminance as f32 / 255.0 * (gradient.len() as f32 - 1.0)) as usize;
            let ch = gradient[idx] as char;
        
            char_batch.push(ch);
        }

         // last line
        if !char_batch.is_empty() {
            output_img = draw_text(&output_img, 
                text_color,
                0,
                (old_y as f32 * config.scale) as i32, 
                font_scale,
                font, 
                &char_batch
            );
        }

        DynamicImage::ImageLuma8(output_img.clone())
    }

    pub fn apply_ascii_rgb(img: &image::RgbImage, config: &AsciiConfig) -> DynamicImage {
        let gradient: &[u8] = config.charset.as_bytes();
        let mut old_y = 0;
        let mut char_batch = String::new();
        let font_scale = PxScale { x: config.scale, y: config.scale};
        let font = &config.font;
        let nwidth: u32 =  ((img.width() as f32) / config.scale).floor().max(1.0) as u32;
        let nheight: u32 =  ((img.height() as f32) / config.scale).floor().max(1.0) as u32;

        // resize old img, create new one, and define ascii ramp
        let resized_rgb = imageops::resize(img, nwidth, nheight, imageops::FilterType::Nearest);
        let resized_gray = imageops::colorops::grayscale(&resized_rgb);
        
        let mut output_img = RgbImage::from_pixel(
            img.width(), 
            img.height(), 
            Rgb([0, 0 ,0])
        );

        // test text color
        let mut batch_color = Rgb([0u8, 0u8, 0u8]);
        let mut batch_x = 0i32;
        let flush_batch = |output: &RgbImage,
        batch: &mut String,
        color: Rgb<u8>,
        x: i32,
        y: u32| -> RgbImage {
            if !batch.is_empty() {
                let result = draw_text(
                    output,
                    color,
                    x,
                    (y as f32 * config.scale) as i32,
                    font_scale,
                    font,
                    batch
                );
                batch.clear();
                result
            } else {
                output.clone()
            }
        };

        // loop through pixels
        for (x, y, pixel) in resized_rgb.enumerate_pixels() {
            // flush current batch if new line
            if y != old_y {
                output_img = flush_batch(&output_img, &mut char_batch, batch_color, batch_x, old_y);
                old_y = y;
            } 

            let [r, g, b] = pixel.0;
            let luminance = resized_gray.get_pixel(x, y).0[0];
            let idx = (luminance as f32 / 255.0 * (gradient.len() - 1) as f32) as usize;
            let ch = gradient[idx] as char;
        
            let current_color = *pixel;
            let is_space = ch.is_whitespace();

            // skip spaces
            if is_space {
                if !char_batch.is_empty() {
                    output_img = flush_batch(&output_img, &mut char_batch, batch_color, batch_x, y);
                }
                continue;
            }

            // flush batch on color change
            if !char_batch.is_empty() && current_color != batch_color {
                output_img = flush_batch(&output_img, &mut char_batch, batch_color, batch_x, y);
            } 

            if char_batch.is_empty() {
                batch_color = current_color;
                batch_x = (x as f32 * config.scale) as i32;
            }
            char_batch.push(ch);
        }

         // last line
        if !char_batch.is_empty() {
            output_img = flush_batch(&output_img, &mut char_batch, batch_color, batch_x, old_y);
        }

        DynamicImage::ImageRgb8(output_img)
    }
}