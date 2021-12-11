use cogs_gamedev::ease::Interpolator;
use hex2d::{Coordinate, IntegerSpacing};
use macroquad::prelude::*;

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, GamemodeDrawer},
    model::{BoardAction, Marble},
    utils::{
        draw::{hexcolor, mouse_position_pixel},
        text::{Billboard, Markup, TextSpan},
    },
    HEIGHT, WIDTH,
};

use super::{
    PlaySettings, BOARD_CENTER_X, BOARD_CENTER_Y, MARBLE_SIZE, MARBLE_SPAN_X, MARBLE_SPAN_Y,
};

/// Speed for one on or off of the blink
const CLEAR_ALL_BLINK_SPEED: u32 = 10;
/// How many bg timer points to one hexagon
const BG_HEX_SPEED: u32 = 20;
/// How many hexagons there are
const BG_HEX_COUNT: u32 = 6;

pub struct Drawer {
    pub marbles: Vec<(Coordinate, Marble)>,
    pub pattern: Option<Vec<Coordinate>>,

    /// All the coordinates of marbles in blobs big enough to be removed,
    /// if next on the agenda is to clear blobs (otherwise it will be empty)
    pub to_remove: Vec<Coordinate>,
    pub radius: usize,
    pub next_spawn_point: Option<Coordinate>,
    /// The action we're about to do and time ticking up until it's completed
    pub next_action: Option<(BoardAction, u32)>,

    pub bg_funni_timer: f32,
    pub score: u32,

    pub paused: bool,

    pub settings: PlaySettings,
}

impl GamemodeDrawer for Drawer {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo) {
        clear_background(hexcolor(0x14182e_ff));

        if self.settings.funni_background {
            for hex_idx in (0..BG_HEX_COUNT).rev() {
                let radius = (hex_idx as f32 + (self.bg_funni_timer / BG_HEX_SPEED as f32).fract())
                    * WIDTH
                    / BG_HEX_COUNT as f32
                    * 1.1;
                let color = if (self.bg_funni_timer.trunc() as u32 / BG_HEX_SPEED + hex_idx)
                    % BG_HEX_COUNT
                    % 2
                    == 0
                {
                    hexcolor(0x14182e_ff)
                } else {
                    hexcolor(0x4b1d52_ff)
                };

                draw_hexagon(
                    BOARD_CENTER_X,
                    BOARD_CENTER_Y,
                    radius,
                    2.0,
                    false,
                    hexcolor(0xcc2f7b_ff),
                    color,
                );
            }
        }

        for bg_pos in Coordinate::new(0, 0).range_iter(self.radius as _) {
            let (ox, oy) =
                bg_pos.to_pixel_integer(IntegerSpacing::PointyTop(MARBLE_SPAN_X, MARBLE_SPAN_Y));

            let corner_x = ox as f32 - MARBLE_SIZE / 2.0 + BOARD_CENTER_X;
            let corner_y = oy as f32 - MARBLE_SIZE / 2.0 + BOARD_CENTER_Y;

            let (sx, color) = if self.next_spawn_point == Some(bg_pos) {
                (1, hexcolor(0xff4538_a0))
            } else {
                (0, hexcolor(0xdfe0e8_a0))
            };

            draw_texture_ex(
                assets.textures.marble_atlas,
                corner_x,
                corner_y,
                color,
                DrawTextureParams {
                    source: Some(Rect::new(
                        sx as f32 * MARBLE_SIZE,
                        2.0 * MARBLE_SIZE,
                        MARBLE_SIZE,
                        MARBLE_SIZE,
                    )),
                    ..Default::default()
                },
            );
        }

        for (pos, marble) in self.marbles.iter() {
            let dark = hexcolor(0x291d2b_ff);
            let sigil_color = match &self.next_action {
                Some((BoardAction::ClearBlobs(_), _)) if self.to_remove.contains(pos) => WHITE,
                Some((BoardAction::DeleteColor(col), timer)) if col == marble => {
                    if *timer / CLEAR_ALL_BLINK_SPEED % 2 == 0 {
                        hexcolor(0xffee83_ff)
                    } else {
                        WHITE
                    }
                }
                _ => dark,
            };

            let (corner_x, corner_y) = match &self.next_action {
                Some((BoardAction::Cycle(path), timer)) if path.contains(pos) => {
                    let idx = path
                        .iter()
                        .enumerate()
                        .find_map(
                            |(idx, pathpos)| {
                                if pathpos == pos {
                                    Some(idx)
                                } else {
                                    None
                                }
                            },
                        )
                        .unwrap();
                    let next = path[(idx + 1) % path.len()];

                    let start = pos_to_marble_corner(*pos);
                    let start = [start.0, start.1];
                    let end = pos_to_marble_corner(next);
                    let end = [end.0, end.1];

                    let t = *timer as f32 / BoardAction::CYCLE_TIME as f32;
                    let middle = Interpolator::lerp(t, start, end);
                    (middle[0].round(), middle[1].round())
                }
                _ => pos_to_marble_corner(*pos),
            };

            let sx = marble.clone() as u32 as f32 * MARBLE_SIZE;
            draw_texture_ex(
                assets.textures.marble_atlas,
                corner_x,
                corner_y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(sx, 8.0, MARBLE_SIZE, MARBLE_SIZE)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                assets.textures.marble_atlas,
                corner_x,
                corner_y,
                sigil_color,
                DrawTextureParams {
                    source: Some(Rect::new(sx, 0.0, MARBLE_SIZE, MARBLE_SIZE)),
                    ..Default::default()
                },
            );
        }

