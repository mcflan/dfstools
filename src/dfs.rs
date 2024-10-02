/// DFS Tools
///
/// BBC Micro DFS disc format
///
///
/// James Macfarlane 2024

use std::ops::{Index, IndexMut};
use std::path::Path;
use libc;
use std::ffi::CString;
use std::io;
use std::io::BufReader;
use std::fs::File;
use std::io::Read;


pub const SECTOR_SIZE: usize             = 256;
pub const SECTORS_PER_TRACK: usize       = 10;
pub const TRACK_SIZE: usize              = DFS_SECTOR_SIZE * DFS_SECTORS_PER_TRACK;
pub const LABEL_SIZE: usize              = 12;
pub const FILENAME_LEN: usize            = 7;

#[derive(Default, Debug)]
struct CatFile {
    name: String,
    dir: u8,
    locked: bool,
    load_addr: u32, 
    exec_addr: u32,
    size: u32, 
    sector: u16, 
}

#[derive(Default, Debug)]
struct Cat {
    label: String,
    nfiles: usize,
    files: Vec<CatFile>,
    nsectors: usize,
    boot_option: u8,
}

#[derive(Debug)]
struct DfsImg {
    data: Vec<u8>,
}

impl DfsImg {

    // Get size of file.
    // Convenience wrapper for fstat.
    fn filesize(file: &str) -> io::Result<usize> {
        unsafe {
            let fname = CString::new(file).unwrap();
            let mut stat: libc::stat = std::mem::zeroed();
            if libc::stat(fname.as_ptr(), &mut stat) >= 0 {
                Ok(stat.st_size as usize)
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub fn from_file(filename: &str) -> io::Result<Self> {
        // TODO: proper error handling

        let size = Self::filesize(filename).expect("File IO Error");

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

        let my_buf = BufReader::new(File::open(filename)?);

        let img = Self {
            data: my_buf.bytes().collect()?,
        };

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
    // for more info on DFS format.
    fn cat(&self) -> Cat {
        let cat = Cat::default();

        cat.label =
              Self::str_from_null_term(self.offs(0, 0, 8))
            + &Self::str_from_null_term(self.offs(1, 0, 3));
        cat.nfiles = (self.byte(1, 5) >> 3) as usize;
        if cat.nfiles > (SECTOR_SIZE-8) {
            println!("warning - number of files ({:}) too large.", cat.nfiles);
        }
        cat.nsectors = (((self.byte(1, 6) & 0x0f) << 8) + self.byte(1, 7)) as usize;
        cat.boot_option = (self.byte(1, 6) >> 4) & 0xf;
        cat.files = Vec::new();
        for i in 0..cat.nfiles {
            let file = CatFile::default();
            let j = 8 + i * 8;
            // Sector 0 contains the name info in 8-byte blocks
            file.name = Self::str_from_null_term(self.offs(0, j, 7)).trim().into();
            // Dir and lock state
            file.dir = self.byte(0, j + 7) & 0x7f;
            file.locked = self.byte(0, j + 7) & 0x80 > 0;
     
            // Sector 1 contains the addresses, lengths and locations.
            file.load_addr =
                *offs(img, 1, 8+i*8 + 0) + 
                (*offs(img, 1, 8+i*8 + 1) << 8) +
                (((*offs(img, 1, 8+i*8 + 6) >> 2) & 3) << 16);
            file.exec_addr =
                *offs(img, 1, 8+i*8 + 2) + 
                (*offs(img, 1, 8+i*8 + 3) << 8) +
                (((*offs(img, 1, 8+i*8 + 6) >> 6) & 3) << 16);
            file.size =
                *offs(img, 1, 8+i*8 + 4) + 
                (*offs(img, 1, 8+i*8 + 5) << 8) +
                (((*offs(img, 1, 8+i*8 + 6) >> 4) & 3) << 16);
            file.sector =
                *offs(img, 1, 8+i*8 + 7) + 
                (((*offs(img, 1, 8+i*8 + 6) >> 0) & 3) << 8);
        }
        return cat;
    }

    void dfs_cat_fprint(FILE *fp, dfs_cat_t *cat)
    {
        assert(cat != NULL);
        assert(cat->files != NULL);
        char *label = strdup(cat->label);
        remove_nonprint_chars(label);
        fprintf(fp, "Label \"%s\", %2d tracks, boot option %2d, %2d files:\n",
                label,
                cat->nsectors/DFS_SECTORS_PER_TRACK,
                cat->boot_option,
                cat->nfiles);
        for (int i = 0; i < cat->nfiles; i++) {
            dfs_cat_file_t *f = cat->files + i;
            if (f->dir == '$') { // Don't show default dir
                fprintf(fp, "  %-7s  ", f->name);
            } else {
                fprintf(fp, "%c.%-7s  ", f->dir, f->name);
            }
            fprintf(fp, " size %6d, sector %3d, load 0x%05X, exec 0x%05X\n",
                    f->size, f->sector, f->load_addr, f->exec_addr);
        }
    }
}
