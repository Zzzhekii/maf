use maf;

use walkdir;
use walkdir::WalkDir;

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

            let Ok(archive_bytes) = std::fs::read(archive_path) else {
                println!("Couldn't read the provided archive file `{}`", archive_path);
                return;
            };

            let archive = maf::Archive::read(&mut archive_bytes.as_slice()).unwrap();

            for entry in archive.entries() {
                let path =
                    std::path::Path::new(out_path).join(std::path::Path::new(entry.path.path()));
                // Create the directories
                if let Some(parent) = path.clone().parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                // Write the file entry
                std::fs::write(path.to_str().unwrap(), &entry.contents).unwrap();
            }
        }
        "pack" => {
            let file_paths = &args[1..args.len() - 1];
            let out_path = &args[args.len() - 1];

            let mut archive_builder = maf::Archive::builder();

            let mut archive_entries: Vec<(maf::Path, Vec<u8>)> = Vec::new();

            for path in file_paths {
                for entry in WalkDir::new(path) {
                    let Ok(entry) = entry else {
                        println!("Couldn't read a source file `{}`", path);
                        continue
                    };

                    if entry.metadata().unwrap().is_dir() {
                        continue;
                    }

                    let relative_path = entry.path().strip_prefix(std::path::Path::new(path)).unwrap();
                    let relative_path = if relative_path.to_str().unwrap().len() > 0 {
                        relative_path
                    } else {
                        std::path::Path::new(entry.path().file_name().unwrap())
                    };

                    archive_entries.push((
                        maf::Path::from_unix_str(relative_path.to_str().unwrap()).unwrap(),
                        std::fs::read(entry.path()).unwrap()
                    ));
                }
            }

            let mut archive_entries: Vec<(maf::Path, &[u8])> = archive_entries
                .iter()
                .map(|(path, contents)| (path.clone(), contents.as_slice()))
                .collect();

            archive_builder.add_entry_list(&mut archive_entries);

            std::fs::write(out_path, archive_builder.build().to_bytes()).unwrap();
        }
        _ => exit_usage!(),
    }
}
