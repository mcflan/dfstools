#![allow(dead_code)]
#![allow(unused_variables)]

mod dfs;

use clap::Parser;

use dfs::DfsImg;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    //#[arg(short, long, help="Trace buffer length")]
    //tracelen: Option<usize>,

    #[arg(help="Disk image to load")]
    image: String,

}

fn main() {
    let cli = Cli::parse();
    println!("Opening {:}", cli.image);
    match DfsImg::from_file(&cli.image) {
        Ok(img) => {
            img.cat().print();
        }
        Err(err) => println!("{:?}", err),
    }
}
