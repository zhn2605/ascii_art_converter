pub mod input {
    // kinda useless for now as its just one form of input. 
    use std::{io::{self, Error, ErrorKind,}, path::PathBuf};
    use image::{ImageBuffer, ImageError, ImageReader, Luma, Rgb};

    pub enum LoadedImageType {
        Luma(ImageBuffer<Luma<u8>, Vec<u8>>),
        Rgb(ImageBuffer<Rgb<u8>, Vec<u8>>),
    }

    pub enum ColorMode {
        Luma,
        Rgb,
    }

    pub fn validify_file_path(path: &PathBuf) -> io::Result<()> {
        if !path.exists() {
            Err(Error::new(ErrorKind::NotFound, "File does not exist"))
        } else if !path.is_file() {
            Err(Error::new(ErrorKind::InvalidInput, "Not a file"))
        } else {
            Ok(())
        }
    }

    pub fn open_image(path: PathBuf, mode: ColorMode ) -> Result<LoadedImageType, ImageError> {
        let img = ImageReader::open(&path)?.decode()?;

        Ok(match mode {
            ColorMode::Rgb => LoadedImageType::Rgb(img.to_rgb8()),
            ColorMode::Luma => LoadedImageType::Luma(img.to_luma8())
        })
    }
}