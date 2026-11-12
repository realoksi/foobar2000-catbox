# foobar2000-catbox

*A helper application designed to upload album art to [catbox.moe](https://catbox.moe/) for use
with [foo_discord_rich](https://github.com/TheQwertiest/foo_discord_rich) or its forks.*

[![Rust](https://img.shields.io/badge/Rust-D34516.svg?logo=rust&logoColor=white)](https://rust-lang.org/)
![License](https://img.shields.io/badge/license-MIT_or_Apache--2.0-blue)
[![Coverage](https://img.shields.io/codecov/c/gh/realoksi/foobar2000-catbox)](https://app.codecov.io/gh/realoksi/foobar2000-catbox)
[![Build](https://img.shields.io/github/actions/workflow/status/realoksi/foobar2000-catbox/rust.yml)](https://github.com/realoksi/foobar2000-catbox/actions/workflows/rust.yml)
[![Release](https://img.shields.io/github/v/release/realoksi/foobar2000-catbox)](https://github.com/realoksi/foobar2000-catbox/releases/latest)
![Downloads](https://img.shields.io/github/downloads/realoksi/foobar2000-catbox/total)

---

## Install

Download the [latest release](https://github.com/realoksi/foobar2000-catbox/releases), or
[compile it from source](#compile). Then follow the setup instructions
[here](https://theqwertiest.github.io/foo_discord_rich/docs/features/advanced_users/art_upload/).

See the [configuration](#configure) section for optional settings.

## Compile

Get [rustup](https://www.rust-lang.org/tools/install) and [git](https://git-scm.com/downloads).

Open a terminal.

Clone the repository
or [directly download](https://github.com/realoksi/foobar2000-catbox/archive/refs/heads/production.zip) and unzip it.

```sh
git clone https://github.com/realoksi/foobar2000-catbox.git
```

`cd` into the directory.

```sh
cd foobar2000-catbox
```

### Windows

Compile the application using the release profile.

```sh
cargo build -r
```

Find the executable at `target/release/foobar2000-catbox.exe`

### Cross-compiling from Linux or macOS

Add a Windows target with rustup.

```sh
rustup target add x86_64-pc-windows-gnu
```

You will also need [mingw-w64](https://www.mingw-w64.org/downloads).

Compile a release build with the Windows target specified.

```sh
cargo build --target x86_64-pc-windows-gnu -r
```

Find the executable at `target/x86_64-pc-windows-gnu/release/foobar2000-catbox.exe`

## Configure

To configure the output, create a `config.toml` file in the same directory as `foobar2000-catbox.exe`.

See the following table:

| Name                           | Type             | Default                              | Description                                                                                                                                                                                  |
|--------------------------------|------------------|--------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| user_hash                      | String           | Not set                              | Links uploads to your catbox account. Leave unset for anonymous uploads                                                                                                                      |
| timeout                        | Number (seconds) | 10                                   | Maximum time an upload can take, including connecting                                                                                                                                        |
| connection_timeout             | Number (seconds) | 5                                    | Maximum time to establish a connection                                                                                                                                                       |
| transcode                      | Table            | Not set                              | When set, the image is always re-encoded before being uploaded. When not set, the image is passed through unchanged                                                                          |
| transcode.format               | String           | jpg                                  | "png", "jpg" (or "jpeg"), or "webp"                                                                                                                                                          |
| transcode.quality              | Number (0-100)   | 80                                   | A fallback quality value for JPG and WebP. Ignored for PNG                                                                                                                                   |
| transcode.png                  | Table            | Not set                              |                                                                                                                                                                                              |
| transcode.png.compression      | String           | fast                                 | "fast", "best", or "uncompressed"                                                                                                                                                            |
| transcode.png.filter           | String           | adaptive                             | "nofilter", "sub", "up", "avg", "paeth", or "adaptive"                                                                                                                                       |
| transcode.jpg                  | Table            | Not set                              |                                                                                                                                                                                              |
| transcode.jpg.quality          | Number (0-100)   | transcode.quality                    | Overrides the global quality value for this format                                                                                                                                           |
| transcode.webp                 | Table            | Not set                              |                                                                                                                                                                                              |
| transcode.webp.quality         | Number (0-100)   | transcode.quality                    | Overrides the global quality value for this format                                                                                                                                           |
| transcode.webp.lossless        | Boolean          | false                                | Enables lossless compression. When this is true, quality determines the effort put into compression. So, 100 = most effort, smaller file, slowest and 0 = least effort, larger file, fastest |
| transcode.clamp                | Table            | Not set                              | When set, images are scaled down to fit the resolution below. Aspect ratio is preserved                                                                                                      |
| transcode.clamp.max_resolution | Inline table     | Required when transcode.clamp is set | { width = W, height = H } in pixels                                                                                                                                                          |

See a more complete example below:

```toml
user_hash = "fa06e93babb0163ad24951b25" # not a real hash
timeout = 10
connection_timeout = 5

[transcode]
format = "png"
quality = 80

[transcode.png]
compression = "best"
filter = "adaptive"

[transcode.jpg]
quality = 50

[transcode.webp]
quality = 60
lossless = true

[transcode.clamp]
max_resolution = { width = 470, height = 470 }
```

Lastly, you can use the sample [`config.toml`](config.toml) from this repository as a starting point.

### Notes:

- Configuration is optional, enabled only by the presence of a `config.toml` file. Without it, the image is passed
  through unchanged
- A malformed configuration will immediately fail
- Unknown keys will immediately fail
- JPG format discards transparency
- Adjust timeout if necessary. By default, [foo_discord_rich](https://github.com/TheQwertiest/foo_discord_rich) gives
  the
  uploader [only 10 seconds](https://theqwertiest.github.io/foo_discord_rich/docs/features/advanced_users/art_upload/#technical-details)
  to complete the upload

## License

[MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE)