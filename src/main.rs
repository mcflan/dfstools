#![allow(dead_code)]
#![allow(unused_variables)]

mod dfs;

use std::io;
use clap::Parser;

use dfs::{CatFile, DfsImg};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help="Select surface (0 or 1) of double-sided disk")]
    side: Option<u8>,

    #[arg(short, long, help="Extract to directory", value_name="DIR")]
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
    let sfc = cli.side.unwrap_or(0);
    let ct = img.cat(sfc);
    let files = if let Some(pat) = cli.pattern {
        ct.find(&pat)
    } else {
        ct.files()
    };
    if let Some(dir) = cli.extract {
        // Extract files
        img.extract_files(sfc, files, &dir)?;
    } else {
        // Display catalogue
        cat_print(&img, cli.side, files);
    }
    Ok(())
}

fn cat_print(img: &DfsImg, side: Option<u8>, files: Vec<CatFile>) {
    if img.dsd() { 
        if let Some(sfc) = side {
            println!("Side {}.", sfc);
            img.cat(sfc).print(Some(&files));
        } else {
            print!("Side 0: ");
            img.cat(0).print(None);
            print!("Side 1: ");
            img.cat(1).print(None);
        }
    } else {
        img.cat(0).print(Some(&files));
    }
}
