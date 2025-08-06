mod constants;
mod types;

use std::{env, io::Cursor};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Bad arg count (expected 2).");
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

    let bnl = types::BNLFile::from_cursor(&mut Cursor::new(decompressed));

    dbg!(bnl);

    /*
    with open("bundles/aid_script/ghoulies_chapter1_scene1_2playcam.bnl", "rb") as f:
        data = f.read()

    data = data[:40] + zlib.decompress(data[40:])

    data[:100]

    with open("bundles/playcampy", "wb") as f:
        f.write(data)

    */
}
