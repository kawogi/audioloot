# Audioloot - Discord Music Bot

This bot plays tracks from [Audiotool](https://audiotool.com) in a Discord voice channel.

This was just a hobby project and as such has a _works for me™_ quality.

Use at your own risk. If you like a track, dont't forget to tell the artist!

## How to build?

0. Install `libssl-dev` and `libopus-dev` as a prerequisite.
1. Install [the rust toolchain](https://www.rust-lang.org/learn/get-started) if you haven't already.
2. Check out this project
3. find `// Configure Guild and channel roles` in the `main.rs` and change the hard-coded ids to match your server and channels
4. Build the project running `cargo build --release` from the project's root

## How to run?

Linux command line: `DISCORD_TOKEN="MyVerySecretTokenThatIWillNeverShareWithAnyone" target/release/audioloot`