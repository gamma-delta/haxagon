use cogs_gamedev::controls::InputHandler;
use macroquad::{
    audio::{play_sound_once},
    prelude::*,
};

use crate::{
    boilerplates::{DrawerBox, FrameInfo, Gamemode, GamemodeDrawer, Transition},
    controls::{Control, InputSubscriber},
    model::PlaySettings,
    utils::{
        button::Button,
        draw::hexcolor,
        profile::Profile,
        text::{draw_pixel_text, TextAlign},
    },
    Assets, HEIGHT,
};

#[derive(Debug, Clone)]
pub struct ModePlaySettings {
    settings: PlaySettings,

    b_background: Button,
    b_animation: Button,

    b_back: Button,
}

impl Gamemode for ModePlaySettings {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        _frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if controls.clicked_down(Control::Click) {
            let mut sound = Some(assets.sounds.close_loop);
            if self.b_background.mouse_hovering() {
                self.settings.funni_background = !self.settings.funni_background;
            } else if self.b_animation.mouse_hovering() {
                self.settings.animations = !self.settings.animations;
            } else if self.b_back.mouse_hovering() {
                sound = Some(assets.sounds.shunt);
            } else {
                sound = None;
            }
            if let Some(sound) = sound {
                play_sound_once(sound);
            }

            if self.b_back.mouse_hovering() {
                let mut profile = Profile::get();
                profile.settings = self.settings;
                return Transition::PopWith(Box::new(self.settings) as _);
            }
        }

        let mut play_enter = false;
        for b in [
            &mut self.b_background,
            &mut self.b_animation,
            &mut self.b_back,
        ] {
            if b.mouse_entered() {
                play_enter = true;
            }
            b.post_update();
        }
        if play_enter {
            play_sound_once(assets.sounds.select);
        }

        Transition::None
    }

    fn get_draw_info(&mut self) -> DrawerBox {
        Box::new(self.clone())
    }
}

impl GamemodeDrawer for ModePlaySettings {
    fn draw(&self, assets: &Assets, _frame_info: FrameInfo) {
        clear_background(hexcolor(0x14182e_ff));

        let color = hexcolor(0x4b1d52_ff);
        let highlight = hexcolor(0x692464_ff);
        let border = hexcolor(0xcc2f7b_ff);
        let blight = hexcolor(0xff5277_ff);

        let line_x = self.b_animation.bounds().right() + 5.0;
        draw_line(line_x, 0.0, line_x, HEIGHT, 1.0, border);

        let msg = if self.b_background.mouse_hovering() {
            Some(format!(
                "ENABLE/DISABLE\nBACKGROUND EFFECTS\n\nCURRENTLY {}",
                if self.settings.funni_background {
                    "ON"
                } else {
                    "OFF"
                }
            ))
        } else if self.b_animation.mouse_hovering() {
            Some(format!("IF ON, MARBLES MOVE\nSMOOTHLY WHEN \nDRAGGED.\nIF OFF, MARBLES JUMP\nTO THEIR\nTARGET POSITIONS.\n\nCURRENTLY {}", if self.settings.animations {
                "ON"
            } else {
                "OFF"
            }))
        } else {
            None
        };
        if let Some(msg) = msg {
            draw_pixel_text(
                &msg,
                line_x + 3.0,
                5.0,
                TextAlign::Left,
                border,
                assets.textures.fonts.small,
            );
        }

        self.b_background
            .draw(color, border, highlight, blight, 1.01);
        let text = format!(
            "BACKGROUND {}",
            if self.settings.funni_background {
                "ON"
            } else {
                "OFF"
            }
        );
        draw_pixel_text(
            &text,
            self.b_background.x() + self.b_background.w() / 2.0,
            self.b_background.y() + 2.0,
            TextAlign::Center,
            if self.b_background.mouse_hovering() {
                blight
            } else {
                border
            },
            assets.textures.fonts.small,
        );

        self.b_animation
            .draw(color, border, highlight, blight, 1.01);
        let text = format!(
            "ANIMATIONS {}",
            if self.settings.animations {
                "ON"
            } else {
                "OFF"
            }
        );
        draw_pixel_text(
            &text,
            self.b_animation.x() + self.b_animation.w() / 2.0,
            self.b_animation.y() + 2.0,
            TextAlign::Center,
            if self.b_animation.mouse_hovering() {
                blight
            } else {
                border
            },
            assets.textures.fonts.small,
        );

        self.b_back.draw(color, border, highlight, blight, 1.01);
        draw_pixel_text(
            "RETURN",
            self.b_back.x() + self.b_back.w() / 2.0,
            self.b_back.y() + 2.0,
            TextAlign::Center,
            if self.b_back.mouse_hovering() {
                blight
            } else {
                border
            },
            assets.textures.fonts.small,
        );
    }
}

impl ModePlaySettings {
    pub fn new(start_settings: PlaySettings) -> Self {
        let x = 5.0;
        let w = 4.0 * 15.0;
        let h = 9.0;
        let y_stride = h + 2.0;
        let y = 5.0;

        Self {
            settings: start_settings,

            b_background: Button::new(x, y, w, h),
            b_animation: Button::new(x, y + y_stride, w, h),
            b_back: Button::new(3.0, HEIGHT - h - 3.0, 4.0 * 12.0, h),
        }
    }
}
