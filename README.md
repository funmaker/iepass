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
    cargo install cargo-make ldproxy espflash
    cargo install espup probe-rs-tools --locked
    espup install --targets=esp32s3
    ```

### Build:
```bash
$ cargo make build
```

### Run Tests:
```bash
$ cargo make test
```

### Build and flash:
```bash
$ cargo make flash
```
