// use maf;

// const USAGE: &str = "USAGE: mafar [unpack or pack] ...
// \tunpack [MAF archive file] [path to unpack to]
// \tpack   [list of files/directories to pack] [MAF archive out path]";

fn main() {
    // TODO: WRITE THE COMMAND LINE INTERFACE YOU SHITFACE!

    // let mut archive_builder = maf::Archive::builder();
    // let archive = archive_builder
    //     .add_entry(
    //         maf::Path::from_unix_str("bruh/lmao.txt").unwrap(),
    //         "Lorem ipsum baby".as_bytes(),
    //     )
    //     .add_entry(
    //         maf::Path::from_unix_str("skull_emoji.txt").unwrap(),
    //         "I read smut about Jesus".as_bytes(),
    //     )
    //     .build();
    // let bytes = archive.to_bytes();
    // std::fs::write("out.maf", bytes).unwrap();

    // let archive = maf::Archive::read(&mut std::fs::read("out.maf").unwrap().as_slice()).unwrap();
    // archive.entries()
    //     .iter()
    //     .for_each(|entry| println!("{}", String::from_utf8(entry.contents.clone()).unwrap()));
}
