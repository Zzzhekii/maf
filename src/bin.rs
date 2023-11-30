// use maf;

use std::fs;

const USAGE: &str = "USAGE: mafar [unpack or pack] ...
\tunpack [MAF archive file] [path to unpack to]
\tpack   [list of files/directories to pack] [MAF archive out path]";

fn main() {
    macro_rules! exit_usage {
        () => {{
            println!("{}", USAGE);
            return;
        }};
    }

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() < 3 {
        exit_usage!();
    }

    match args[0].as_str() {
        "unpack" => {
            if args.len() != 3 {
                exit_usage!()
            }

            let archive_path = &args[1];
            let out_path = &args[2];

            let Ok(archive_bytes) = fs::read(archive_path) else {
                println!("Couldn't read the provided archive file `{}`", archive_path);
                return;
            };

            let archive = maf::Archive::read(&mut archive_bytes.as_slice()).unwrap();

            for entry in archive.entries() {
                let path = std::path::Path::new(out_path)
                    .join(std::path::Path::new(entry.path.path()));
                // Create the directories
                if let Some(parent) = path.clone().parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                // Write the file entry
                std::fs::write(
                    path
                    .to_str()
                    .unwrap(),
                    &entry.contents,
                )
                .unwrap();
            }
        }
        "pack" => {
            // TODO: Add packing functionality
        }
        _ => exit_usage!(),
    }
}
