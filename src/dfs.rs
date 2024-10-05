#![allow(dead_code)]
#![allow(unused_variables)]

/// DFS Tools
///
/// BBC Micro DFS disc format
///
///
/// James Macfarlane 2024

use std::io::{self, BufReader, Read, BufWriter, Write};
use std::fs::{File, create_dir_all};
use std::path::Path;

use glob_match::glob_match;

pub const SECTOR_SIZE: usize             = 256;
pub const SECTORS_PER_TRACK: usize       = 10;
pub const TRACK_SIZE: usize              = SECTOR_SIZE * SECTORS_PER_TRACK;
pub const LABEL_SIZE: usize              = 12;
pub const FILENAME_LEN: usize            = 7;

#[derive(Default, Debug, Clone)]
pub struct CatFile {
    name: String,
    dir: char,
    locked: bool,
    load_addr: u32, 
    exec_addr: u32,
    size: u32, 
    sector: u16, 
}

impl CatFile {
    pub fn fullname(&self) -> String {
        format!("{}.{}", self.dir, self.name)
    }

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
    /// Pretty-print the disk catalogue
    pub fn print(&self) {
        self.print_info();
        Self::print_header();
        Self::print_files(&self.files);
    }

    pub fn print_info(&self) {
        let label = self.label.clone();
        //remove_nonprint_chars(label);
        println!("Label: {:11} Cycle: {:}, Tracks: {:2}, Boot Opt: {:}, {:2} files.",
                label,
                self.cycle,
                self.nsectors/SECTORS_PER_TRACK,
                self.boot_option,
                self.nfiles
                );
    }

    pub fn print_header() {
        println!("   Name    Lock    Size    Sector  Load Addr  Exec Addr");
    }

