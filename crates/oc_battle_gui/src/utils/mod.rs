use std::path::PathBuf;

pub mod drag;
pub mod hover;
pub mod pick_radius;
pub mod point;
pub mod selected;

#[macro_export]
macro_rules! http_to_file {
    ($url:expr, $path:expr) => {
        if let Some(parent) = $path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut resp = reqwest::blocking::get($url)?.error_for_status()?;
        let mut file = std::fs::File::create($path)?;
        std::io::copy(&mut resp, &mut file)?;
    };
}

pub fn untar(path: &PathBuf, destination: &PathBuf) -> Result<(), std::io::Error> {
    let file = std::fs::File::open(path)?;
    tracing::info!("Decompress {} to {}", path.display(), destination.display());
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(destination)?;
    std::fs::remove_file(path)?;
    Ok(())
}
