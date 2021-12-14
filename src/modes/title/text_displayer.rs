use cogs_gamedev::controls::InputHandler;
use macroquad::{
    audio::play_sound_once,
    prelude::{clear_background, vec2, Color, Vec2},
};

use crate::{
    assets::Assets,
    boilerplates::{DrawerBox, FrameInfo, Gamemode, GamemodeDrawer, Transition},
    controls::{Control, InputSubscriber},
    utils::{
        button::Button,
        draw::hexcolor,
        text::{draw_pixel_text, Billboard, TextAlign},
    },
    HEIGHT, WIDTH,
};

use super::DontRestartMusicToken;

#[derive(Debug, Clone)]
pub struct ModeTextDisplayer {
    message: String,
    bg_color: Color,
    b_back: Button,
}

impl Gamemode for ModeTextDisplayer {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if (self.b_back.mouse_hovering() && controls.clicked_down(Control::Click))
            || controls.clicked_down(Control::Pause)
        {
            play_sound_once(assets.sounds.shunt);
            return Transition::PopWith(Box::new(DontRestartMusicToken));
        }
        if self.b_back.mouse_entered() {
            play_sound_once(assets.sounds.select);
        }
        self.b_back.post_update();

        Transition::None
    }

    fn get_draw_info(&mut self) -> DrawerBox {
        Box::new(self.clone())
    }
}

impl GamemodeDrawer for ModeTextDisplayer {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo) {
        clear_background(self.bg_color);

        let color = hexcolor(0x4b1d52_ff);
        let highlight = hexcolor(0x692464_ff);
        let border = hexcolor(0xcc2f7b_ff);
        let blight = hexcolor(0xff5277_ff);

        draw_pixel_text(
            &self.message,
            3.0,
            3.0,
            TextAlign::Left,
            blight,
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

impl ModeTextDisplayer {
    pub fn new(message: String, bg_color: Color) -> Self {
        let w = 4.0 * 12.0;
        let h = 9.0;

        Self {
            message,
            bg_color,
            b_back: Button::new(WIDTH - w - 3.0, HEIGHT - h - 3.0, w, h),
        }
    }
}
