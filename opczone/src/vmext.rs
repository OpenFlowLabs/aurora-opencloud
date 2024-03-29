use crate::machine::OnDiskPayload;
use miette::Diagnostic;
use std::fs::File;
use std::path::Path;

pub const ZONE_ETC_DIR: &str = "/etc/zones";

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum VMExtError {
    #[error(transparent)]
    JSONError(#[from] serde_json::Error),
    #[error("{0}: {1}")]
    IOError(String, #[source] std::io::Error),
}

type Result<T> = miette::Result<T, VMExtError>;

pub fn get_brand_config(zonename: &str) -> Result<OnDiskPayload> {
    let path = Path::new(ZONE_ETC_DIR).join(format!("{}_vmext.json", zonename));
    let file = File::open(&path).map_err(|e| {
        VMExtError::IOError(format!("can not read brand config {}", &path.display()), e)
    })?;

    Ok(serde_json::from_reader(file)?)
}

pub fn write_brand_config(cfg: &OnDiskPayload) -> Result<()> {
    let path = Path::new(ZONE_ETC_DIR).join(format!("{}_vmext.json", &cfg.uuid.to_string()));
    let mut file = File::create(&path).map_err(|e| {
        VMExtError::IOError(
            format!("can not open brand config {} for writing", &path.display()),
            e,
        )
    })?;

    Ok(serde_json::to_writer(&mut file, &cfg)?)
}

pub fn get_brand_default_config() -> OnDiskPayload {
    OnDiskPayload::default()
}
