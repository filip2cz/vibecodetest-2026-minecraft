# Vibecraft (Minecraft Clone in Rust)

A simple Minecraft clone built in Rust using the modern **Bevy** game engine (version 0.19.1).

*(For Czech instructions, see [README_CZ.md](README_CZ.md))*

## Features
- Optimized voxel terrain rendering (Greedy Meshing)
- Physics and collisions (proper Minecraft player hitbox size and jumping)
- Continuous block mining and placing by holding the mouse buttons
- Visual block targeting (wireframe highlight)
- Various block types (grass, dirt, stone, wood) and semi-transparent glass
- User interface (Hotbar) with scroll-wheel block selection

---

## 🚀 How to run (General Instructions)

If you are using a standard Linux distribution (Ubuntu, Arch, Fedora, etc.), macOS, or Windows, follow these steps:

1. **Install Rust** (preferably via [rustup.rs](https://rustup.rs/)).
2. Make sure you have the required system dependencies for the Bevy engine installed (if you are on Linux):
   - Alsa (for audio)
   - Udev
   - Wayland / X11 and Vulkan
   - C/C++ compiler and pkg-config
3. Open a terminal in the project directory.
4. Compile and run the game using:
   ```bash
   cargo run --release
   ```
*(Note: The first compilation might take a while as it downloads and compiles the Bevy engine. The `--release` flag ensures maximum performance.)*

---

## ❄️ How to run on NixOS

NixOS doesn't handle dynamic libraries the traditional way, so a standard `cargo run` on a clean system will likely fail to find windowing libraries (like `libxkbcommon.so`).

However, this project includes a `shell.nix` file that automatically solves this for you!

1. Open a terminal in the project directory.
2. Launch the game environment by running:
   ```bash
   nix-shell --run "cargo run --release"
   ```
This command temporarily installs all dependencies, sets the proper paths to graphical libraries (`LD_LIBRARY_PATH`), and boots up the game. No manual configuration required!

---

## 🎮 Controls

- **W, A, S, D:** Movement
- **Space:** Jump
- **Mouse Movement:** Look around
- **Left Mouse Button:** Clicking locks the cursor to the window (required for looking around). Holding it mines/destroys blocks.
- **Right Mouse Button:** Places the currently selected block (can be held down).
- **Mouse Scroll Wheel:** Selects a block type in your Hotbar.
- **Numbers 1-5:** Quick select specific blocks.
- **Esc (Escape):** Unlocks and shows the mouse cursor (e.g., to close the game).
