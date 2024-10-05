#![allow(dead_code)]
#![allow(unused_variables)]

mod dfs;

use std::io;
use clap::Parser;

use dfs::DfsImg;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help="Select side (0 or 1) of double-sided disk")]
    side: Option<u8>,

    #[arg(short, long, help="Extract to directory")]
    extract: Option<String>,

    #[arg(short, long, help="Operate only on files matching pattern")]
    pattern: Option<String>,

    #[arg(help="Disk image to load")]
    image: String,

}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = ops(cli) {
        eprintln!("{:?}", err);
    }
}

fn ops(cli: Cli) -> io::Result<()> {
    println!("Disk Image: {:}", cli.image);
    let img = DfsImg::from_file(&cli.image)?;
    if let Some(dir) = cli.extract {
        // Extract files
        let sfc = cli.side.unwrap_or(0);
        let files = img.cat(sfc).files();
        img.extract_files(sfc, files, &dir)?;
    } else {
        // Display catalogue
        cat(&img, cli.side);
    }
    Ok(())
}

fn cat(img: &DfsImg, side: Option<u8>) {
    if img.dsd() { 
        if let Some(sfc) = side {
            println!("Side {}.", sfc);
            img.cat(sfc).print();
        } else {
            print!("Side 0: ");
            img.cat(0).print();
            print!("Side 1: ");
            img.cat(1).print();
        }
    } else {
        img.cat(0).print();
    }
}
