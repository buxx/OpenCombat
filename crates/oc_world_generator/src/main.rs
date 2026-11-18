use std::path::PathBuf;

use clap::Parser;
use image::{ImageBuffer, Rgb};
use noise::{NoiseFn, Perlin};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    pub width: u64,

    #[clap(long)]
    pub height: u64,

    #[clap()]
    output: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let scale = 250.0; // Adjust for zoom level

    let map = generate_perlin_map(args.width as usize, args.height as usize, scale);
    save_map_as_image(&map, &args.output);

    Ok(())
}

fn generate_perlin_map(width: usize, height: usize, scale: f64) -> Vec<Vec<f64>> {
    let perlin = Perlin::new(42); // Seed for reproducibility
    let mut map = vec![vec![0.0; width]; height];

    map.par_iter_mut().enumerate().for_each(|(y, row)| {
        for (x, cell) in row.iter_mut().enumerate() {
            *cell = perlin.get([(x as f64 / scale), (y as f64 / scale)]);
        }
    });

    map
}

fn save_map_as_image(map: &[Vec<f64>], filename: &PathBuf) {
    let width = map[0].len() as u32;
    let height = map.len() as u32;

    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let value = map[y as usize][x as usize];
            // Normalize to 0-255
            let pixel_value = ((value + 1.0) * 127.5) as u8;
            img.put_pixel(x, y, Rgb([pixel_value, pixel_value, pixel_value]));
        }
    }

    img.save(filename).unwrap();
}
