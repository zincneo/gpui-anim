{
  description = "gpui-anim flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        guiLibs = with pkgs;[
          # misc. libraries
          openssl
          pkg-config
          glib
          pango
          atkmm
          gdk-pixbuf
          gtk3
          libsoup_3
          webkitgtk_4_1
          # GUI libs
          libxkbcommon
          libGL
          fontconfig
          # wayland libraries
          wayland
          # x11 libraries
          libxcursor
          libxrandr
          libxi
          libx11
          libxcb
          # vulkan
          vulkan-loader
        ];
      in
      {
        devShells.default = pkgs.mkShell rec {
          buildInputs = guiLibs;
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
        };
      });
}
