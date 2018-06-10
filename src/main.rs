extern crate clap;
extern crate memmap;

use clap::{App, Arg};
use memmap::{Mmap, MmapOptions};
use std::fs::{create_dir, metadata, File};
use std::io::Write;
use std::path::Path;

struct Jpeg {
    soi: usize,
    eoi: usize,
}

fn main() {
    let matches = App::new("jpeg-recover")
        .version("0.1.0")
        .author("Lisongmin")
        .about("recover jpeg from disk which filesystem metadata is broken.")
        .arg(
            Arg::with_name("input_file")
                .short("i")
                .long("input")
                .help("path to disk which contains the lost images.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output_dir")
                .short("o")
                .long("output")
                .help("directory where save the recovery files.")
                .default_value("/tmp/recover_jpeg")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("block_size")
                .short("s")
                .long("block-size")
                .help("specified block device size, in byte.")
                .default_value("0")
                .takes_value(true),
        )
        .get_matches();

    let input_file = matches.value_of("input_file").unwrap();
    let output_dir = matches.value_of("output_dir").unwrap();
    let block_size = matches
        .value_of("block_size")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let min_jpeg_size = 1024 * 500;
    let max_jpeg_size = 1024 * 1024 * 10;

    recover_jpeg(
        input_file,
        output_dir,
        block_size,
        min_jpeg_size,
        max_jpeg_size,
    );
}

fn recover_jpeg(
    input_file: &str,
    output_dir: &str,
    block_size: usize,
    min_jpeg_size: usize,
    max_jpeg_size: usize,
) {
    let file = match File::open(input_file) {
        Ok(file) => file,
        Err(e) => panic!("Can not open {}, detail: {}", input_file, e),
    };

    let fsize = if block_size > 0 {
        block_size
    } else {
        match metadata(input_file) {
            Ok(meta) => meta.len() as usize,
            Err(e) => panic!("Can not read metadata of {}, detail: {}", input_file, e),
        }
    };

    let mmap = unsafe {
        match MmapOptions::new().len(fsize).map(&file) {
            Err(e) => panic!("Can not mmap {}, detail: {}", input_file, e),
            Ok(file) => file,
        }
    };

    let out_path = Path::new(output_dir);
    if !out_path.exists() {
        match create_dir(out_path) {
            Err(e) => panic!("Can not create directory {}, detail: {}", output_dir, e),
            Ok(_) => (),
        };
    };

    let mut offset = 0usize;
    while let Some(jpeg) = find_jpeg(&mmap, offset, fsize, min_jpeg_size, max_jpeg_size) {
        let recover_path = format!("{}/recover-{}-{}.jpeg", output_dir, jpeg.soi, jpeg.eoi);
        println!("Match jpeg save to {}", recover_path);
        match File::create(&recover_path) {
            Ok(mut file) => file.write_all(&mmap[jpeg.soi..jpeg.eoi]).unwrap(),
            Err(e) => panic!(
                "Can not write recover file to {}, detail: {}",
                recover_path, e
            ),
        }

        offset = jpeg.eoi;
    }
}

// jpeg is format with:
//
// * begin with(SOI): "ff d8"
// * vender flag: byte[6:10] = "JFIF" or "Exif"
// * end with(EOI): "ff d9"
//
// **Note** that jpeg may embeded another jpeg, so "SOI SOI EOI EOI" is valid.
//
// wo can only recover sequence jpeg datas, so we should stop match if size is large than
// max_jpeg_size.
fn find_jpeg(
    mmap: &Mmap,
    offset: usize,
    fsize: usize,
    min_jpeg_size: usize,
    max_jpeg_size: usize,
) -> Option<Jpeg> {
    if offset >= fsize {
        return None;
    }

    let type_flag_pos = 10;
    let soi_flag: [u8; 2] = [0xff, 0xd8];
    let eoi_flag: [u8; 2] = [0xff, 0xd9];

    let mut soi = offset;
    'refind: loop {
        let mut matched = false;
        for x in soi..fsize - type_flag_pos {
            if mmap[x..x + 2] == soi_flag {
                if &mmap[x + 6..x + 10] == b"Exif" || &mmap[x + 6..x + 10] == b"JFIF" {
                    soi = x;
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            return None;
        }

        let mut embeded_soi = 0;
        for x in soi..fsize - 1 {
            if &mmap[x..x + 2] == eoi_flag {
                embeded_soi -= 1;
                if embeded_soi <= 0 {
                    if x - soi < min_jpeg_size {
                        soi += type_flag_pos;
                        continue 'refind;
                    }
                    return Some(Jpeg {
                        soi: soi,
                        eoi: x + 2,
                    });
                }
            } else if &mmap[x..x + 2] == soi_flag {
                embeded_soi += 1;
            }
            if x - soi > max_jpeg_size {
                soi += type_flag_pos;
                continue 'refind;
            }
        }
        break;
    }
    None
}
