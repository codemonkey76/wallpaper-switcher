use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct Monitor {
    name: String,
    transform: u8,
}

fn main() {
    let output = Command::new("hyprctl")
        .arg("monitors")
        .arg("-j")
        .output()
        .expect("Failed to run hyprctl");

    let json = String::from_utf8_lossy(&output.stdout);
    let monitors: Vec<Monitor> = serde_json::from_str(&json).expect("Failed to parse hyprctl JSON");

    let (portrait_images, landscape_images) = get_wallpapers_by_orientation(&format!(
        "{}/Pictures/Wallpapers",
        std::env::var("HOME").unwrap()
    ));

    let landscape_state_path = format!(
        "{}/.cache/hyprpaper_index_landscape",
        std::env::var("HOME").unwrap()
    );
    let portrait_state_path = format!(
        "{}/.cache/hyprpaper_index_portrait",
        std::env::var("HOME").unwrap()
    );

    let mut landscape_index = load_index(&landscape_state_path);
    let mut portrait_index = load_index(&portrait_state_path);

    for m in monitors {
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
            println!("Setting wallpaper for {}: {}", m.name, wallpaper.display());

            let _ = Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("preload")
                .arg(wallpaper)
                .status();
            let _ = Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("wallpaper")
                .arg(format!("{},{}", m.name, wallpaper.display()))
                .status();
        } else {
            println!("No suitable wallpaper found for monitor {}", m.name);
        }
    }

    save_index(&landscape_state_path, landscape_index);
    save_index(&portrait_state_path, portrait_index);
}

fn get_image_orientation(path: &PathBuf) -> Option<&'static str> {
    let output = Command::new("identify")
        .arg("-format")
        .arg("%w %h")
        .arg(path)
        .output()
        .ok()?;

    let out_str = String::from_utf8_lossy(&output.stdout);
    let mut parts = out_str.split_whitespace();
    let width: u32 = parts.next()?.parse().ok()?;
    let height: u32 = parts.next()?.parse().ok()?;

    Some(if width >= height {
        "landscape"
    } else {
        "portrait"
    })
}

fn get_wallpapers_by_orientation(dir: &str) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let re = Regex::new(r"(?i)\.(jpe?g|png|webp|jxl)$").unwrap();
    let mut portrait = Vec::new();
    let mut landscape = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.into_path();
        println!("Found file: {}", path.display());
        if !re.is_match(path.to_string_lossy().as_ref()) {
            continue;
        }

        match get_image_orientation(&path) {
            Some("portrait") => portrait.push(path),
            Some("landscape") => landscape.push(path),
            _ => (),
        }
    }

    (portrait, landscape)
}

fn load_index(path: &str) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_index(path: &str, index: usize) {
    let _ = fs::write(path, index.to_string());
}
