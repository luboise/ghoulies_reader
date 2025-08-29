mod constants;

mod types;

pub mod image_data;

use std::{
    env,
    ffi::OsStr,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use crate::types::{
    BNLFile,
    asset::{Asset, texture::Texture},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        print_usage();
        return;
    }

    let bnl_path = PathBuf::from(&args[1]);

    println!("Opening BNL file {}", bnl_path.display());

    let data: Vec<u8> = match std::fs::read(&bnl_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Unable to open file {}. Error: {}", bnl_path.display(), e);
            return;
        }
    };

    let decompressed: Vec<u8> = match miniz_oxide::inflate::decompress_to_vec_zlib(&data[40..]) {
        Ok(d) => {
            let mut res = data[0..40].to_vec();
            res.extend_from_slice(&d);
            res
        }
        Err(e) => {
            println!("Unable to decompress: {}", e);
            return;
        }
    };

    let bnl = match BNLFile::from_cursor(&mut Cursor::new(decompressed)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Unable to process BNL file: {}", e);
            std::process::exit(1);
        }
    };

    let raw_assets = bnl.get_raw_assets();

    let out_filename = format!(
        "{}_bnl",
        bnl_path
            .file_stem()
            .unwrap_or(OsStr::new("unknown"))
            .display()
    );

    // ./out/common_bnl
    let bnl_out_path = Path::new("./out").join(out_filename);

    raw_assets.iter().for_each(|raw_asset| {
        // ./out/common_bnl/aid_texture_xyz
        let asset_path: PathBuf = bnl_out_path.join(&raw_asset.name);

        if asset_path.is_file() {
            eprintln!(
                "Unable to write to {} (A file already exists by that name)",
                asset_path.display()
            );
            return;
        } else if !asset_path.exists() {
            match fs::create_dir_all(&asset_path) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!(
                        "Unable to create directory {}.\nError: {}",
                        asset_path.display(),
                        e
                    );
                    return;
                }
            }
        }

        std::fs::write(asset_path.join("descriptor"), &raw_asset.descriptor_bytes).unwrap_or_else(
            |e| {
                eprintln!(
                    "Unable to write descriptor for {}\nError: {}",
                    &raw_asset.name, e
                );
            },
        );

        raw_asset
            .data_slices
            .iter()
            .enumerate()
            .for_each(|(i, slice)| {
                std::fs::write(asset_path.join(format!("resource{}", i)), slice).unwrap_or_else(
                    |e| {
                        eprintln!(
                            "Unable to write descriptor for {}\nError: {}",
                            &raw_asset.name, e
                        );
                    },
                );
            });
    });
}

fn print_usage() {
    println!(
        r"Usage: ghoulies_reader [path to BNL file].
Example:
    ghoulies_reader ./common.bnl
    ghoulies_reader ./gbtg/bundles/common.bnl"
    );
}
