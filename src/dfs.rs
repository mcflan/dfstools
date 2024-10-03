#![allow(dead_code)]
#![allow(unused_variables)]

/// DFS Tools
///
/// BBC Micro DFS disc format
///
///
/// James Macfarlane 2024

use std::io;
use std::io::BufReader;
use std::fs::File;
use std::io::Read;


pub const SECTOR_SIZE: usize             = 256;
pub const SECTORS_PER_TRACK: usize       = 10;
pub const TRACK_SIZE: usize              = SECTOR_SIZE * SECTORS_PER_TRACK;
pub const LABEL_SIZE: usize              = 12;
pub const FILENAME_LEN: usize            = 7;

#[derive(Default, Debug)]
pub struct CatFile {
    name: String,
    dir: char,
    locked: bool,
    load_addr: u32, 
    exec_addr: u32,
    size: u32, 
    sector: u16, 
}

#[derive(Default, Debug)]
pub struct Cat {
    label: String,
    cycle: u8,
    nfiles: usize,
    files: Vec<CatFile>,
    nsectors: usize,
    boot_option: u8,
}

impl Cat {
    pub fn print(&self) {
        let label = self.label.clone();
        //remove_nonprint_chars(label);
        println!("Label: \"{:}\" Cycle: {:}, Tracks: {:2}, Boot Opt: {:}. {:2} files:",
                label,
                self.cycle,
                self.nsectors/SECTORS_PER_TRACK,
                self.boot_option,
                self.nfiles
                );
        if self.files.len() > 0 {
            println!("   Name    Lock    Size    Sector  Load Addr  Exec Addr");
        }
        for f in &self.files {
            if f.dir == '$' { // Don't show default dir
                print!("   {:-7}  ", f.name);
            } else {
                print!(" {:}.{:-7}  ", f.dir, f.name);
            }
            println!("  {:}  {:6}       {:3}    0x{:05X}    0x{:05X}",
                    if f.locked {"L"} else {" "},
                    f.size, f.sector, f.load_addr, f.exec_addr);
        }
    }
}

#[derive(Debug)]
pub struct DfsImg {
    data: Vec<u8>,
}

fn ascii_to_char(a: u8) -> char {
    char::from_u32(a as u32).unwrap()
}

impl DfsImg {

    pub fn from_file(filename: &str) -> io::Result<Self> {
        // TODO: proper error handling

        let my_buf = BufReader::new(File::open(filename)?);

        let img = Self {
            data: my_buf.bytes().collect::<Result<Vec<_>,_>>()?,
        };

        let size = img.data.len();

        // Check input file size makes sense
        if (size % TRACK_SIZE) > 0 {
            let err = io::Error::other(format!("{:}: size of file ({:}) is not a multiple of DFS track size ({:}).",
                    filename, size, TRACK_SIZE));
            return Err(err);
        }

        if (size / TRACK_SIZE) < 2 {
            let err = io::Error::other(format!("{:}: size of file ({:}) is too small to hold a catalogue.",
                    filename, size));
            return Err(err);
        }

        return Ok(img);
    }

    fn remove_nonprint_chars(s: String) -> String {
        s.replace(|c: char| !c.is_ascii(), " ")
    }

    fn offs(&self, sector: usize, offset: usize, len: usize) -> &[u8] {
        let start = sector * SECTOR_SIZE + offset;
        return &self.data[start..start+len];
    }

    fn byte(&self, sector: usize, offset: usize) -> u8 {
        let start = sector * SECTOR_SIZE + offset;
        return self.data[start];
    }

    fn str_from_null_term(bytes: &[u8]) -> String {
        let mut s = String::new();
        for b in bytes {
            if *b == 0 { break; }
            s.push(char::from_u32(*b as u32).unwrap());
        }
        s
    }

    // Extract catalogue info from a DFS image.
    // See http://www.cowsarenotpurple.co.uk/bbccomputer/native/adfs.html
    // for more info on DFS format, also:
    // https://area51.dev/bbc/bbcmos/filesystems/dfs/
    pub fn cat(&self) -> Cat {
        let mut cat = Cat::default();

        cat.label =
              Self::str_from_null_term(self.offs(0, 0, 8))
            + &Self::str_from_null_term(self.offs(1, 0, 3));
        // Cycle Number is in BCD format
        cat.cycle = (self.byte(1, 4) >> 4) * 10 + (self.byte(1, 4) & 0xf);
        cat.nfiles = (self.byte(1, 5) >> 3) as usize;
        if cat.nfiles > (SECTOR_SIZE-8) {
            println!("warning - number of files ({:}) too large.", cat.nfiles);
        }
        cat.nsectors = u16::from_le_bytes([
            self.byte(1, 7),
            self.byte(1, 6) & 0x0f,
        ]) as usize;
        cat.boot_option = (self.byte(1, 6) >> 4) & 0xf;
        cat.files = Vec::new();
        for i in 0..cat.nfiles {
            let mut file = CatFile::default();
            let j = 8 + i * 8;
            // Sector 0 contains the name info in 8-byte blocks
            file.name = Self::str_from_null_term(self.offs(0, j, 7)).trim().into();
            // Dir and lock state
            file.dir = ascii_to_char(self.byte(0, j + 7) & 0x7f);
            file.locked = self.byte(0, j + 7) & 0x80 > 0;
     
            // Sector 1 contains the addresses, lengths and locations.
            file.load_addr = u32::from_le_bytes([
                self.byte(1, j + 0),
                self.byte(1, j + 1),
                (self.byte(1, j + 6) >> 2) & 3,
                 0
            ]);
            file.exec_addr = u32::from_le_bytes([
                self.byte(1, j + 2),
                self.byte(1, j + 3),
                (self.byte(1, j + 6) >> 6) & 3,
                0
            ]);
            file.size = u32::from_le_bytes([
                self.byte(1, j + 4),
                self.byte(1, j + 5),
                (self.byte(1, j + 6) >> 4) & 3,
                0
            ]);
            file.sector = u16::from_le_bytes([
                self.byte(1, j + 7),
                (self.byte(1, j + 6) >> 0) & 3,
            ]);
            cat.files.push(file);
        }
        return cat;
    }

}
