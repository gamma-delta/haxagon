use ahash::AHashMap;
use cogs_gamedev::{controls::InputHandler, grids::Coord};
use hex2d::{Angle, Coordinate, Spacing};
use itertools::Itertools;
use macroquad::{
    audio::{play_sound, stop_sound, PlaySoundParams},
    prelude::{mouse_position, vec2, Mat2},
};

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, Gamemode, GamemodeDrawer, Transition},
    controls::{Control, InputSubscriber},
    model::{Board, BoardAction, BoardSettings, Marble},
    utils::draw::mouse_position_pixel,
    HEIGHT, WIDTH,
};

use self::{denoument::ModeLosingTransition, draw::Drawer};

mod denoument;
mod draw;

const BOARD_CENTER_X: f32 = WIDTH / 2.0;
const BOARD_CENTER_Y: f32 = HEIGHT / 2.0;

/// Diameter of the marble itself
const MARBLE_SIZE: f32 = 8.0;
/// Horizontal distance between marbles
const MARBLE_SPAN_X: i32 = 10;
/// Vertical distance between marbles
const MARBLE_SPAN_Y: i32 = 8;

pub struct ModePlaying {
    pub board: Board,
    pub pattern: Option<Vec<Coordinate>>,

    pub bg_funni_timer: f32,

    pub played_music: bool,
    pub paused: bool,
}

impl Gamemode for ModePlaying {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if !self.played_music {
            self.played_music = true;
            play_sound(
                assets.sounds.music1,
                PlaySoundParams {
                    looped: true,
                    volume: 0.5,
                },
            );
        }

        if self.paused {
            let (mx, my) = mouse_position_pixel();
            let unpause = controls.clicked_down(Control::Pause)
                || controls.clicked_down(Control::Click)
                    && (0.0..=WIDTH).contains(&mx)
                    && (0.0..=HEIGHT).contains(&my);
            if unpause {
                self.paused = false;
            }

            Transition::None
        } else {
            self.actually_update(controls, assets)
        }
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        let marbles = self
            .board
            .get_marbles()
            .iter()
            .map(|(c, m)| (*c, m.clone()))
            .collect();
        let next_action = self.board.next_action().cloned();
        let to_remove = if let Some(BoardAction::ClearBlobs(_)) = &next_action {
            self.board.find_blobs().into_iter().flatten().collect()
        } else {
            Vec::new()
        };
        let next_action = next_action.map(|action| (action, self.board.action_timer()));

        Box::new(Drawer {
            marbles,
            pattern: self.pattern.clone(),
            next_spawn_point: self.board.next_spawn_point(),
            radius: self.board.radius(),
            next_action,
            to_remove,
            bg_funni_timer: self.bg_funni_timer,
            score: self.board.score(),
            paused: self.paused,
        })
    }
}

impl ModePlaying {
    pub fn new(settings: BoardSettings) -> Self {
        Self {
            board: Board::new(settings),
            pattern: None,
            bg_funni_timer: 0.0,
            played_music: false,
            paused: false,
        }
    }

