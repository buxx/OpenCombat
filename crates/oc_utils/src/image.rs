use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

// Warning: partially AI generated
pub fn get_png_dimensions(path: &PathBuf) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; 24];
    file.read_exact(&mut buf)?;

    // PNG signature is 8 bytes, IHDR chunk follows
    // Width is at bytes 16-19, height at bytes 20-23
    if &buf[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("File is not a valid PNG file".into());
    }

    let width = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let height = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);

    Ok((width, height))
}
