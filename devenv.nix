{ pkgs, ... }:

{
  packages = [
    pkgs.bacon
    pkgs.cargo-machete
    pkgs.cargo-edit
    pkgs.cargo-deny
    pkgs.cargo-apk
    pkgs.udev pkgs.alsa-lib pkgs.vulkan-loader
    pkgs.libX11 pkgs.libXcursor pkgs.libXi pkgs.libXrandr # To use the x11 feature
    pkgs.libxkbcommon pkgs.wayland 
  ];
  
  languages.rust = {
    enable = true;
    channel = "stable";
    mold.enable = true;
    targets = [
      "aarch64-linux-android"
      "armv7-linux-androideabi"
      "wasm32-unknown-unknown"
      "aarch64-unknown-linux-gnu"
    ];
  };

  enterShell = ''
    export LD_LIBRARY_PATH=${pkgs.vulkan-loader}/lib:${pkgs.libxkbcommon}/lib:$LD_LIBRARY_PATH
  '';

  android = {
    enable = true;
    platforms.version = [ "30" "35" ];
  };
}
