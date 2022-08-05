use miette::{Context, IntoDiagnostic};
use opcimage::definition::Document;
use std::fs;

fn main() -> miette::Result<()> {
    let file = "testdata/image_base.kdl";

    let text = fs::read_to_string(file)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot read {:?}", file))?;

    let _config = match knuffel::parse::<Document>(file, &text) {
        Ok(config) => config,
        Err(e) => {
            println!("{:?}", miette::Report::new(e));
            std::process::exit(1);
        }
    };

    Ok(())
}
