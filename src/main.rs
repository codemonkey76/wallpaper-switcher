mod blur;
mod cache;
mod classify;

use blur::generate_blur_if_needed;
use cache::{WallpaperMeta, load_metadata_cache, save_metadata_cache};
use classify::classify_wallpapers;

use blake3::hash;
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct Monitor {
    name: String,
    transform: u8,
}

fn main() {
    let total_start = Instant::now();

    // Get monitor info
    let start = Instant::now();
    let output = Command::new("hyprctl")
        .arg("monitors")
        .arg("-j")
        .output()
        .expect("Failed to run hyprctl");
    println!("Time to run hyprctl: {:.2?}", start.elapsed());

    let json = String::from_utf8_lossy(&output.stdout);
    let monitors: Vec<Monitor> = serde_json::from_str(&json).expect("Failed to parse hyprctl JSON");

    // Paths
    let home = env::var("HOME").unwrap();
    let wp_dir = format!("{}/Pictures/Wallpapers", home);
    let meta_path = format!("{}/.cache/wallpaper_meta.json", home);

    let landscape_index_path = format!("{}/.cache/hyprpaper_index_landscape", home);
    let portrait_index_path = format!("{}/.cache/hyprpaper_index_portrait", home);

    let mut landscape_index = cache::load_index(&landscape_index_path);
    let mut portrait_index = cache::load_index(&portrait_index_path);

    // Load metadata and scan files
    let start = Instant::now();
    let mut metadata: HashMap<String, WallpaperMeta> = load_metadata_cache(&meta_path);
    println!("Time to load metadata: {:.2?}", start.elapsed());

    let start = Instant::now();
    let re = Regex::new(r"(?i)\.(jpe?g|png|webp|jxl)$").unwrap();
    let (portrait_images, landscape_images) = classify_wallpapers(&wp_dir, &re, &mut metadata);
    println!("Time to classify wallpapers: {:.2?}", start.elapsed());

    portrait_images
        .iter()
        .chain(landscape_images.iter())
        .collect::<Vec<_>>()
        .par_iter()
        .for_each(|path| {
            let out_path = format!(
                "/tmp/hyprlock-cache/{}.png",
                blake3::hash(path.to_string_lossy().as_bytes())
            );
            generate_blur_if_needed(path, Path::new(&out_path));
        });

    let start = Instant::now();
    save_metadata_cache(&meta_path, &metadata);
    println!("Time to save metadata: {:.2?}", start.elapsed());

    for m in monitors {
        println!("Processing monitor: {}", m.name);

        let orientation = if m.transform % 2 == 0 {
            "landscape"
        } else {
            "portrait"
        };

        let selected = match orientation {
            "landscape" if !landscape_images.is_empty() => {
                let wp = &landscape_images[landscape_index % landscape_images.len()];
                landscape_index += 1;
                Some(wp)
            }
            "portrait" if !portrait_images.is_empty() => {
                let wp = &portrait_images[portrait_index % portrait_images.len()];
                portrait_index += 1;
                Some(wp)
            }
            _ => None,
        };

        if let Some(wallpaper) = selected {
            let preload_start = Instant::now();
            println!("Setting wallpaper for {}: {}", m.name, wallpaper.display());

            let _ = Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("preload")
                .arg(wallpaper)
                .status();
            println!("  Preload time: {:.2?}", preload_start.elapsed());

            let set_start = Instant::now();
            let _ = Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("wallpaper")
                .arg(format!("{},{}", m.name, wallpaper.display()))
                .status();
            println!("  Set wallpaper time: {:.2?}", set_start.elapsed());

            let blur_start = Instant::now();
            let darkened_path = format!(
                "/tmp/hyprlock-cache/{}.png",
                hash(wallpaper.to_string_lossy().as_bytes())
            );

            generate_blur_if_needed(wallpaper, Path::new(&darkened_path));
            println!("  Blur generation time: {:.2?}", blur_start.elapsed());
        } else {
            println!("No suitable wallpaper found for monitor {}", m.name);
        }
    }

    cache::save_index(&landscape_index_path, landscape_index);
    cache::save_index(&portrait_index_path, portrait_index);
    println!("Total runtime: {:.2?}", total_start.elapsed());
}