        if let Some(pat) = &self.pattern {
            draw_pattern(pat, WHITE, assets);
        }

        let text = format!("{}", self.score * 100);
        let text_x = BOARD_CENTER_X - 5.0 * (text.len() as f32 - 1.0) / 2.0;
        let text_y = BOARD_CENTER_Y - (self.radius as i32 * MARBLE_SPAN_Y) as f32;
        Billboard::draw_now(
            vec![TextSpan {
                text,
                markup: Markup {
                    color: WHITE,
                    font: assets.textures.fonts.small,
                    kerning: 1.0,
                    vert_space: 1.0,
                    wave: None,
                },
            }],
            vec2(text_x.round(), text_y.round()),
            vec2(0.0, -5.0),
            None,
        );

        if self.paused {
            draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, hexcolor(0x291d2b_a0));

            Billboard::draw_now(
                vec![TextSpan {
                    text: "PAUSED".to_owned(),
                    markup: Markup {
                        color: WHITE,
                        font: assets.textures.fonts.small,
                        kerning: 1.0,
                        vert_space: 1.0,
                        wave: None,
                    },
                }],
                vec2(WIDTH / 2.0 - 10.0, HEIGHT / 2.0),
                vec2(0.0, -5.0),
                None,
            );
        }
    }
}

/// give the corner x/y poses of the marble at the given position
fn pos_to_marble_corner(pos: Coordinate) -> (f32, f32) {
    let (ox, oy) = pos.to_pixel_integer(IntegerSpacing::PointyTop(MARBLE_SPAN_X, MARBLE_SPAN_Y));
    let corner_x = ox as f32 - MARBLE_SIZE / 2.0 + BOARD_CENTER_X;
    let corner_y = oy as f32 - MARBLE_SIZE / 2.0 + BOARD_CENTER_Y;
    (corner_x, corner_y)
}

fn draw_pattern(pat: &[Coordinate], color: Color, assets: &Assets) {
    gl_use_material(assets.shaders.pattern_beam);

    for span in pat.windows(2) {
        let (x1, y1) = pos_to_marble_corner(span[0]);
        let (x2, y2) = pos_to_marble_corner(span[1]);

        draw_line_but_with_uvs(
            x1 + MARBLE_SIZE / 2.0,
            y1 + MARBLE_SIZE / 2.0,
            x2 + MARBLE_SIZE / 2.0,
            y2 + MARBLE_SIZE / 2.0,
            1.0,
            color,
        );
    }

    let (x1, y1) = pos_to_marble_corner(*pat.last().unwrap());
    let (x2, y2) = mouse_position_pixel();
    draw_line_but_with_uvs(
        x1 + MARBLE_SIZE / 2.0,
        y1 + MARBLE_SIZE / 2.0,
        x2,
        y2,
        1.0,
        color,
    );

    gl_use_default_material();
}

pub fn draw_line_but_with_uvs(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    let context = unsafe { get_internal_gl() };
    let dx = x2 - x1;
    let dy = y2 - y1;

    // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment

    let nx = -dy;
    let ny = dx;

    let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
    if tlen < std::f32::EPSILON {
        return;
    }
    let tx = nx / tlen;
    let ty = ny / tlen;

    context.quad_gl.texture(None);
    context.quad_gl.draw_mode(DrawMode::Triangles);
    context.quad_gl.geometry(
        &[
            Vertex::new(x1 + tx, y1 + ty, 0., 0., 0., color),
            Vertex::new(x1 - tx, y1 - ty, 0., 0., 0., color),
            Vertex::new(x2 + tx, y2 + ty, 0., 1., 0., color),
            Vertex::new(x2 - tx, y2 - ty, 0., 1., 0., color),
        ],
        &[0, 1, 2, 2, 1, 3],
    );
}
