use battle_core::physics::event::cannon_blast::CannonBlast;



use crate::engine::message::EngineMessage;
use crate::engine::Engine;
use crate::graphics::message::GraphicsMessage;

impl Engine {
    pub fn tick_cannon_blasts(&self) -> Vec<EngineMessage> {
        puffin::profile_scope!("tick_cannon_blasts");
        let mut messages = vec![];
        let frame_i = self.gui_state.frame_i();

        for canon_blast in self.battle_state.cannon_blasts() {
            messages.extend(self.canon_blast_fx(canon_blast));

            if canon_blast.start() == frame_i {
                messages.push(EngineMessage::Graphics(
                    GraphicsMessage::PushCanonBlastAnimation(
                        *canon_blast.point(),
                        *canon_blast.angle(),
                        canon_blast.weapon_sprite_type().clone(),
                        canon_blast.soldier_animation_type().clone(),
                    ),
                ))
            }

            if canon_blast.finished(frame_i) {
                // TODO : Remove by self.point can remove other cannon_blasts. Find better methodology
                messages.push(EngineMessage::Graphics(
                    GraphicsMessage::RemoveCanonBlastAnimation(*canon_blast.point()),
                ))
            }
        }

        messages
    }

    pub fn canon_blast_fx(&self, canon_blast: &CannonBlast) -> Vec<EngineMessage> {
        let mut messages = vec![];

        if canon_blast.start() == self.gui_state.frame_i() {
            // FIXME BS NOW : sounds (like explosion) should be generated here (instead on gesture ?)
            // FIXME BS NOW : animation is not already pushed in tick_cannon_blasts ?
            messages.push(EngineMessage::Graphics(
                GraphicsMessage::PushCanonBlastAnimation(
                    *canon_blast.point(),
                    *canon_blast.angle(),
                    canon_blast.weapon_sprite_type().clone(),
                    canon_blast.soldier_animation_type().clone(),
                ),
            ));
        }

        messages
    }

    // pub fn draw_cannon_blasts(&self, _mesh_builder: &mut MeshBuilder) -> GameResult {
    //     // Nothing here because drawn by graphics sequences
    //     Ok(())
    // }
}
