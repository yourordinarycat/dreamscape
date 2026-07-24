mod articles;
mod directives;
mod layouts;
mod manifest;
mod resources;

use std::fs;

use layouts::get_layouts;
use manifest::load_manifest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let base_out_dir = cwd.join("dist");
    fs::remove_dir_all(&base_out_dir)?;

    let mut out_dir = base_out_dir.clone();

    // 1. Process manifest
    let manifest = load_manifest(cwd.join("manifest.json"))?;

    if let Some(base_path) = &manifest.base_path {
        out_dir.push(base_path);
    }

    // 2. Process static resources
    let resources_dir = cwd.join("resources");
    let out_resources_dir = out_dir.join("resources");

    let resource_map =
        resources::discovery::find_all(&resources_dir, &out_resources_dir, &base_out_dir)?;

    // 3. Get articles
    let articles_dir = cwd.join("articles");
    let articles = articles::discovery::find_all(articles_dir, &manifest)?;

    // 4. Get layouts
    let layouts_dir = cwd.join("layouts");
    let layouts = get_layouts(layouts_dir)?;

    // 5. Process articles
    articles::writer::write_to(out_dir, &manifest, &articles, &layouts, &resource_map)?;

    Ok(())
}
