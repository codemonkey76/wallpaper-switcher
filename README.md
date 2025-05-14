# 🖼️ Hyprland Wallpaper Switcher

A Rust-based wallpaper switcher for [Hyprland](https://github.com/hyprwm/Hyprland), with automatic orientation detection, rotation tracking, and blurred lock screen background generation.

---

## ✨ Features

- Detects each monitor's orientation (landscape or portrait) using `hyprctl`
- Rotates wallpapers per-orientation, one per monitor
- Avoids repeating wallpapers until the list is exhausted
- Creates blurred + darkened overlays for use with `swaylock-effects`
- Automatically stores rotation state per orientation in `~/.cache/`

---

## 📂 Expected Directory Structure

Your wallpapers should be stored in:
```bash
~/Pictures/Wallpapers/
├── nature1.jpg
├── anime_vertical.webp
└── urban_landscape.png
```
> Orientation is auto-detected via image dimensions (not by subfolder).

---

## 🔧 Requirements

Make sure the following are installed and in your `$PATH`:

- [`hyprctl`](https://wiki.hyprland.org/Configuring/Hyprctl/)
- [`imagemagick`](https://imagemagick.org/) (`magick` CLI for blur overlays)
- [`identify`](part of `imagemagick`) for resolution detection
- [`walkdir`](Rust dependency; bundled)

Optional for lock screen integration:
- `swaylock-effects`

---

## 🚀 Usage

1. **Build the binary**:

   ```bash
   cargo build --release
   ```
2. Run the app:

```bash
./target/release/wallpaper-switcher
```

It will:
- Load all wallpaper images under `~/Pictures/Wallpapers`
- Assign images to monitors based on orientation
- Cycle through images and remember which one was used last
- Preload + apply wallpapers via `hyprctl hyprpaper`
- Create a blurred/darkened image per monitor in `/tmp/hyprlock/<monitor>.png`

3. (Optional) Add a keybind to your Hyprland config:
```bash
bind = $mod, W, exec, ~/dev/wallpaper-switcher/target/release/wallpaper-switcher
```

## 🔐 Lock Screen Integration

To use the blurred wallpapers with `swaylock-effects`, configure it like:
```bash
swaylock -i /tmp/hyprlock/DP-1.png
```
Each monitor will have a unique file based on its name.

## 💾 Caching & State
Rotation index is stored in:

```bash
~/.cache/hyprpaper_index_landscape
~/.cache/hyprpaper_index_portrait
```

These track your progress through each list and reset automatically when exhausted.

## 📜 License
MIT
