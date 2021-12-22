use std::collections::HashMap;

use macroquad::prelude::warn;
use quad_wasmnastics::storage::{self, Location};
use serde::{Deserialize, Serialize};

use crate::model::{BoardSettingsModeKey, PlaySettings};

const SERIALIZATION_VERSION: &str = "1";

/// Profile information. The `get` function loads it from storage; on drop it saves it back.
#[derive(Serialize, Deserialize, Default)]
pub struct Profile {
    #[serde(default)]
    pub highscores: HashMap<BoardSettingsModeKey, u32>,
    #[serde(default)]
    pub settings: PlaySettings,
}

impl Profile {
    pub fn get() -> Profile {
        let maybe_profile: anyhow::Result<Profile> = (|| {
            // note we save the raw bincode! it's already gzipped!
            // if we gzipped it here it would jut be gzipped twice
            let data = storage::load_from(&Location {
                version: String::from(SERIALIZATION_VERSION),
                ..Default::default()
            })?;
            let profile = bincode::deserialize(&data)?;
            Ok(profile)
        })();
        match maybe_profile {
            Ok(it) => it,
            Err(oh_no) => {
                warn!("Couldn't load profile! Loading default...\n{:?}", oh_no);
                Profile::default()
            }
        }
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        let res: anyhow::Result<()> = (|| {
            let data = bincode::serialize(self)?;
            storage::save_to(
                &data,
                &Location {
                    version: String::from(SERIALIZATION_VERSION),
                    ..Default::default()
                },
            )?;
            Ok(())
        })();
        if let Err(oh_no) = res {
            warn!("Couldn't save profile!\n{:?}", oh_no);
        }
    }
}
