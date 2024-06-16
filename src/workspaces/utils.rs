use anyhow::anyhow;
use std::env;
use std::path::PathBuf;

lazy_static::lazy_static! {
    pub static ref XDG_RUNTIME_DIR: String = match env::var_os("XDG_RUNTIME_DIR")
            .ok_or(anyhow!(
                "Failed to get XDG_RUNTIME_DIR environment variable"
            )) {
                Ok(var) => match var.to_str() {
                    Some(val) => val.into(),
                    None => {
                        log::warn!("Failed to read XDG_RUNTIME_DIR");
                        "".into()
                    }
                },
                Err(err) => {
                    log::warn!("Failed to get XDG_RUNTIME_DIR. error={err}");
                    "".into()
                }
            };

    pub static ref HIS: String =
        match env::var_os("HYPRLAND_INSTANCE_SIGNATURE")
        .ok_or(anyhow!(
            "Failed to get HYPRLAND_INSTANCE_SIGNATURE environment variable"
        )) {
            Ok(var) => match var.to_str() {
                Some(val) => val.into(),
                None => {
                    log::warn!("Failed to read HYPRLAND_INSTANCE_SIGNATURE");
                    "".into()
                }
            },
            Err(err) => {
                log::warn!("Failed to get HYPRLAND_INSTANCE_SIGNATURE. error={err}");
                "".into()
            }
        };

    pub static ref HYPR_DIR: PathBuf =
        format!("{}/hypr/{}/", *XDG_RUNTIME_DIR, *HIS).into();
}
