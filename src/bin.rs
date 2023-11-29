use std::fmt::format;
use maf;

const USAGE: &str = "USAGE: mafar [unpack or pack] ...
\tunpack [MAF archive file] [path to unpack to]
\tpack   [list of files/directories to pack] [MAF archive out path]";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("{}", USAGE);
        return;
    }

    match args[1].as_str() {
        "pack" => {
            let list_to_pack = &args[2..args.len() - 1];
            let out = &args[args.len() - 1];

            let mut archive_builder = maf::Builder::new();

            let contents: Vec<Vec<u8>> = list_to_pack
                .iter()
                .map(|path| std::fs::read(path).expect(&format!("Couldn't read the file `{}`.", path)))
                .collect();

            for i in 0..contents.len() {
                archive_builder.add_entry(maf::Path(list_to_pack[i].as_bytes()), contents[i].as_slice());
            }

            std::fs::write(out, archive_builder.build().to_raw_bytes())
                .expect(&format!("Couldn't write the archive to `{}`.", out));
        },
        "unpack" => {
            if args.len() != 4 {
                println!("{}", USAGE);
                return;
            }

            let archive_path = &args[2];
            let out_path = &args[3];

            let archive_raw = std::fs::read(archive_path).expect(&format!("Couldn't read the MAF archive `{}`.", archive_path));

            let archive = maf::Archive::from_raw_bytes(archive_raw.as_slice()).unwrap();

            for (path, contents) in archive.list_entries() {
                let out= String::from_utf8([out_path.as_bytes(), "/".as_bytes(), path].concat()).unwrap();
                std::fs::write(&out, contents)
                    .expect(&format!("Couldn't write the file out to `{}`.", out));
            }
        }
        _ => {
            println!("{}", USAGE);
            return;
        }
    }

    // let contents1 = std::fs::read("test1.txt").expect("Should have been able to read the file");
    //
    // let contents2 = std::fs::read("test2.txt").expect("Should have been able to read the file");
    //
    // let mut archive_builder = maf::Builder::new();
    //
    // let raw = archive_builder
    //     .add_entry(
    //         maf::Path("ashtsaht/ahstahst/asht.txt".as_bytes()),
    //         &contents1,
    //     )
    //     .add_entry(
    //         maf::Path("ashtsaht/ahstahst/ashta.txt".as_bytes()),
    //         &contents2,
    //     )
    //     .build()
    //     .to_raw_bytes()
    //     .leak();
    //
    // let archive = maf::Archive::from_raw_bytes(raw).unwrap();
    //
    // println!("HEADER {:?}", archive.header);
    // println!("ENTRY MAPPING LIST {:?}", archive.entry_mapping_list);
    // println!("ENTRY PATH LIST {:?}", archive.entry_path_list);
    //
    // archive.entry_data_list.iter().for_each(|data| println!("\nDATA\n{}\n", String::from_utf8(data.to_vec()).unwrap()))
}
