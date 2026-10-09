{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    gcc
    pkg-config
    udev
    alsa-lib
    vulkan-loader
    libx11
    libxcursor
    libxi
    libxrandr
    libxkbcommon
    wayland
  ];

  # Bevy uses dlopen to load dynamic libraries at runtime.
  # On NixOS, we need to explicitly set LD_LIBRARY_PATH so that the binary can find them.
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
    udev
    alsa-lib
    vulkan-loader
    libx11
    libxcursor
    libxi
    libxrandr
    libxkbcommon
    wayland
  ]);
}
