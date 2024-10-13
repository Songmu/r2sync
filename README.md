# r2sync

[![Test Status](https://github.com/Songmu/r2sync/workflows/Rust/badge.svg)][actions]
[![License](https://img.shields.io/github/license/Songmu/r2sync)][license]
[![Crates.io](https://img.shields.io/crates/v/r2sync.svg)](https://crates.io/crates/r2sync)
[![Documentation](https://docs.rs/r2sync/badge.svg)](https://docs.rs/r2sync)

[actions]: https://githb.com/Songmu/r2sync/actions?workflow=Rust
[license]: https://github.com/Songmu/r2sync/blob/main/LICENSE

**r2sync** is a command-line tool for synchronizing files between a local directory and Cloudflare R2. It allows seamless syncing of files to and from your R2 bucket.

## Features

- Sync local directories with Cloudflare R2 buckets.

## Installation

To install `r2sync`, ensure you have [Rust](https://www.rust-lang.org/) installed, and then run:

```bash
cargo install r2sync
```

Alternatively, you can clone the repository and build the project locally:

```bash
git clone https://github.com/Songmu/r2sync.git
cd r2sync
cargo build --release
```

## Usage

Once installed, you can start syncing files by using the `r2sync` command.

### Basic Usage

```bash
r2sync ./localdir r2://bucket.example.com/dir
```

This will sync the contents of `./localdir` to the R2 bucket at `r2://bucket.example.com/dir`.

### Syncing from R2 to Local Directory

To sync files from R2 to a local directory:

```bash
r2sync r2://bucket.example.com/dir ./localdir
```

### Full Command Line Options

- `--public-domain - If a public domain is specified, the remote content is checked before the file is put out, and if it is identical, the put is skipped.

## Configuration

You can provide authentication details via environment variables.

### Environment Variables

```bash
export R2_ACCOUNT_ID=<your-account-id>
export R2_ACCESS_KEY_ID=<your-access-key>
export R2_SECRET_ACCESS_KEY=<your-secret-key>
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please submit a pull request or open an issue to discuss your ideas.

## Contact

For any questions or issues, feel free to open an issue on the [GitHub repository](https://github.com/Songmu/r2sync) or reach out via email at y.songmu@gmail.com.
