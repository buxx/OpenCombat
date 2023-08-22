use battle_core::audio::Sound;
use ggez::{
    audio::{SoundSource, Source},
    Context, GameResult,
};
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub struct Player {
    sounds: HashMap<Sound, Source>,
}

impl Player {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut sounds = HashMap::new();

        for sound in Sound::iter() {
            sounds.insert(sound, Source::new(ctx, sound.file_path())?);
        }

        Ok(Self { sounds })
    }

    pub fn play(&mut self, sound: &Sound, ctx: &mut Context) -> GameResult {
        puffin::profile_scope!("play_sound", sound.to_string());

        match self.sounds.get_mut(sound) {
            Some(source) => {
                source.play_detached(ctx)?;
            }
            None => {
                println!("ERROR :: Unknown sound {:?}", sound)
            }
        };

        Ok(())
    }
}
