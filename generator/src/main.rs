mod resources;
mod manifest;

use resources::create_resource_map;
use manifest::load_manifest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let base_out_dir = cwd.join("dist");
    let mut out_dir = base_out_dir.clone();

    // 1. Process manifest
    let manifest = load_manifest(cwd.join("manifest.json"))?;
    println!("Manifest: {:#?}", manifest);

    if let Some(base_path) = manifest.base_path {
        out_dir.push(base_path);
    }

    // 2. Process static resources
    let resources_dir = cwd.join("resources");
    let out_resources_dir = out_dir.join("resources");

    println!("Resources directory: {}", out_resources_dir.to_str().unwrap().to_owned());

    // 2.1. Copy directory
    let resource_map = create_resource_map(
        &resources_dir,
        &out_resources_dir,
        &base_out_dir)?;
    println!("Resources: {:#?}", resource_map);

    Ok(())
}
