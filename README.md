# Duplicate File Finder

A command-line utility for finding duplicate files in a directory.

This tool scans a specified directory, groups files by size, and then computes SHA-256 hashes for files of the same size to identify duplicates. It uses parallelism to speed up the hashing process.

## Features

- Scans a directory recursively for files.
- Groups files by size to quickly identify potential duplicates.
- Computes SHA-256 hashes for files of the same size to confirm duplicates.
- Uses `rayon` for parallel processing to speed up hashing.
- Simple command-line interface.

## Usage

To use the Duplicate File Finder, you need to have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).

1. **Clone the repository:**
   ```sh
   git clone https://github.com/your-username/duplicate-finder.git
   cd duplicate-finder
   ```

2. **Build the project:**
   ```sh
   cargo build --release
   ```

3. **Run the tool:**
   ```sh
   ./target/release/duplicate_finder <DIRECTORY>
   ```
   Replace `<DIRECTORY>` with the path to the directory you want to scan.

   For example:
   ```sh
   ./target/release/duplicate_finder /path/to/your/documents
   ```

## How It Works

The tool works in three main steps:

1.  **Group files by size:** It walks the directory and groups all files by their size. Only groups with more than one file are considered potential duplicates.
2.  **Compute hashes:** For each group of files with the same size, it computes the SHA-256 hash of each file. This is done in parallel to improve performance.
3.  **Identify duplicates:** Files with the same hash are considered duplicates. The tool then prints out the sets of duplicate files.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
