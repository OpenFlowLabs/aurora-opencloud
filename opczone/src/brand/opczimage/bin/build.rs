use anyhow::{anyhow, bail, Result};
use clap::Parser;
use common::{init_slog_logging, info};
use opczone::build::bundle::{BuildBundleKind, Bundle, BUILD_BUNDLE_IMAGE_PATH};
use opczone::build::run_action;
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Cli {}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false, true)?;

    let _cli: Cli = Cli::parse();

    // First try find the bundle we have
    let build_bundle = search_build_bundle()?;

    // Load build Instructions
    let bundle = load_build_bundle(&build_bundle)?;
    let bundle_audit = bundle.get_audit_info();

    println!("Building Image {} by {}", &bundle.document.name, bundle.document.author.clone().unwrap_or("Anonymous".into()));

    let actions = if bundle_audit.is_base_image() {
        bundle.document.actions.clone()[1..].to_vec()
    } else {
        bundle.document.actions.clone()
    };

    let zonename = zone::current()?;

    for action in actions {
        run_action("/", &zonename, &bundle, action)?;
    }

    Ok(())
}

struct BuildBundleSearchResult {
    kind: BuildBundleKind,
    path: PathBuf,
}

fn search_build_bundle() -> Result<BuildBundleSearchResult> {
    // First try to find the directory kind
    let bundle_dir_path = Path::new(BUILD_BUNDLE_IMAGE_PATH);
    if bundle_dir_path.exists() {
        return Ok(BuildBundleSearchResult {
            kind: BuildBundleKind::Directory,
            path: bundle_dir_path.to_path_buf(),
        });
    }

    bail!("could not find any known kind of build bundle")
}

fn load_build_bundle(search_result: &BuildBundleSearchResult) -> Result<Bundle> {
    let bundle = Bundle::new(&search_result.path).map_err(|err| anyhow!("{:?}", err))?;
    Ok(bundle)
}
