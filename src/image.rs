use nannou::image::{ImageDecoder, dxt::DXTVariant};

use crate::types::d3d::{D3DFormat, LinearColour, StandardFormat, Swizzled};

pub fn transcode(
    width: usize,
    height: usize,
    src_format: D3DFormat,
    dst_format: D3DFormat,
    bytes: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    if src_format == dst_format {
        return Ok(bytes.to_vec().to_owned());
    }

    match src_format {
        D3DFormat::Standard(StandardFormat::DXT1) => match dst_format {
            D3DFormat::Linear(LinearColour::R8G8B8A8) => {
                let decoder = nannou::image::codecs::dxt::DxtDecoder::new(
                    bytes,
                    width as u32,
                    height as u32,
                    DXTVariant::DXT1,
                )
                .expect("");

                let mut buf = vec![0u8; width * height * 3];
                decoder
                    .read_image(&mut buf)
                    .map_err(std::io::Error::other)?;

                let mut bytes = vec![];

                for chunk in buf.chunks(3) {
                    bytes.extend_from_slice(chunk);
                    bytes.push(u8::MAX);
                }

                Ok(bytes)
            }
            _ => Err(std::io::Error::other(
                "Unsupported destination format for transcoding.",
            )),
        },

        D3DFormat::Standard(StandardFormat::DXT2Or3) => match dst_format {
            D3DFormat::Linear(LinearColour::R8G8B8A8) => {
                let decoder = nannou::image::codecs::dxt::DxtDecoder::new(
                    bytes,
                    width as u32,
                    height as u32,
                    DXTVariant::DXT3,
                )
                .expect("");

                let mut buf = vec![0u8; width * height * 4];
                decoder
                    .read_image(&mut buf)
                    .map_err(std::io::Error::other)?;

                Ok(buf)
            }
            _ => Err(std::io::Error::other(
                "Unsupported destination format for transcoding.",
            )),
        },

        D3DFormat::Swizzled(Swizzled::A8B8G8R8) => match dst_format {
            D3DFormat::Linear(LinearColour::R8G8B8A8) => {
                let mut ret_bytes = bytes.to_vec();

                ret_bytes.chunks_mut(4).for_each(|chunk| {
                    chunk.reverse();
                });

                Ok(ret_bytes)
            }
            _ => Err(std::io::Error::other(
                "Unsupported destination format for transcoding.",
            )),
        },

        D3DFormat::Swizzled(Swizzled::B8G8R8A8) => match dst_format {
            D3DFormat::Linear(LinearColour::R8G8B8A8) => {
                let mut ret_bytes = bytes.to_vec();

                ret_bytes.chunks_mut(4).for_each(|chunk| {
                    let b = chunk[0];
                    let r = chunk[2];

                    chunk[0] = r;
                    chunk[2] = b;
                });

                Ok(ret_bytes)
            }
            _ => Err(std::io::Error::other(
                "Unsupported destination format for transcoding.",
            )),
        },

        D3DFormat::Swizzled(Swizzled::A8R8G8B8) => match dst_format {
            D3DFormat::Linear(LinearColour::R8G8B8A8) => {
                let mut ret_bytes = bytes.to_vec();

                ret_bytes.chunks_mut(4).for_each(|chunk| {
                    chunk.rotate_left(1);
                });

                Ok(ret_bytes)
            }
            _ => Err(std::io::Error::other(
                "Unsupported destination format for transcoding.",
            )),
        },
        _ => Err(std::io::Error::other(
            "Unsupported source format for transcoding.",
        )),
    }
}
