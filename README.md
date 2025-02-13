# Seshat

A simple [OPDS Catalog](https://specs.opds.io/opds-1.2) server for Calibre libraries.
I built this so I can self-host my Calibre libraries on my mobile devices without Calibre itself installed.
In fact, the application can be ran directly on Android devices with [Termux](https://termux.dev/en/) installed.

It cannot be used on its own (at least for now, who knows?). You need an e-book reader capable of reading OPDS Catalogs.
Personally, I use [KOReader](https://github.com/koreader/koreader) (not affiliated).

## Usage

First, build the application by running:

```sh
cargo build --release
```

Then, execute it by running:

```sh
./target/release/seshat \
    --lib:name "Awesome Library" --lib:path "./Awesome Library" \
    --lib:name "Bad Library" --lib:path "./Bad Library"
```

Multiple libraries are supported. Each library is defined by using the `--lib:name` and `--lib:path` options (in that order).
For more information, run `./target/release/seshat --help`.

## Building

To build for your host, run:

```sh
cargo build --release
```

### Building for Android

To build for Android, first install the [`cross`](https://github.com/cross-rs/cross?tab=readme-ov-file#installation) utility, as explained by their docs.

Afterwards, to build for the ARMv8-A/ARM64 architecture, run:

```sh
cross build --target aarch64-linux-android --release
```

To build for other architectures, substitute the target with one of the following options:

- `armv7-linux-androideabi` (ARMv7-a)
- `arm-linux-androideabi` (ARMv6)
- `x86_64-linux-android` (x86_64)

> [!NOTE]
> Only ARMv8-A is tested. Other targets are very likely to work, but it's not guaranteed.
> In any case, if any issues arise, [create an issue](https://github.com/thunder04/seshat/issues/new).

## Project Name

The name comes from [Seshat](https://en.wikipedia.org/wiki/Seshat), the ancient Egyptian goddess of writing, wisdom, and knowledge.

## Licensing

The source code is MIT licensed.
