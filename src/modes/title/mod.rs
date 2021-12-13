use std::f32::consts::TAU;

use ::rand::Rng;
use cogs_gamedev::controls::InputHandler;
use hex2d::{Angle, Direction};
use macroquad::audio::{play_sound, stop_sound, PlaySoundParams};
use macroquad::rand::compat::QuadRand;
use macroquad::{audio::play_sound_once, miniquad as mq, prelude::*};

use crate::{
    assets::Assets,
    boilerplates::*,
    controls::{Control, InputSubscriber},
    model::{BoardSettings, Marble},
    utils::{
        button::Button,
        draw::{self, hexcolor, mouse_position_pixel},
        text::{draw_pixel_text, TextAlign},
    },
    HEIGHT, WIDTH,
};

use super::playing::PlaySettings;
use super::ModePlaying;

/// How often new hexagons spawn.
// Title screen music is in 12/8, 8th = 200bpm. we want a pulse every 3 beats.
// (60 seconds / 1 minute) * (1 minute / 200 beats) * (3 beats / 1 hex)
// then make it a *little* faster to combat lag.
const HEX_TIMER: f64 = 60.0 / 200.0 * 3.0 * 0.99;

#[derive(Clone)]
pub struct ModeTitle {
    b_play: Button,
    b_mode_select: Button,
    b_tutorial: Button,
    b_settings: Button,
    b_credits: Button,

    prev_hex_time: f64,
    hexagons: Vec<(Vec2, u32)>,

    settings: PlaySettings,
}

impl Gamemode for ModeTitle {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if controls.clicked_down(Control::Click) {
            self.hexagons.push((mouse_position_pixel().into(), 0));
        }
        let now = macroquad::time::get_time();
        if now > self.prev_hex_time + HEX_TIMER {
            self.hexagons.push((vec2(WIDTH / 2.0, HEIGHT / 2.0), 0));
            self.prev_hex_time = now;
        }

        for (_, time) in self.hexagons.iter_mut() {
            *time += 1;
        }
        self.hexagons
            .retain(|(_, time)| hex_radius(*time) < WIDTH * 2.0);

        let mut select_sound = false;
        let mut click_sound = false;
        for button in [
            &self.b_play,
            &self.b_mode_select,
            &self.b_tutorial,
            &self.b_settings,
            &self.b_credits,
        ] {
            if button.mouse_entered() {
                select_sound = true;
                if controls.clicked_down(Control::Click) {}
            }
            if button.mouse_hovering() && controls.clicked_down(Control::Click) {
                click_sound = true;
            }
        }
        if click_sound {
            play_sound_once(assets.sounds.shunt);
        } else if select_sound {
            play_sound_once(assets.sounds.select);
        }

        let mut trans = Transition::None;

        if controls.clicked_down(Control::Click) {
            if self.b_play.mouse_hovering() {
                trans = Transition::Push(Box::new(ModePlaying::new(
                    BoardSettings::classic(),
                    self.settings,
                    assets,
                )));
                stop_sound(assets.sounds.title_music);
            } else if self.b_tutorial.mouse_hovering() {
                // ...
            }
        }

        for button in [
            &mut self.b_play,
            &mut self.b_mode_select,
            &mut self.b_tutorial,
            &mut self.b_settings,
            &mut self.b_credits,
        ] {
            button.post_update();
        }

        trans
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(self.clone())
    }

    fn on_reveal(&mut self, states: Option<Vec<GamemodeBox>>, assets: &Assets) {
        self.hexagons.clear();

        let mut play_music = true;
        if let Some(states) = states {
            // todo: get settings
        }

        if play_music {
            play_sound(
                assets.sounds.title_music,
                PlaySoundParams {
                    looped: true,
                    volume: 0.5,
                },
            );
        }
    }
}

impl GamemodeDrawer for ModeTitle {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo) {
        clear_background(hexcolor(0x14182e_ff));

        if self.settings.funni_background {
            for (pos, time) in self.hexagons.iter() {
                draw_hexagon(
                    pos.x,
                    pos.y,
                    hex_radius(*time),
                    2.0,
                    false,
                    hexcolor(0x9c2a70_ff),
                    hexcolor(0x14182e_ff),
                );
            }
        }

        let logo_x = WIDTH / 2.0 - assets.textures.title_logo.width() / 2.0;
        let logo_y = HEIGHT * 0.2;
        draw_texture(assets.textures.title_logo, logo_x, logo_y, WHITE);

        let color = hexcolor(0x4b1d52_ff);
        let highlight = hexcolor(0x692464_ff);
        let border = hexcolor(0xcc2f7b_ff);
        let blight = hexcolor(0xff5277_ff);

        for (button, text) in [
            (&self.b_play, "PLAY"),
            (&self.b_mode_select, "MODE SELECT"),
            (&self.b_tutorial, "TUTORIAL"),
            (&self.b_settings, "SETTINGS"),
            (&self.b_credits, "CREDITS"),
        ] {
            button.draw(color, border, highlight, blight, 1.1);

            let text_color = if button.mouse_hovering() {
                blight
            } else {
                border
            };
            draw_pixel_text(
                text,
                button.x() + button.w() / 2.0,
                button.y() + 2.0,
                TextAlign::Center,
                text_color,
                assets.textures.fonts.small,
            );
        }
    }
}

impl ModeTitle {
    pub fn new() -> Self {
        let w = 4.0 * 12.0;
        let x = WIDTH / 2.0 - w / 2.0;

        let h = 9.0;
        let y_stride = h + 2.0;
        let y = HEIGHT * 0.5 - y_stride;

        let wide_w = 4.0 * 16.0;
        let wide_x = WIDTH / 2.0 - wide_w / 2.0;

        Self {
            b_play: Button::new(x, y, w, h),
            b_mode_select: Button::new(x, y + y_stride, w, h),
            b_tutorial: Button::new(x, y + 2.0 * y_stride, w, h),
            b_settings: Button::new(x, y + 3.0 * y_stride, w, h),
            b_credits: Button::new(wide_x, y + 4.5 * y_stride, wide_w, h),

            settings: PlaySettings::default(),

            prev_hex_time: 0.0,
            hexagons: Vec::new(),
        }
    }
}

fn hex_radius(time: u32) -> f32 {
    time as f32
}

fn new_directions() -> (Direction, Direction) {
    let in_dir = Direction::from_int(QuadRand.gen_range(0..6));
    let out_dir = in_dir
        + match QuadRand.gen_range(0..5) {
            0 => Angle::Forward,
            1 => Angle::Left,
            2 => Angle::LeftBack,
            3 => Angle::Right,
            4 => Angle::RightBack,
            _ => unreachable!(),
        };
    (in_dir, out_dir)
}
