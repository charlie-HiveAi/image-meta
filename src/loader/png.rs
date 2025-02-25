use std::io::{BufRead, Cursor, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};

use crate::errors::{ImageError, ImageResult};
use crate::types::{Color, Dimensions, Format, ImageMeta};

const SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

pub fn load<R: ?Sized + BufRead + Seek>(image: &mut R) -> ImageResult<ImageMeta> {
    read_signature(image)?;

    let (dimensions, color) = read_header(image)?;
    // let animation_frames = read_fctls(image)?;

    Ok(ImageMeta {
        animation_frames: None,
        color,
        dimensions,
        format: Format::Png,
    })
}

fn read_signature<R: ?Sized + BufRead + Seek>(image: &mut R) -> ImageResult {
    let mut signature = [0u8; 8];
    image.read_exact(&mut signature)?;
    if SIGNATURE != signature {
        return Err(ImageError::InvalidSignature);
    }
    Ok(())
}

fn read_header<R: ?Sized + BufRead + Seek>(image: &mut R) -> ImageResult<(Dimensions, Color)> {
    use crate::types::ColorMode::*;

    let (chunk_name, chunk_data) = read_chunk(image)?;
    if chunk_name != *b"IHDR" {
        return Err(ImageError::CorruptImage("Not IHDR".into()));
    }
    let mut chunk_data = Cursor::new(chunk_data);

    let width = chunk_data.read_u32::<BigEndian>()?;
    let height = chunk_data.read_u32::<BigEndian>()?;
    let resolution = chunk_data.read_u8()?;
    let color = chunk_data.read_u8()?;
    let (mode, alpha_channel) = match color {
        0 => (Grayscale, false),
        2 => (Rgb, false),
        3 => (Indexed, false),
        4 => (Grayscale, true),
        6 => (Rgb, true),
        _ => {
            return Err(ImageError::CorruptImage(
                format!("Invalid color type: {}", color).into(),
            ))
        }
    };
    let color = Color {
        mode,
        alpha_channel,
        resolution,
    };

    // 1 compression_method
    // 1 filter_method
    // 1 interlace_method

    Ok((Dimensions { height, width }, color))
}

fn read_chunk<R: ?Sized + BufRead + Seek>(image: &mut R) -> ImageResult<([u8; 4], Vec<u8>)> {
    let length = image.read_u32::<BigEndian>()?;
    let mut chunk_name = [0u8; 4];
    image.read_exact(&mut chunk_name)?;
    let mut result = vec![0u8; length as usize];
    image.read_exact(&mut result)?;
    // Skip CRC
    image.seek(SeekFrom::Current(4))?;
    Ok((chunk_name, result))
}

// fn read_fctls<R: ?Sized + BufRead + Seek>(image: &mut R) -> ImageResult<Option<usize>> {
//     let mut result = 0;
//     let mut chunk_name = [0u8; 4];
//     loop {
//         let length = image.read_u32::<BigEndian>()?;
//         image.read_exact(&mut chunk_name)?;
//         if chunk_name == *b"fcTL" {
//             result += 1;
//         }
//         image.seek(SeekFrom::Current(i64::from(length) + 4))?; // 4 means CRC
//         if chunk_name == *b"IEND" {
//             break;
//         }
//     }
//     if 0 < result {
//         Ok(Some(result))
//     } else {
//         Ok(None)
//     }
// }
