{
  description = "Rust devshell for Shuftle";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
    nixgl.url = "github:guibou/nixGL";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            # The Android SDK is unfree and needs its license accepted.
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" "aarch64-linux-android" ];
        };
        # The SDK versions the Gradle build expects, from its version catalog.
        androidVersions = (builtins.fromTOML (builtins.readFile ./android/gradle/libs.versions.toml)).versions;
        androidBuildTools = androidVersions.androidBuildTools;
        androidNdk = androidVersions.androidNdk;
        androidSdk = (pkgs.androidenv.composeAndroidPackages {
          platformVersions = [ androidVersions.androidCompileSdk ];
          buildToolsVersions = [ androidBuildTools ];
          includeNDK = true;
          ndkVersions = [ androidNdk ];
        }).androidsdk;
        androidHome = "${androidSdk}/libexec/android-sdk";
      in
      with pkgs;
      {
        devShells.default = mkShell rec {
          nativeBuildInputs = [
            pkg-config

            # Android
            androidSdk
            cargo-ndk
            gradle
            jdk21_headless
          ];
          buildInputs = [
              bacon
              cargo-machete
              cargo-edit
              cargo-deny
              wasm-bindgen-cli
              rust-analyzer
              rust

              udev alsa-lib vulkan-loader
              libX11 libXcursor libXi libXrandr # To use the x11 feature
              libxkbcommon wayland # To use the wayland feature
          ];
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

          ANDROID_HOME = androidHome;
          ANDROID_NDK_HOME = "${androidHome}/ndk/${androidNdk}";
          # Tools read either name: set both so a stray one can't point elsewhere.
          ANDROID_NDK_ROOT = "${androidHome}/ndk/${androidNdk}";
          JAVA_HOME = jdk21_headless.home;
          # Gradle downloads its own aapt2, which can't run on NixOS: use the SDK's.
          GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidHome}/build-tools/${androidBuildTools}/aapt2";
        };
      }
    );
}
