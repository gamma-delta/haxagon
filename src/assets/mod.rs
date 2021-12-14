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
    pub end_jingle: Sound,

    pub music0: Sound,
    pub music1: Sound,
    pub music2: Sound,

    pub select: Sound,
    pub close_loop: Sound,
    pub shunt: Sound,
    pub clear1: Sound,
    pub clear2: Sound,
    pub clear3: Sound,
    pub clear4: Sound,
    pub clear5: Sound,
    pub clear_all: Sound,
}

impl Sounds {
    async fn init() -> Self {
        Self {
            splash_jingle: sound("splash/jingle").await,

            title_music: sound("music/title").await,
            end_jingle: sound("music/ending").await,

            music0: sound("music/music0").await,
            music1: sound("music/music1").await,
            music2: sound("music/music2").await,

            select: sound("sfx/select").await,
            close_loop: sound("sfx/close_loop").await,
            shunt: sound("sfx/shunt").await,
            clear1: sound("sfx/clear1").await,
            clear2: sound("sfx/clear2").await,
            clear3: sound("sfx/clear3").await,
            clear4: sound("sfx/clear4").await,
            clear5: sound("sfx/clear5").await,
            clear_all: sound("sfx/clear_all").await,
        }
    }
}

pub struct Shaders {
    pub pattern_beam: Material,
    pub noise: Material,
}

impl Shaders {
    async fn init() -> Self {
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
