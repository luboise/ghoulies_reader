mod constants;
mod types;

use std::{env, io::Cursor, path::Path};

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

    let bnl = match types::BNLFile::from_cursor(&mut Cursor::new(decompressed)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Unable to process BNL file: {}", e);
            std::process::exit(1);
        }
    };

    let path = Path::new("./processed");

    match bnl.dump(path) {
        Ok(_) => (),
        Err(e) => eprintln!("Unable to dump BNL file: {}", e),
    };
}

fn print_usage() {
    println!(
        r"Usage: ghoulies_reader [path to BNL file].
Example:
    ghoulies_reader ./common.bnl
    ghoulies_reader ./gbtg/bundles/common.bnl"
    );
}