    /// The actual update code when not paused
    fn actually_update(&mut self, controls: &InputSubscriber, assets: &Assets) -> Transition {
        let (mx, my) = mouse_position_pixel();
        let pause = controls.clicked_down(Control::Pause)
            || (controls.clicked_down(Control::Click) && !(0.0..=WIDTH).contains(&mx)
                || !(0.0..=HEIGHT).contains(&my));
        if pause {
            self.paused = true;
            return Transition::None;
        }

        match &mut self.pattern {
            None if controls.clicked_down(Control::Click) => {
                let pos = mouse_to_hex();
                if self.board.is_in_bounds(&pos) {
                    self.pattern = Some(vec![pos])
                }
            }
            Some(pat) if controls.pressed(Control::Click) => {
                let pos = mouse_to_hex();
                if self.board.is_in_bounds(&pos) {
                    let mut maybe_pat = pat.clone();
                    maybe_pat.push(pos);
                    match is_pattern_valid(&maybe_pat, self.board.get_marbles()) {
                        PatternExtensionValidity::Continue | PatternExtensionValidity::Closing => {
                            *pat = maybe_pat;
                            play_sound(
                                assets.sounds.select,
                                PlaySoundParams {
                                    looped: false,
                                    volume: 1.0,
                                },
                            );
                        }
                        PatternExtensionValidity::Invalid => {}
                    }
                }
            }
            // mouse up but with pattern
            Some(pat) => {
                if PatternExtensionValidity::Closing
                    == is_pattern_valid(pat, self.board.get_marbles())
                {
                    let pat = std::mem::take(pat);
                    let action = self.pattern_to_action(pat);

                    self.board.push_action(action);
                    self.board.push_action(BoardAction::ClearBlobs(1));
                }
                // if we're not pressing gotta clear it
                self.pattern = None;
            }
            None => {}
        }

        if let Some(next_action) = self.board.next_action() {
            let timer = self.board.action_timer();
            let finish_time = next_action.time();
            let sound = match next_action {
                BoardAction::Cycle(_) if timer == 0 => Some((assets.sounds.shunt, 1.0)),
                BoardAction::DeleteColor(_) if timer == 0 => Some((assets.sounds.clear_all, 1.0)),
                BoardAction::ClearBlobs(mult) if timer == finish_time - 1 => {
                    let mult = self.board.find_blobs().len() as u32 + *mult;
                    let sound = match mult {
                        // There will always be at least 1 blob
                        0..=2 => assets.sounds.clear1,
                        3 => assets.sounds.clear2,
                        _ => assets.sounds.clear3,
                    };
                    Some((sound, 1.0))
                }
                _ => None,
            };
            if let Some((sound, volume)) = sound {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume,
                    },
                );
            }
        }

        let failure = self.board.tick();
        if failure {
            stop_sound(assets.sounds.music1);
            return Transition::Swap(Box::new(ModeLosingTransition::new(self)));
        }

        let dist = if let Some(sp) = self.board.next_spawn_point() {
            sp.distance(Coordinate::new(0, 0)) as f32
        } else {
            -1.0
        };
        let speed = 1.0 - ((dist - 1.0) / self.board.radius() as f32);
        self.bg_funni_timer += speed.sqrt();

        Transition::None
    }

    /// always follow this with a clear blobs sil vous plait
    fn pattern_to_action(&self, mut pat: Vec<Coordinate>) -> BoardAction {
        // Chexagon if it's a hexagon
        let is_hexagon = || {
            // Note that everything is already looped
            let deltas = pat
                .windows(2)
                .map(|span| *span[0].directions_to(span[1]).first().unwrap())
                .collect::<Vec<_>>();
            let angles = deltas
                .windows(2)
                .map(|span| span[1] - span[0])
                .collect::<Vec<_>>();

            let all_corners_same = angles
                .iter()
                .enumerate()
                .filter_map(|(idx, a)| {
                    if *a == Angle::Left || *a == Angle::Right {
                        Some(self.board.get_marble(&pat[idx + 1]))
                    } else {
                        None
                    }
                })
                .chain(std::iter::once(self.board.get_marble(&pat[0])))
                .all_equal();
            if !all_corners_same {
                return false;
            }

            let mut side_len = None;
            let mut turn_angle = None;
            let mut current_side_len = 0;
            for angle in angles {
                match angle {
                    Angle::Forward => current_side_len += 1,
                    Angle::Left | Angle::Right => {
                        match side_len {
                            None => side_len = Some(current_side_len),
                            Some(real_len) => {
                                if real_len != current_side_len {
                                    return false;
                                }
                            }
                        }
                        match turn_angle {
                            None => turn_angle = Some(angle),
                            Some(real_angle) => {
                                if real_angle != angle {
                                    return false;
                                }
                            }
                        }
                        current_side_len = 0;
                    }
                    _ => return false,
                }
            }
            true
        };

        if is_hexagon() {
            BoardAction::DeleteColor(self.board.get_marble(&pat[0]).unwrap().clone())
        } else {
            // Oh well.
            // Because last == first we need to remove one of them
            // otherwise the cycle breaks
            pat.pop();
            BoardAction::Cycle(pat)
        }
    }
}

fn mouse_to_hex() -> Coordinate {
    let (mx, my) = mouse_position_pixel();
    let board_x = mx - BOARD_CENTER_X;
    let board_y = my - BOARD_CENTER_Y;

    // hex2d does not come with a function to convert back from blocky pixel coords to hex.
    // so we roll our own
    // also i could const fold all this but lazyyy
    let forward_transform = Mat2::from_cols_array(&[
        MARBLE_SPAN_X as f32,
        0.0,
        MARBLE_SPAN_X as f32 / 2.0,
        MARBLE_SPAN_Y as f32,
    ]);
    let transform = forward_transform.inverse();
    let (q, r) = (transform * vec2(board_x, board_y)).into();

    // i hate hexagons, dunno why i need all this awful rotating
    Coordinate::<i32>::nearest(r, q).rotate_around_zero(Angle::RightBack)
}

fn is_pattern_valid(
    pattern: &[Coordinate],
    board: &AHashMap<Coordinate, Marble>,
) -> PatternExtensionValidity {
    if !pattern.iter().all(|c| board.get(c).is_some()) {
        return PatternExtensionValidity::Invalid;
    }

    let len = pattern.len();
    let new = pattern[len - 1];

    // Nothing under a length of 4 can be valid
    // len of 4 for the 3+ things in the loop and one more for the final one
    if len < 4 {
        return if len == 2 && pattern[0] == pattern[1] {
            // No overlaps
            PatternExtensionValidity::Invalid
        } else if len <= 2 {
            // Length of 2 and below is too short to be bad
            // without overlaps
            PatternExtensionValidity::Continue
        } else if pattern[1..len - 1].contains(&new) {
            // An overlap is always a no-go
            // we know the len is 3 here so no OOB errors
            PatternExtensionValidity::Invalid
        } else {
            // hey it might be ok
            PatternExtensionValidity::Continue
        };
    }

    let end = pattern[len - 2];

    if end.distance(new) != 1 || pattern[1..len - 1].contains(&new) {
        // No wild leaps
        // No overlaps 'cept the first
        PatternExtensionValidity::Invalid
    } else if pattern[0] == new {
        // We've closed the loop!
        PatternExtensionValidity::Closing
    } else {
        // try some more
        PatternExtensionValidity::Continue
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternExtensionValidity {
    /// Add it on
    Continue,
    /// Don't add it on
    Invalid,
    /// We finished
    Closing,
}
