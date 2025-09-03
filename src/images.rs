use crate::d3d::{D3DFormat, LinearColour, StandardFormat, Swizzled};

use texpresso::{Algorithm::RangeFit, Format::Bc1, Format::Bc2};

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
                let buf = bcndecode::decode(
                    bytes,
                    width,
                    height,
                    bcndecode::BcnEncoding::Bc1, // BC1 = DXT1
                    bcndecode::BcnDecoderFormat::RGBA,
                )
                .map_err(std::io::Error::other)?;

                Ok(buf)
            }
            _ => Err(std::io::Error::other(
                "Unsupported destination format for transcoding.",
            )),
        },

        D3DFormat::Standard(StandardFormat::DXT2Or3) => match dst_format {
            D3DFormat::Linear(LinearColour::R8G8B8A8) => {
                let buf = bcndecode::decode(
                    bytes,
                    width,
                    height,
                    bcndecode::BcnEncoding::Bc2, // BC2 = DXT2, BC3 and DXT3 treated the same
                    bcndecode::BcnDecoderFormat::RGBA,
                )
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

        D3DFormat::Swizzled(Swizzled::R8G8B8A8) => match dst_format {
            D3DFormat::Standard(StandardFormat::DXT1) => {
                let mut data_copy = vec![0x00; bytes.len()];

                for (i, chunk) in bytes.chunks_exact(4).enumerate() {
                    let j = 4 * i;

                    data_copy[j] = chunk[2];
                    data_copy[j + 1] = chunk[1];
                    data_copy[j + 2] = chunk[0];
                    data_copy[j + 3] = chunk[3];
                }

                let mut converted_bytes = vec![0x00; Bc1.compressed_size(width, height)];

                Bc1.compress(
                    &data_copy,
                    width,
                    height,
                    texpresso::Params {
                        ..Default::default()
                    },
                    &mut converted_bytes,
                );

                Ok(converted_bytes)
            }

            D3DFormat::Standard(StandardFormat::DXT2Or3) => {
                let mut data_copy = vec![0x00; bytes.len()];

                for (i, chunk) in bytes.chunks_exact(4).enumerate() {
                    let j = 4 * i;

                    data_copy[j] = chunk[2];
                    data_copy[j + 1] = chunk[1];
                    data_copy[j + 2] = chunk[0];
                    data_copy[j + 3] = chunk[3];
                }

                let mut converted_bytes = vec![0x00; Bc2.compressed_size(width, height)];

                Bc2.compress(
                    &data_copy,
                    width,
                    height,
                    texpresso::Params {
                        ..Default::default()
                    },
                    &mut converted_bytes,
                );

                Ok(converted_bytes)
            }

            D3DFormat::Swizzled(Swizzled::B8G8R8A8) => {
                let mut data_copy = vec![0x00; bytes.len()];

                for (i, chunk) in bytes.chunks_exact(4).enumerate() {
                    let j = 4 * i;

                    data_copy[j] = chunk[2];
                    data_copy[j + 1] = chunk[1];
                    data_copy[j + 2] = chunk[0];
                    data_copy[j + 3] = chunk[3];
                }

                Ok(data_copy)
            }

            _ => Err(std::io::Error::other(
                "Unsupported source format for transcoding.",
            )),
        },

        D3DFormat::Swizzled(Swizzled::B8G8R8A8) => match dst_format {
            D3DFormat::Standard(StandardFormat::DXT1) => {
                let mut converted_bytes = vec![0x00; Bc1.compressed_size(width, height)];

                Bc1.compress(
                    bytes,
                    width,
                    height,
                    texpresso::Params {
                        ..Default::default()
                    },
                    &mut converted_bytes,
                );

                Ok(converted_bytes)
            }
            D3DFormat::Standard(StandardFormat::DXT2Or3) => {
                let mut converted_bytes = vec![0x00; Bc2.compressed_size(width, height)];

                Bc2.compress(
                    bytes,
                    width,
                    height,
                    texpresso::Params {
                        ..Default::default()
                    },
                    &mut converted_bytes,
                );

                Ok(converted_bytes)
            }

            D3DFormat::Swizzled(Swizzled::R8G8B8A8) => {
                let mut data_copy = vec![0x00; bytes.len()];

                for (i, chunk) in bytes.chunks_exact(4).enumerate() {
                    let j = 4 * i;

                    data_copy[j] = chunk[2];
                    data_copy[j + 1] = chunk[1];
                    data_copy[j + 2] = chunk[0];
                    data_copy[j + 3] = chunk[3];
                }

                Ok(data_copy)
            }

            _ => Err(std::io::Error::other(
                "Unsupported source format for transcoding.",
            )),
        },

        _ => Err(std::io::Error::other(
            "Unsupported source format for transcoding.",
        )),
    }
}
