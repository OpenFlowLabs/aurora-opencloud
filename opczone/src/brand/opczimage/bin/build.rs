use clap::Parser;
use common::init_slog_logging;
use miette::{IntoDiagnostic, Result};
use opczone::brand::ZONECONTROL_NGZ_PATH;
use opczone::build::bundle::{BuildBundleKind, Bundle, BUILD_BUNDLE_IMAGE_PATH};
use opczone::build::run_action;
use std::fs::File;
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
    //let bundle_audit = bundle.get_audit_info();

    println!(
        "Building Image {} by {}",
        &bundle.document.name,
        bundle.document.author.clone().unwrap_or("Anonymous".into())
    );

    let actions = bundle.document.actions.clone();

    let zonename = zone::current_blocking().into_diagnostic()?;

    let sysconfig_path = Path::new(ZONECONTROL_NGZ_PATH).join("sysconfig.json");

    let sysconfig_file = File::open(&sysconfig_path).into_diagnostic()?;

    let set: libsysconfig::InstructionsSet =
        serde_json::from_reader(sysconfig_file).into_diagnostic()?;

    let mut img = libsysconfig::Image::new();

    // For some Reason we get problems that the network is not online fast enough
    // So we insert 1 second delay between comands to settle things.
    // TODO: expose this feature to the configudarion
    img.insert_delay(1);

    img.apply_instructions(set).into_diagnostic()?;

    for action in actions {
        run_action("/", &zonename, &bundle, action)?;
    }

    Ok(())
}

struct BuildBundleSearchResult {
    #[allow(dead_code)]
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

    miette::bail!("could not find any known kind of build bundle")
}

fn load_build_bundle(search_result: &BuildBundleSearchResult) -> Result<Bundle> {
    let bundle = Bundle::new(&search_result.path)?;
    Ok(bundle)
}
