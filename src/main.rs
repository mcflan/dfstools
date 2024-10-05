#![allow(dead_code)]
#![allow(unused_variables)]

mod dfs;

use clap::Parser;

use dfs::DfsImg;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help="Select side (0 or 1) of double-sided disk")]
    side: Option<u8>,

    #[arg(help="Disk image to load")]
    image: String,

}

fn main() {
    let cli = Cli::parse();
    println!("Disk Image: {:}", cli.image);
    match DfsImg::from_file(&cli.image) {
        Ok(img) => {
            if img.dsd() { 
                if let Some(side) = cli.side {
                    println!("Side {}.", side);
                    img.cat(side).print();
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
        Err(err) => println!("{:?}", err),
    }
}
