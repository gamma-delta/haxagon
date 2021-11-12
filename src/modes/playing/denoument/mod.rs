use ahash::AHashMap;
use cogs_gamedev::controls::InputHandler;
use hex2d::{Coordinate, IntegerSpacing};
use macroquad::{
    audio::{play_sound, play_sound_once, PlaySoundParams},
    prelude::*,
};

use crate::{
    assets::Assets,
    boilerplates::*,
    controls::{Control, InputSubscriber},
    model::{BoardSettings, Marble},
    modes::playing::{BOARD_CENTER_X, BOARD_CENTER_Y, MARBLE_SIZE, MARBLE_SPAN_X, MARBLE_SPAN_Y},
    utils::{
        button::Button,
        draw::{self, hexcolor},
        profile::Profile,
        text::{draw_pixel_text, TextAlign},
    },
    HEIGHT, WIDTH,
};

use super::{ModePlaying, PlaySettings};

/// Transition between having just lost the game and the losing screen
#[derive(Clone)]
pub struct ModeLosingTransition {
    marbles: AHashMap<Coordinate, Marble>,
    radius: usize,
    time: u32,
    /// Score to pass on to the next stage
    score: u32,
    /// if there was a previous score it's here
    prev_score: Option<u32>,

    board_settings: BoardSettings,
    play_settings: PlaySettings,
}

impl Gamemode for ModeLosingTransition {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if self.time == 0 {
            play_sound(
                assets.sounds.end_jingle,
                PlaySoundParams {
                    looped: false,
                    volume: 0.8,
                },
            );
        }
        self.time += 1;

        if self.time > 120 {
            Transition::Swap(Box::new(ModeLosingScreen::new(self)))
        } else {
            Transition::None
        }
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(self.clone())
    }
}

impl GamemodeDrawer for ModeLosingTransition {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo) {
        clear_background(hexcolor(0x14182e_ff));
        // No need to draw background ticks cause they'll all be filled.

        for (pos, marble) in self.marbles.iter() {
            let dark = hexcolor(0x291d2b_ff);

            let scale = self.scale();
            let distance = pos.distance(Coordinate::new(0, 0));
            let (ox, oy) =
                pos.to_pixel_integer(IntegerSpacing::PointyTop(MARBLE_SPAN_X, MARBLE_SPAN_Y));
            let swirl_angle = self.swirl(distance) + (oy as f32).atan2(ox as f32);
            let px_distance = (ox as f32).hypot(oy as f32) * self.spread(distance);

            let corner_x = (swirl_angle.cos() * px_distance as f32 - MARBLE_SIZE / 2.0) * scale
                + BOARD_CENTER_X;
            let corner_y = (swirl_angle.sin() * px_distance as f32 - MARBLE_SIZE / 2.0) * scale
                + BOARD_CENTER_Y;

            let sx = marble.clone() as u32 as f32 * MARBLE_SIZE;
            draw_texture_ex(
                assets.textures.marble_atlas,
                corner_x,
                corner_y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(sx, 8.0, MARBLE_SIZE, MARBLE_SIZE)),
                    dest_size: Some(MARBLE_SIZE * vec2(scale, scale)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                assets.textures.marble_atlas,
                corner_x,
                corner_y,
                dark,
                DrawTextureParams {
                    source: Some(Rect::new(sx, 0.0, MARBLE_SIZE, MARBLE_SIZE)),
                    dest_size: Some(MARBLE_SIZE * vec2(scale, scale)),
                    ..Default::default()
                },
            );
        }

        gl_use_material(assets.shaders.noise);
        let mut fg = hexcolor(0x14182e_ff);
        fg.a = (self.time as f32 / 120.0).powi(4).clamp(0.0, 1.0);
        draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, fg);
        gl_use_default_material();
    }
}

impl ModeLosingTransition {
    /// also saves the score
    pub fn new(prev: &ModePlaying) -> Self {
        let board_settings = prev.board.settings().clone();

        let mut profile = Profile::get();

        let prev_score = if let Some(mk) = board_settings.mode_key {
            match profile.highscores.get_mut(&mk) {
                Some(prev_score) => {
                    // save it so we can return it
                    let save = *prev_score;
                    *prev_score = save.max(prev.board.score());
                    Some(save)
                }
                None => {
                    profile.highscores.insert(mk, prev.board.score());
                    None
                }
            }
        } else {
            None
        };

        Self {
            marbles: prev.board.get_marbles().clone(),
            radius: prev.board.radius(),
            time: 0,
            score: prev.board.score(),
            prev_score,
            board_settings,
            play_settings: prev.settings,
        }
    }

