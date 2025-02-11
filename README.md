# Seshat

A simple [OPDS-compatible](https://specs.opds.io/opds-1.2) catalog viewer for Calibre libraries.

I built this so I can self-host my Calibre library on my mobile device without Calibre running on the network, or anywhere remotely.
In fact, the application can be ran directly on Android devices with [Termux](https://termux.dev/en/) installed.

It cannot be used on its own (at least for now, who knows?). You need a e-book reader capable of reading OPDS catalogs, such as [KOReader](https://github.com/koreader/koreader).

## Building

Building for your platform can be done with the ordinary `cargo` command.
Building for Android however, requires the [`cross`](https://github.com/cross-rs/cross?tab=readme-ov-file#installation) utility installed.

- Build for your host: `cargo build --release`
- Build for Android: `cross build --target aarch64-linux-android --release`

## Project Name

The name comes from [Seshat](https://en.wikipedia.org/wiki/Seshat), the ancient Egyptian goddess of writing, wisdom, and knowledge.

## Licensing

The source code is MIT licensed.
