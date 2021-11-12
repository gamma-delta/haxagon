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

/// How often new hexagons spawn
const HEX_TIMER: u32 = 50;
const MARBLE_TIMER: u32 = 60;

#[derive(Clone)]
pub struct ModeSplash {
    b_classic: Button,
    b_advanced: Button,
    b_static: Button,
    b_toggle_background: Button,

    hex_timer: u32,
    hexagons: Vec<(Vec2, u32)>,

    settings: PlaySettings,

    marble_timer: u32,
    marble: Marble,
    in_dir: Direction,
    out_dir: Direction,
}

impl Gamemode for ModeSplash {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if controls.clicked_down(Control::Click) || self.hex_timer == 0 {
            let pos = if controls.clicked_down(Control::Click) {
                mouse_position_pixel().into()
            } else {
                self.hex_timer = HEX_TIMER;
                vec2(WIDTH / 2.0, HEIGHT / 2.0)
            };
            self.hexagons.push((pos, 0));
        } else {
            self.hex_timer -= 1;
        }

        if self.marble_timer > MARBLE_TIMER {
            self.marble_timer = 0;
            self.marble = Marble::random(7);
            let (in_dir, out_dir) = new_directions();
            self.in_dir = in_dir;
            self.out_dir = out_dir;
        } else {
            self.marble_timer += 1;
        }

        for (_, time) in self.hexagons.iter_mut() {
            *time += 1;
        }
        self.hexagons
            .retain(|(_, time)| hex_radius(*time) < WIDTH * 2.0);

        if controls.clicked_down(Control::Click) {
            let mut fwshh = false;
            if self.b_toggle_background.mouse_hovering() {
                self.settings.funni_background = !self.settings.funni_background;
                fwshh = true;
            }

            let next_settings = if self.b_classic.mouse_hovering() {
                Some(BoardSettings::classic())
            } else if self.b_advanced.mouse_hovering() {
                Some(BoardSettings::advanced())
            } else if self.b_static.mouse_hovering() {
                Some(BoardSettings::no_gravity())
            } else {
                None
            };
            fwshh |= next_settings.is_some();

            if fwshh {
                play_sound_once(assets.sounds.shunt);
            }

            if let Some(settings) = next_settings {
                stop_sound(assets.sounds.title_music);
                return Transition::Push(Box::new(ModePlaying::new(settings, self.settings)));
            }
        }

        let mut select_sound = false;
        for button in [
            &mut self.b_classic,
            &mut self.b_advanced,
            &mut self.b_static,
            &mut self.b_toggle_background,
        ] {
            if button.mouse_entered() || button.mouse_left() {
                select_sound = true;
            }
            button.post_update();
        }
        if select_sound {
            play_sound_once(assets.sounds.select);
        }

        Transition::None
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(self.clone())
    }

    fn on_reveal(&mut self, assets: &Assets) {
        self.hexagons.clear();
        self.hex_timer = HEX_TIMER;
        play_sound(
            assets.sounds.title_music,
            PlaySoundParams {
                looped: true,
                volume: 0.5,
            },
        )
    }
}

impl GamemodeDrawer for ModeSplash {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo) {
        clear_background(hexcolor(0x14182e_ff));

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

        let logo_x = WIDTH / 2.0 - assets.textures.title_logo.width() / 2.0;
        let logo_y = HEIGHT * 0.2;
        draw_texture(assets.textures.title_logo, logo_x, logo_y, WHITE);

        /*
        // Fill in the stencil
        gl_use_material(assets.shaders.stencil_write);
        draw_texture(assets.textures.splash_stencil, logo_x, logo_y, WHITE);

        // Now things here will be only inside the stencil--fancy dancy
        let sx = self.marble.clone() as u32 as f32 * 8.0;
        let go_in = self.marble_timer < MARBLE_TIMER / 2;
        let angle: f32 = if go_in {
            self.in_dir.to_radians_flat::<f32>()
        } else {
            self.out_dir.to_radians_flat()
        } + TAU / 12.0;
        let t = if go_in {
            1.0 - self.marble_timer as f32 / (MARBLE_TIMER as f32 / 2.0)
        } else {
            (self.marble_timer as f32 - MARBLE_TIMER as f32 / 2.0) / (MARBLE_TIMER as f32 / 2.0)
        };
        let x = logo_x + 111.0 + t * 16.0 * angle.cos();
        let y = logo_y + 5.0 + t * 16.0 * angle.sin();

        gl_use_material(assets.shaders.stencil_mask);
        draw_texture_ex(
            assets.textures.marble_atlas,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(sx, 8.0, 8.0, 8.0)),
                ..Default::default()
            },
        );
        draw_texture_ex(
            assets.textures.marble_atlas,
            x,
            y,
            hexcolor(0x291d2b_ff),
            DrawTextureParams {
                source: Some(Rect::new(sx, 0.0, 8.0, 8.0)),
                ..Default::default()
            },
        );

        gl_use_default_material();
        */

        let color = hexcolor(0x4b1d52_ff);
        let highlight = hexcolor(0x692464_ff);
        let border = hexcolor(0xcc2f7b_ff);
        let blight = hexcolor(0xff5277_ff);
        let bg_text = if self.settings.funni_background {
            "BACKGROUND ON"
        } else {
            "BACKGROUND OFF"
        };

        for (button, text) in [
            (&self.b_classic, "CLASSIC"),
            (&self.b_advanced, "ADVANCED"),
            (&self.b_static, "STATIC"),
            (&self.b_toggle_background, bg_text),
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

impl ModeSplash {
    pub fn new() -> Self {
        let w = 4.0 * 12.0;
        let x = WIDTH / 2.0 - w / 2.0;

        let h = 9.0;
        let y = HEIGHT * 0.5;

        let wide_w = 4.0 * 16.0;
        let wide_x = WIDTH / 2.0 - wide_w / 2.0;

        let (in_dir, out_dir) = new_directions();

        Self {
            b_classic: Button::new(x, y - h - 2.0, w, h),
            b_advanced: Button::new(x, y, w, h),
            b_static: Button::new(x, y + h + 2.0, w, h),
            b_toggle_background: Button::new(wide_x, y + (h + 2.0) * 2.5, wide_w, h),

            settings: PlaySettings::default(),

            hex_timer: 0,
            hexagons: Vec::new(),

            marble: Marble::random(7),
            in_dir,
            out_dir,
            marble_timer: 0,
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
