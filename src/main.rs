mod constants;
mod types;

use std::{env, io::Cursor, path::Path};

use crate::types::{BNLFile, asset::texture::Texture};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        print_usage();
        return;
    }

    println!("Opening {}", &args[1]);

    let data: Vec<u8> = match std::fs::read(&args[1]) {
        Ok(f) => f,
        Err(e) => {
            println!("Unable to open file {}. Error: {}", &args[1], e);
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

    let tex = bnl
        .get_asset::<Texture>("aid_texture_ghoulies_gameselect_challenges_bedtimegory")
        .expect("Unable to get texture");

    dbg!(&tex);

    /*
    let out_path = Path::new("./processed").join(Path::new(&args[1]).file_stem().unwrap());

    if !out_path.exists() {
        match std::fs::create_dir_all(&out_path) {
            Err(e) => {
                eprintln!("Unable to create dir {}", &out_path.to_str().unwrap());
                std::process::exit(1);
            }
            _ => (),
        };
    } else if out_path.is_file() {
        eprintln!(
            "Unable to extract to {} as a file already exists at that location.",
            out_path.to_str().unwrap()
        );
        std::process::exit(1);
    }
    */

    /*
    match bnl.dump(&out_path) {
        Ok(_) => (),
        Err(e) => eprintln!("Unable to dump BNL file: {}", e),
    };
    */
}

fn print_usage() {
    println!(
        r"Usage: ghoulies_reader [path to BNL file].
Example:
    ghoulies_reader ./common.bnl
    ghoulies_reader ./gbtg/bundles/common.bnl"
    );
}
