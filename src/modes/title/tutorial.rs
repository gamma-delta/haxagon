use cogs_gamedev::controls::InputHandler;
use macroquad::{
    audio::play_sound_once,
    prelude::{clear_background, vec2, Vec2},
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

const VERB: &str = if cfg!(any(target_os = "ios", target_os = "android")) {
    "TAP"
} else {
    "CLICK"
};

#[derive(Debug, Clone)]
pub struct ModeTutorial {
    b_back: Button,

    billboard: Billboard,
}

impl Gamemode for ModeTutorial {
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

impl GamemodeDrawer for ModeTutorial {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo) {
        clear_background(hexcolor(0x21181b_ff));

        self.billboard.draw();

        let color = hexcolor(0x4b1d52_ff);
        let highlight = hexcolor(0x692464_ff);
        let border = hexcolor(0xcc2f7b_ff);
        let blight = hexcolor(0xff5277_ff);

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

impl ModeTutorial {
    pub fn new(assets: &Assets) -> Self {
        let msg = [
            "[$cff5277$HAXAGON INSTRUCTIONS\n\n",
            VERB,
            " AND DRAG ON THE BOARD IN A\n",
            "CLOSED LOOP TO MOVE MARBLES.[$v3$\n$v]",
            "MAKE GROUPS OF 4 OR MORE MARBLES\n",
            "TO CLEAR THEM FOR POINTS.[$v3$\n$v]",
            "NEW MARBLES SPAWN AT THE RED DOT.\n",
            "DON'T LET THE BOARD FILL UP!",
        ]
        .concat();
        let spans = Billboard::from_markup(msg, assets.textures.fonts.small).unwrap();

        let w = 4.0 * 12.0;

        Self {
            b_back: Button::new(WIDTH - w - 3.0, 3.0, w, 9.0),

            billboard: Billboard::new(spans, vec2(3.0, 10.0), Vec2::ZERO, None),
        }
    }
}