    /// How much to scale up the distance from the center and size of the marble
    fn scale(&self) -> f32 {
        (self.time as f32 / 60.0).powi(4) + 1.0
    }

    fn swirl(&self, distance: i32) -> f32 {
        let rank = distance as f32 / self.radius as f32;
        let x = self.time as f32 + rank * 60.0;
        (x / 10.0 - 10.0).exp().ln_1p() * 0.5 * if distance % 2 == 0 { 1.0 } else { -1.0 }
    }

    fn spread(&self, distance: i32) -> f32 {
        let rank = distance as f32 / self.radius as f32;
        let x = self.time as f32 + rank * 60.0;
        (x / 10.0 - 10.0).exp().ln_1p() * 0.5 + 1.0
    }
}

/// Losing screen, sadde
#[derive(Clone)]
pub struct ModeLosingScreen {
    time: u32,

    score: u32,
    prev_score: Option<u32>,
    /// Settings so we can play again with the same settings if you want
    board_settings: BoardSettings,
    play_settings: PlaySettings,

    b_again: Button,
    b_quit: Button,
}

impl Gamemode for ModeLosingScreen {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        self.time += 1;

        if self.b_again.mouse_hovering() && controls.clicked_down(Control::Click) {
            play_sound_once(assets.sounds.shunt);
            return Transition::Swap(Box::new(ModePlaying::new(
                self.board_settings.clone(),
                self.play_settings.clone(),
            )));
        } else if self.b_quit.mouse_hovering() && controls.clicked_down(Control::Click)
            || controls.clicked_down(Control::Pause)
        {
            play_sound_once(assets.sounds.shunt);
            return Transition::Pop; // back to the title screen
        }

        let mut any_change = false;
        for b in [&mut self.b_again, &mut self.b_quit] {
            if b.mouse_entered() || b.mouse_left() {
                any_change = true;
            }
            b.post_update();
        }
        if any_change {
            play_sound_once(assets.sounds.select);
        }

        Transition::None
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(self.clone())
    }
}

impl GamemodeDrawer for ModeLosingScreen {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo) {
        clear_background(hexcolor(0x14182e_ff));

        let color = hexcolor(0x4b1d52_ff);
        let highlight = hexcolor(0x692464_ff);
        let border = hexcolor(0xcc2f7b_ff);
        let blight = hexcolor(0xff5277_ff);

        let text = match self.prev_score {
            Some(prev) if prev < self.score => format!(
                "GAME OVER\nSCORE: {}\nNEW BEST! PREVIOUS: {}",
                self.score * 100,
                prev * 100
            ),
            Some(_) => format!("GAME OVER\nSCORE: {}", self.score * 100),
            None => format!("GAME OVER\nSCORE: {}\n NEW BEST!", self.score * 100),
        };

        draw_pixel_text(
            &text,
            WIDTH / 2.0,
            HEIGHT * 0.25,
            TextAlign::Center,
            blight,
            assets.textures.fonts.small,
        );

        self.b_again.draw(color, border, highlight, blight, 1.1);
        self.b_quit.draw(color, border, highlight, blight, 1.1);
        draw_pixel_text(
            "PLAY AGAIN",
            self.b_again.x() + self.b_again.w() / 2.0,
            self.b_again.y() + 2.0,
            TextAlign::Center,
            if self.b_again.mouse_hovering() {
                blight
            } else {
                border
            },
            assets.textures.fonts.small,
        );
        draw_pixel_text(
            "QUIT",
            self.b_quit.x() + self.b_quit.w() / 2.0,
            self.b_quit.y() + 2.0,
            TextAlign::Center,
            if self.b_quit.mouse_hovering() {
                blight
            } else {
                border
            },
            assets.textures.fonts.small,
        );

        gl_use_material(assets.shaders.noise);
        let mut fg = hexcolor(0x14182e_ff);
        fg.a = (1.0 - self.time as f32 / 150.0).clamp(0.0, 1.0);
        draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, fg);
        gl_use_default_material();
    }
}

impl ModeLosingScreen {
    pub fn new(prev: &ModeLosingTransition) -> Self {
        let w = 12.0 * 4.0 + 4.0;
        let x = WIDTH / 2.0 - w / 2.0;
        Self {
            score: prev.score,
            prev_score: prev.prev_score,
            board_settings: prev.board_settings.clone(),
            play_settings: prev.play_settings.clone(),
            time: 0,
            b_again: Button::new(x, HEIGHT / 2.0 + 3.0, w, 9.0),
            b_quit: Button::new(x, HEIGHT / 2.0 + 14.0, w, 9.0),
        }
    }
}
