use std::path::Path;

use image::{Rgba, RgbaImage};
use oc_mod::Mod;
use oc_root::{files, sprites};

const SOLDIERS_SPRITES: &str = "sprites/soldiers.png";
/// Pixels with an alpha under this value are considered as background
const ALPHA_THRESHOLD: u8 = 16;
/// Outline thickness in pixels
const THICKNESS: i32 = 1;

pub fn cache_soldiers_outline(
    mod_: &Mod,
    mod_path: &Path,
    roots: &files::Roots,
) -> Result<(), anyhow::Error> {
    let files = files::Files::derived_only(roots.clone(), &mod_.canonical(), "");
    let target = files.fs(files::File::IndividualOutline);
    if target.exists() {
        tracing::info!("Soldiers outline already exists ({})", target.display());
        return Ok(());
    }

    let source = mod_path.join(SOLDIERS_SPRITES);
    tracing::info!(
        "Prepare cache for soldiers outline {} from {}",
        target.display(),
        source.display()
    );
    let image = image::open(&source)?.to_rgba8();
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    outline_spritesheet(
        &image,
        sprites::individual::COLUMNS as u32,
        sprites::individual::ROWS as u32,
    )
    .save(&target)?;

    Ok(())
}

// Warning: AI generated
/// For each cell of the spritesheet, paint (in white) transparent pixels touching opaque ones.
/// Outline never overflows its cell.
pub fn outline_spritesheet(image: &RgbaImage, columns: u32, rows: u32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let (cell_w, cell_h) = (width / columns, height / rows);
    let mut outline = RgbaImage::new(width, height);

    for cell_x in 0..columns {
        for cell_y in 0..rows {
            let (x0, y0) = (cell_x * cell_w, cell_y * cell_h);
            for y in 0..cell_h as i32 {
                for x in 0..cell_w as i32 {
                    if is_opaque(image, x0, y0, x, y, cell_w, cell_h) {
                        continue;
                    }
                    let touch = (-THICKNESS..=THICKNESS).any(|dy| {
                        (-THICKNESS..=THICKNESS)
                            .any(|dx| is_opaque(image, x0, y0, x + dx, y + dy, cell_w, cell_h))
                    });
                    if touch {
                        outline.put_pixel(x0 + x as u32, y0 + y as u32, Rgba([255, 255, 255, 255]));
                    }
                }
            }
        }
    }

    outline
}

// Warning: AI generated
fn is_opaque(image: &RgbaImage, x0: u32, y0: u32, x: i32, y: i32, w: u32, h: u32) -> bool {
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return false;
    }
    image.get_pixel(x0 + x as u32, y0 + y as u32).0[3] >= ALPHA_THRESHOLD
}

// Warning: AI generated
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_surrounds_pixel_inside_its_cell() {
        let mut image = RgbaImage::new(8, 4);
        // Opaque pixel in the corner of cell (0, 0)
        image.put_pixel(0, 0, Rgba([10, 20, 30, 255]));

        let outline = outline_spritesheet(&image, 2, 1);

        assert_eq!(outline.get_pixel(0, 0).0[3], 0); // sprite itself is not outlined
        assert_eq!(outline.get_pixel(1, 0).0[3], 255);
        assert_eq!(outline.get_pixel(0, 1).0[3], 255);
        assert_eq!(outline.get_pixel(1, 1).0[3], 255);
        assert_eq!(outline.get_pixel(2, 0).0[3], 0);
        assert_eq!(outline.get_pixel(4, 0).0[3], 0); // other cell untouched
    }
}
