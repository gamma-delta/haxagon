#![allow(clippy::eval_order_dependence)]

use macroquad::{
    audio::{load_sound, Sound},
    miniquad::*,
    prelude::*,
};
use once_cell::sync::Lazy;

use std::path::PathBuf;

pub struct Assets {
    pub textures: Textures,
    pub sounds: Sounds,
    pub shaders: Shaders,
}

impl Assets {
    pub async fn init() -> Self {
        Self {
            textures: Textures::init().await,
            sounds: Sounds::init().await,
            shaders: Shaders::init().await,
        }
    }
}

pub struct Textures {
    pub fonts: Fonts,

    pub title_banner: Texture2D,
    pub billboard_patch9: Texture2D,

    pub title_logo: Texture2D,
    pub title_stencil: Texture2D,
    pub marble_atlas: Texture2D,
}

impl Textures {
    async fn init() -> Self {
        Self {
            fonts: Fonts::init().await,
            title_banner: texture("splash/banner").await,
            billboard_patch9: texture("ui/billboard_patch9").await,
            title_logo: texture("splash").await,
            title_stencil: texture("splash_stencil").await,
            marble_atlas: texture("marbles").await,
        }
    }
}

pub struct Fonts {
    pub small: Texture2D,
    pub medium: Texture2D,
}

impl Fonts {
    async fn init() -> Self {
        Self {
            small: texture("ui/font_small").await,
            medium: texture("ui/font_medium").await,
        }
    }
}

pub struct Sounds {
    pub splash_jingle: Sound,
    pub title_music: Sound,
    pub haxagon: Sound,
    pub end_jingle: Sound,

    pub select: Sound,
    pub shunt: Sound,
    pub clear1: Sound,
    pub clear2: Sound,
    pub clear3: Sound,
    pub clear_all: Sound,
}

impl Sounds {
    async fn init() -> Self {
        Self {
            splash_jingle: sound("splash/jingle").await,
            title_music: sound("title").await,
            haxagon: sound("haxagon").await,
            end_jingle: sound("ending").await,

            select: sound("select").await,
            shunt: sound("shunt").await,
            clear1: sound("clear1").await,
            clear2: sound("clear2").await,
            clear3: sound("clear3").await,
            clear_all: sound("clear_all").await,
        }
    }
}

pub struct Shaders {
    pub pattern_beam: Material,
    pub noise: Material,

    pub stencil_write: Material,
    pub stencil_mask: Material,
}

impl Shaders {
    async fn init() -> Self {
        // society if this implemented Default
        let ss_write = StencilFaceState {
            fail_op: StencilOp::Replace,
            depth_fail_op: StencilOp::Replace,
            pass_op: StencilOp::Replace,
            test_func: CompareFunc::Always,
            test_ref: 0,
            test_mask: 0xffffffff,
            write_mask: 0xffffffff,
        };
        let ss_mask = StencilFaceState {
            fail_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            // The equation is: (test_ref & mask) OP (stencil_val & mask)
            // LHS will be 0, so we allow the pixel through iff the stencil has any
            // non-0 value drawn to it.
            test_func: CompareFunc::Never,
            test_ref: 0,
            test_mask: 0xffffffff,
            write_mask: 0xffffffff,
        };

        Self {
            pattern_beam: material_vert_frag(
                "standard",
                "pattern_beam",
                MaterialParams {
                    textures: Vec::new(),
                    uniforms: Vec::new(),
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Value(BlendValue::SourceAlpha),
                            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        )),
                        ..Default::default()
                    },
                },
            )
            .await,
            noise: material_vert_frag(
                "standard",
                "noise",
                MaterialParams {
                    textures: Vec::new(),
                    uniforms: Vec::new(),
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Value(BlendValue::SourceAlpha),
                            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        )),
                        ..Default::default()
                    },
                },
            )
            .await,
            stencil_write: material(
                "standard",
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_write: (false, false, false, false),
                        stencil_test: Some(StencilState {
                            back: ss_write,
                            front: ss_write,
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await,
            stencil_mask: material(
                "standard",
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Value(BlendValue::SourceAlpha),
                            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        )),
                        stencil_test: Some(StencilState {
                            back: ss_mask,
                            front: ss_mask,
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await,
        }
    }
}

/// Path to the assets root
static ASSETS_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_arch = "wasm32") {
        PathBuf::from("./assets")
    } else if cfg!(debug_assertions) {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"))
    } else {
        todo!("assets path for release hasn't been finalized yet ;-;")
    }
});

async fn texture(path: &str) -> Texture2D {
    let with_extension = path.to_owned() + ".png";
    let tex = load_texture(
        ASSETS_ROOT
            .join("textures")
            .join(with_extension)
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap();
    tex.set_filter(FilterMode::Nearest);
    tex
}

async fn sound(path: &str) -> Sound {
    let with_extension = path.to_owned() + ".ogg";
    load_sound(
        ASSETS_ROOT
            .join("sounds")
            .join(with_extension)
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap()
}

async fn material_vert_frag(vert_stub: &str, frag_stub: &str, params: MaterialParams) -> Material {
    let full_stub = ASSETS_ROOT.join("shaders");
    let vert = load_string(
        full_stub
            .join(vert_stub)
            .with_extension("vert")
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap();
    let frag = load_string(
        full_stub
            .join(frag_stub)
            .with_extension("frag")
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap();
    load_material(&vert, &frag, params).unwrap()
}

async fn material(path_stub: &str, params: MaterialParams) -> Material {
    material_vert_frag(path_stub, path_stub, params).await
}
