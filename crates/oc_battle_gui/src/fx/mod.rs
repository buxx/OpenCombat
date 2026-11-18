use crate::states::GuiFiles;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_hanabi::prelude::EffectAsset;
use oc_physics::fx::{Audio, Fx, SoundBus, Visual};
use oc_root::{WcfgFrom, geo::ScreenVec2};
use oc_utils::let_some;

use crate::{ingame::BattlePhase, states::GameConfig};

mod explosion;
mod flash;
mod particles;
mod smoke;

#[allow(unused_imports, reason = "used by examples")]
pub use smoke::Smoke;

/// Too many at once starve the output buffer. Extra spatial sounds are dropped.
const MAX_CONCURRENT_EMITTERS: usize = 64;

#[derive(Debug, Event)]
pub struct FxEvent(pub Fx);

/// Spatial sound
#[derive(Component)]
struct Emitter;

#[derive(Component)]
struct Bus(SoundBus);

#[derive(Debug, Resource)]
pub struct BusVolumes {
    pub master: f32,
    pub fire: f32,
    pub effects: f32,
    pub explosion: f32,
    pub ui: f32,
}

impl Default for BusVolumes {
    fn default() -> Self {
        Self {
            master: 1.0,
            fire: 0.7,
            effects: 0.7,
            explosion: 2.5,
            ui: 0.6,
        }
    }
}

#[derive(Debug, Event)]
pub struct SetBusVolume(pub SoundBus, pub f32);

#[derive(Debug, Event)]
pub struct SetMasterVolume(pub f32);

fn on_set_bus_volume(set: On<SetBusVolume>, mut volumes: ResMut<BusVolumes>) {
    *volumes.bus_mut(set.0) = set.1;
}

fn on_set_master_volume(set: On<SetMasterVolume>, mut volumes: ResMut<BusVolumes>) {
    volumes.master = set.0;
}

impl BusVolumes {
    pub fn linear(&self, bus: SoundBus) -> f32 {
        match bus {
            SoundBus::Fire => self.fire,
            SoundBus::Effects => self.effects,
            SoundBus::Explosion => self.explosion,
            SoundBus::Ui => self.ui,
        }
    }

    fn bus_mut(&mut self, bus: SoundBus) -> &mut f32 {
        match bus {
            SoundBus::Fire => &mut self.fire,
            SoundBus::Effects => &mut self.effects,
            SoundBus::Explosion => &mut self.explosion,
            SoundBus::Ui => &mut self.ui,
        }
    }

    pub fn volume(&self, bus: SoundBus) -> Volume {
        Volume::Linear(self.master * self.linear(bus))
    }
}

fn apply_bus_volumes(
    volumes: Res<BusVolumes>,
    mut spatial: Query<(&mut SpatialAudioSink, &Bus)>,
    mut flat: Query<(&mut AudioSink, &Bus)>,
) {
    if !volumes.is_changed() {
        return;
    }
    for (mut sink, bus) in &mut spatial {
        sink.set_volume(volumes.volume(bus.0));
    }
    for (mut sink, bus) in &mut flat {
        sink.set_volume(volumes.volume(bus.0));
    }
}

pub struct FxPlugin;

// See https://taintedcoders.com/bevy/audio
impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BusVolumes>()
            .add_observer(on_fx)
            .add_observer(on_set_bus_volume)
            .add_observer(on_set_master_volume)
            .add_systems(Startup, (setup, particles::setup))
            .add_systems(
                Update,
                (
                    update_listener,
                    apply_bus_volumes,
                    explosion::animate,
                    particles::despawn_finished,
                ),
            );
        #[cfg(feature = "debug")]
        app.add_systems(Update, explosion::draw_debug);
    }
}

pub fn play_phase_start(commands: &mut Commands, g: &GameConfig, phase: BattlePhase) {
    let_some!(g = &g.0, return);
    let groups = &g.mod_.sound_groups;
    let sound = match phase {
        BattlePhase::Deployment => groups.deployment_start,
        BattlePhase::Fight => groups.fight_start,
        BattlePhase::End => None,
    };
    let_some!(sound = sound, return);
    commands.trigger(FxEvent(Fx::Audio(Audio::Play(sound))));
}

fn setup(mut commands: Commands) {
    // Space between the two ears
    let gap = 350.0; // FIXME: update when window size change ?

    commands.spawn((Transform::default(), SpatialListener::new(gap)));
}

#[allow(clippy::too_many_arguments)]
fn on_fx(
    fx: On<FxEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    g: Res<GameConfig>,
    files: Res<GuiFiles>,
    volumes: Res<BusVolumes>,
    emitters: Query<(), With<Emitter>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut effects: ResMut<Assets<EffectAsset>>,
    glow: Option<Res<particles::GlowTexture>>,
) {
    let_some!(g = &g.0, return);
    let_some!(files = &files.0, return);
    let path = files.sounds();

    match &fx.0 {
        Fx::Audio(audio) => match audio {
            Audio::PlayWithGeo(sound, position, bus) => {
                if emitters.iter().count() >= MAX_CONCURRENT_EMITTERS {
                    tracing::debug!(name = "fx-too-many-emitters", sound=?sound);
                    return;
                }
                let sound = g.mod_.sound(*sound);
                let path = path.join(&sound.name);
                tracing::debug!(name = "fx-audio-geo", path=?path, position=?position);
                let position = ScreenVec2::from_(*position, &g.w);
                commands.spawn((
                    Emitter,
                    Bus(*bus),
                    Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(volumes.volume(*bus)),
                ));
            }
            Audio::Play(sound) => {
                let sound = g.mod_.sound(*sound);
                let path = path.join(&sound.name);
                commands.spawn((
                    Bus(SoundBus::Ui),
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::DESPAWN.with_volume(volumes.volume(SoundBus::Ui)),
                ));
            }
        },
        Fx::Visual(visual) => match visual {
            Visual::Explosion(position, rayon) => {
                tracing::debug!(name = "fx-explosion", position=?position, rayon=?rayon);
                explosion::spawn(
                    &mut commands,
                    &asset_server,
                    &mut layouts,
                    g,
                    files,
                    *position,
                    *rayon,
                );
            }
            Visual::Flash(position, rayon) => {
                let_some!(glow = glow.as_deref(), return);
                tracing::debug!(name = "fx-flash", position=?position, rayon=?rayon);
                flash::spawn(&mut commands, &mut effects, glow, g, *position, *rayon);
            }
            Visual::Smoke(position, hold, fade) => {
                let_some!(glow = glow.as_deref(), return);
                tracing::debug!(name = "fx-smoke", position=?position, hold=hold, fade=fade);
                smoke::spawn(
                    &mut commands,
                    &mut effects,
                    glow,
                    g,
                    *position,
                    *hold,
                    *fade,
                );
            }
            #[cfg(feature = "debug")]
            Visual::ExplosionDebug(debug) => explosion::spawn_debug(&mut commands, g, debug),
        },
    }
}

fn update_listener(
    camera: Single<&Transform, (With<Camera2d>, Without<SpatialListener>)>,
    mut listener: Single<&mut Transform, With<SpatialListener>>,
) {
    listener.translation = camera.translation;
}
