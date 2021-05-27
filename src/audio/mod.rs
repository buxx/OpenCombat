use rodio::source::{SamplesConverter, SineWave};
use rodio::{
    source::{Buffered, Source},
    Decoder, OutputStream, OutputStreamHandle,
};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::thread;

type SoundSource = Buffered<Decoder<BufReader<File>>>;

pub enum Sound {
    // Weapons
    GarandM1SingleShot,

    // Mens
    Injured1,
}

pub struct Audio {
    garand_m1_single_shot: SoundSource,
    injured1: SoundSource,
}

fn source(sound_file_string_path: &str) -> SoundSource {
    let file = BufReader::new(File::open(sound_file_string_path).unwrap());
    Decoder::new(file).unwrap().buffered()
}

impl Audio {
    pub fn new() -> Self {
        let garand_m1_single_shot = source("resources/audio/lmg_fire01.mp3");
        let injured1 = source("resources/audio/injured1.mp3");
        Self {
            garand_m1_single_shot,
            injured1,
        }
    }

    // FIXME: Management of sound is not very optimal ... It must be revisited
    pub fn play(&self, sound: Sound) {
        let sound_buffer = match sound {
            Sound::GarandM1SingleShot => self.garand_m1_single_shot.clone(),
            Sound::Injured1 => self.injured1.clone(),
        };

        thread::spawn(|| {
            let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
            let sink = rodio::Sink::try_new(&handle).unwrap();
            sink.append(sound_buffer);
            sink.sleep_until_end();
        });
    }
}
