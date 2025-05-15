use crate::cache::get_mtime;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn generate_blur_if_needed(original: &Path, output: &Path) {
    let orig_mtime = match get_mtime(original) {
        Some(t) => t,
        None => {
            println!("Could not get mtime for original: {}", original.display());
            return;
        }
    };

    if output.exists() {
        let cached_mtime = match get_mtime(output) {
            Some(t) => t,
            None => {
                println!("Could not get mtime for cached: {}", output.display());
                0
            }
        };

        if cached_mtime >= orig_mtime {
            println!("Skipping already cached: {}", output.display());
            return;
        } else {
            println!("Cached version is stale: {}", output.display());
        }
    } else {
        println!("No cached version found for: {}", original.display());

        if let Some(parent) = output.parent() {
            let _ = fs::create_dir_all(parent);
        }
    }

    println!("Regenerating blur for: {}", original.display());

    let _ = Command::new("magick")
        .arg(original)
        .args(["-blur", "0x10"])
        .args(["-fill", "rgba(0,0,0,0.8)"])
        .args(["-draw", "rectangle 0,0 10000,10000"])
        .arg(output)
        .status();
}
