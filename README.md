# Shuftle
Shuftle is just a hobby project, but aims to be a multiplayer collection of classical turn-based games. To play it, see [vitkorp.eu/shuftle](https://vitkorp.eu/shuftle) for every platform it's on.

The rules of the games live in [shuftlib](https://github.com/Shuftle/shuftlib), a Rust library for turn-based games. This repo is the client, built with [Bevy](https://bevy.org).

## Build from source
The [Nix](https://nixos.org) flake provides everything: Rust with the web and Android targets, the Android SDK and NDK, and the tools CI uses. Enter it with `nix develop`, or let [direnv](https://direnv.net) load it from `.envrc`. Without Nix, see [Bevy's setup guide](https://bevy.org/learn/quick-start/getting-started/setup/) for the system libraries.

On desktop:

```sh
cargo run --features dev
```

The `dev` feature links Bevy dynamically for faster rebuilds.

For Android, from `android/`, to build the debug APK into `app/build/outputs/apk/debug/`:

```sh
./gradlew assembleDebug
```

To build it and install it on a connected device or a running emulator, where it shows up as Shuftle next to any Play Store install:

```sh
./gradlew installDebug
```

## Contributing
Pull request titles follow [Conventional Commits](https://www.conventionalcommits.org).
The Rust [code of conduct](https://rust-lang.org/policies/code-of-conduct/) and the Rust [LLM usage policy](https://forge.rust-lang.org/policies/llm-usage.html) apply.

## Licences
The code is licensed under the GNU AGPLv3, in [`LICENSE-AGPL_v3`](LICENSE-AGPL_v3).

The art keeps its own licences. Each pack in [`assets/`](assets) has its own folder, with a `CREDITS.md` naming its source, author and licence, and a `LICENSE` where its licence asks for one.
