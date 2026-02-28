# IE Pass
IE Pass - The Console The Pass

<p align="center">
   <img src=".github/logo.svg" height="200"><br/><br/>
   <a href="https://github.com/funmaker/iepass"><img src="https://github.com/funmaker/iepass/actions/workflows/test.yml/badge.svg" height="23"></a>
</p>

## Usage

### Setup

1) Install [FFmpeg](https://ffmpeg.org/download.html).
2) Install cargo dependencies:
    ```bash
    rustup default nightly-2025-10-25
    cargo install cargo-make ldproxy
    cargo install espup probe-rs-tools --locked
    espup install --targets=esp32s3 --toolchain-version 1.90.0.0
    ```

### Build:
```bash
$ cargo make build
```

### Run Emulator:
```bash
$ cargo make run -- path/to/cart.p8
```

### Build and flash for ESP32-S3:
```bash
$ cargo make flash
```

### Run Tests:
```bash
$ cargo make test
```