    pub fn print_files(files: &Vec<CatFile>) {
        for f in files {
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

    /// Return all entries matching given (unix glob-style) filename pattern.
    pub fn find(&self, pattern: &str) -> Vec<CatFile> {
        let mut matches: Vec<CatFile> = Vec::new();
        for f in &self.files {
            if glob_match(pattern, &f.name) {
                matches.push(f.clone());
            }
        }
        matches
    }

    pub fn files(&self) -> Vec<CatFile> {
        self.files.clone()
    }
}

#[derive(Debug)]
pub struct DfsImg {
    data: [Vec<u8>; 2],
    dsd: bool,
}

fn ascii_to_char(a: u8) -> char {
    char::from_u32(a as u32).unwrap()
}

impl DfsImg {

    pub fn from_file(filename: &str) -> io::Result<Self> {

        // Use the file extension to determine if it's a single- or doube-side image.
        let path = Path::new(filename);
        let sided: Option<bool> = if let Some(ext) = path.extension() {
            let ext = ext.to_str().expect("Some unicode problem?").to_lowercase();
            if ext == "ssd" {
                Some(false)
            } else if ext == "dsd" {
                Some(true)
            } else {
                None
            }
        } else {
            None
        };

        let dsd = if sided.is_none() {
                eprintln!("Expected extension 'ssd' (single-sided) or 'dsd' (double-sided). Defaulting to single-sided.");
                false
            } else {
                sided.unwrap()
        };

        let my_buf = BufReader::new(File::open(filename)?);
        let data = my_buf.bytes().collect::<Result<Vec<_>,_>>()?;

        let size = data.len() / if dsd { 2 } else { 1 };

        // Check input file size makes sense
        if (size % TRACK_SIZE) > 0 {
            let err = io::Error::other(format!("{}: size of file ({:}) is not a multiple of DFS track size ({}).",
                    filename, size, TRACK_SIZE));
            return Err(err);
        }

        if (size / TRACK_SIZE) < 2 {
            let err = io::Error::other(format!("{}: size of file ({:}) is too small to hold a catalogue.",
                    filename, size));
            return Err(err);
        }

        let tracks = size / TRACK_SIZE;

        // Double-sided image has tracks interleaved
        let mut side0: Vec<u8> = Vec::new();
        let mut side1: Vec<u8> = Vec::new();
        if dsd {
            for t in 0..tracks {
                let start0 = (t * 2 + 0) * TRACK_SIZE;
                let start1 = (t * 2 + 1) * TRACK_SIZE;
                side0.extend_from_slice(&data[start0..start0+TRACK_SIZE]);
                side1.extend_from_slice(&data[start1..start1+TRACK_SIZE]);
            }
        } else {
                side0.extend_from_slice(&data[..]);
        }

        Ok(Self {
            data: [
                side0,
                side1,
            ],
            dsd,
        })
    }

    pub fn dsd(&self) -> bool {
        self.dsd
    }

    fn remove_nonprint_chars(s: String) -> String {
        s.replace(|c: char| !c.is_ascii(), " ")
    }

    fn offs(&self, sfc: u8, sector: usize, offset: usize, len: usize) -> &[u8] {
        let start = sector * SECTOR_SIZE + offset;
        &self.data[sfc as usize][start..start+len]
    }

    fn byte(&self, sfc: u8, sector: usize, offset: usize) -> u8 {
        let start = sector * SECTOR_SIZE + offset;
        self.data[sfc as usize][start]
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
    pub fn cat(&self, sfc: u8) -> Cat {
        let mut cat = Cat::default();

        cat.label =
              Self::str_from_null_term(self.offs(sfc, 0, 0, 8))
            + &Self::str_from_null_term(self.offs(sfc, 1, 0, 3));
        // Cycle Number is in BCD format
        cat.cycle = (self.byte(sfc, 1, 4) >> 4) * 10 + (self.byte(sfc, 1, 4) & 0xf);
        cat.nfiles = (self.byte(sfc, 1, 5) >> 3) as usize;
        if cat.nfiles > (SECTOR_SIZE-8) {
            println!("warning - number of files ({}) too large.", cat.nfiles);
        }
        cat.nsectors = u16::from_le_bytes([
            self.byte(sfc, 1, 7),
            self.byte(sfc, 1, 6) & 0x0f,
        ]) as usize;
        cat.boot_option = (self.byte(sfc, 1, 6) >> 4) & 0xf;
        cat.files = Vec::new();
        for i in 0..cat.nfiles {
            let mut file = CatFile::default();
            let j = 8 + i * 8;
            // Sector 0 contains the name info in 8-byte blocks
            file.name = Self::str_from_null_term(self.offs(sfc, 0, j, 7)).trim().into();
            // Dir and lock state
            file.dir = ascii_to_char(self.byte(sfc, 0, j + 7) & 0x7f);
            file.locked = self.byte(sfc, 0, j + 7) & 0x80 > 0;
     
            // Sector 1 contains the addresses, lengths and locations.
            file.load_addr = u32::from_le_bytes([
                self.byte(sfc, 1, j + 0),
                self.byte(sfc, 1, j + 1),
                (self.byte(sfc, 1, j + 6) >> 2) & 3,
                 0
            ]);
            file.exec_addr = u32::from_le_bytes([
                self.byte(sfc, 1, j + 2),
                self.byte(sfc, 1, j + 3),
                (self.byte(sfc, 1, j + 6) >> 6) & 3,
                0
            ]);
            file.size = u32::from_le_bytes([
                self.byte(sfc, 1, j + 4),
                self.byte(sfc, 1, j + 5),
                (self.byte(sfc, 1, j + 6) >> 4) & 3,
                0
            ]);
            file.sector = u16::from_le_bytes([
                self.byte(sfc, 1, j + 7),
                (self.byte(sfc, 1, j + 6) >> 0) & 3,
            ]);
            cat.files.push(file);
        }
        return cat;
    }

    /// Extract given file catalogue entry to path.
    pub fn extract_file(&self, sfc: u8, file: &CatFile, path: &Path) -> io::Result<()> {

        let pathbuf = path.join(file.fullname());
        let dest = pathbuf.as_path();

        eprintln!("Extracting {:} to {:?}", file.fullname(), &dest);
        let mut my_buf = BufWriter::new(File::create(dest)?);

        my_buf.write_all(self.offs(sfc, file.sector as usize, 0, file.size as usize))?;

        Ok(())
    }

    /// Extract given list of file catalogue entries into dir.
    pub fn extract_files(&self, sfc: u8, files: Vec<CatFile>, dir: &str) -> io::Result<()> {
        let path = Path::new(dir);
        // Create dir if needed
        create_dir_all(dir)?;

        // Extract files
        for f in &files {
            self.extract_file(sfc, f, path)?;
        }


        Ok(())
    }

}
