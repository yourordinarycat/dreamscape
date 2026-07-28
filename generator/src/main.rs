mod articles;
mod diagnostics;
mod directives;
mod layouts;
mod manifest;
mod path_ext;
mod resources;

use std::{fs, path::Path};

use diagnostics::Diagnostics;
use layouts::get_layouts;
use manifest::load_manifest;

use crate::resources::href;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut diagnostics = Diagnostics::default();

    let cwd = std::env::current_dir()?;
    let base_out_dir = cwd.join("dist");
    fs::remove_dir_all(&base_out_dir)?;

    let mut out_dir = base_out_dir.clone();

    // 1. Process manifest
    let manifest = load_manifest(cwd.join("manifest.toml"))?;

    if let Some(base_path) = &manifest.base_path {
        out_dir.push(base_path);
    }

    //
    // 2. Process static resources
    //
    // Trying to recover from the initial FS operations & manifest parsing
    // isn't particularly useful as other operations depend on having a
    // manifest available at minimum. Starting here, errors will be added to
    // diagnostics instead of causing an early exit.
    //
    let resources_dir = cwd.join("resources");
    let out_resources_dir = out_dir.join("resources");

    let (resource_map, diag) =
        resources::discovery::find_all(&resources_dir, &out_resources_dir, &base_out_dir);
    diagnostics.merge(diag);

    // 3. Get articles
    let articles_dir = cwd.join("articles");
    let (articles, diag) = articles::discovery::find_all(articles_dir, &manifest);
    diagnostics.merge(diag);

    // 4. Get layouts
    let layouts_dir = cwd.join("layouts");
    let (layouts, diag) = get_layouts(layouts_dir);
    diagnostics.merge(diag);

    // 5. Process articles
    diagnostics.merge(articles::writer::write_to(
        &base_out_dir,
        &manifest,
        &articles,
        &layouts,
        &resource_map,
    ));

    if diagnostics.failed() {
        for err in diagnostics.errors {
            eprintln!("{}", err);
        }
    } else {
        let rel_out = base_out_dir.strip_prefix(cwd)?;
        let url_path = manifest.base_path.unwrap_or("/".to_string());
        let href = href::normalize(Path::new(&url_path));

        println!();
        println!("Generated blog to: {}", base_out_dir.display());
        println!("Take a look at the content by running:");
        println!();
        println!(
            "    static-web-server --port 80 --root {}",
            rel_out.display()
        );
        println!();
        println!("And visiting http://localhost{}", href);
        println!();
    }

    Ok(())
}
