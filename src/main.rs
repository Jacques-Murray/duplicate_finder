//! A command-line utility for finding duplicate files in a directory.
//!
//! This tool scans a specified directory, groups files by size,
//! and then computes SHA-256 hashes for files of the same size to identify duplicates.
//! It uses parallelism to speed up the hashing process.

use clap::Parser;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

/// A type alias for a map where keys are file sizes and values are vectors of file paths.
type FileGroups = HashMap<u64, Vec<PathBuf>>;
/// A type alias for a map where keys are file hashes and values are vectors of file paths.
type DuplicateGroups = HashMap<String, Vec<PathBuf>>;

/// Defines the command-line arguments for the application.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The directory to scan for duplicate files.
    #[arg(required = true, value_name = "DIRECTORY")]
    path: PathBuf,
}

/// The size of the buffer used for reading files when hashing.
const BUFFER_SIZE: usize = 8192;

/// Computes the SHA-256 hash of a file.
///
/// # Arguments
///
/// * `path` - A `PathBuf` to the file to be hashed.
///
/// # Returns
///
/// A `Result` containing the hex-encoded hash string, or an `io::Error` if the file
/// cannot be read.
fn compute_hash(path: &PathBuf) -> io::Result<String> {
    use std::io::BufReader;

    // Open the file and create a buffered reader for efficiency.
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; BUFFER_SIZE];

    // Read the file in chunks and update the hasher.
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Finalize the hash and format it as a hex string.
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Groups files in a directory by their size.
///
/// This function walks the specified directory and collects all files, grouping them
/// into a `HashMap` where the key is the file size and the value is a list of paths
/// of files with that size. It only includes groups with more than one file, as
/// these are potential duplicates.
///
/// # Arguments
///
/// * `path` - A `PathBuf` to the directory to be scanned.
///
/// # Returns
///
/// A `FileGroups` map containing files grouped by size.
fn group_files_by_size(path: &PathBuf) -> FileGroups {
    let mut files_by_size: FileGroups = HashMap::new();

    // Walk the directory, filtering for files.
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                // Group files by size, ignoring empty files.
                if metadata.len() > 0 {
                    files_by_size
                        .entry(metadata.len())
                        .or_default()
                        .push(entry.into_path());
                }
            }
        }
    }

    // Filter out groups with only one file, as they cannot be duplicates.
    files_by_size
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect()
}

/// Finds duplicate files by hashing files of the same size.
///
/// This function takes a map of files grouped by size and computes the SHA-256 hash
/// for each file in parallel using `rayon`. It then groups the files by their hash.
/// Groups with more than one file are considered duplicates.
///
/// # Arguments
///
/// * `potential_duplicates` - A `FileGroups` map where files are grouped by size.
///
/// # Returns
///
/// A `DuplicateGroups` map containing the duplicate files, grouped by hash.
fn find_duplicates(potential_duplicates: FileGroups) -> DuplicateGroups {
    let mut duplicates: DuplicateGroups = HashMap::new();

    // Compute hashes for all potential duplicate files in parallel.
    let hash_results: Vec<_> = potential_duplicates
        .into_par_iter()
        .flat_map(|(_, files)| {
            files
                .into_par_iter()
                .filter_map(|path| match compute_hash(&path) {
                    Ok(hash) => Some((hash, path)),
                    Err(e) => {
                        eprintln!("Failed to hash file '{}': {}", path.display(), e);
                        None
                    }
                })
        })
        .collect();

    // Group files by their computed hash.
    for (hash, path) in hash_results {
        duplicates.entry(hash).or_default().push(path);
    }

    // Filter out groups with only one file, leaving only the actual duplicates.
    duplicates
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect()
}

/// Prints the identified duplicate files to the console.
///
/// # Arguments
///
/// * `duplicates` - A `DuplicateGroups` map containing the duplicate files.
fn print_duplicates(duplicates: &DuplicateGroups) {
    if duplicates.is_empty() {
        println!("No duplicate files found.");
        return;
    }

    // Print each set of duplicate files.
    for (index, files) in duplicates.values().enumerate() {
        println!("--- Duplicate Set {} ---", index + 1);
        for path in files {
            println!("{}", path.display());
        }
    }
}

/// The main entry point of the application.
fn main() {
    // Parse command-line arguments.
    let cli = Cli::parse();

    // Ensure the provided path is a directory.
    if !cli.path.is_dir() {
        eprintln!("Error: Provided path is not a directory.");
        std::process::exit(1);
    }

    println!("Scanning directory: {}", cli.path.display());

    // Step 1: Group files by size to find potential duplicates.
    let potential_duplicates = group_files_by_size(&cli.path);

    // Step 2: Hash the potential duplicates to find actual duplicates.
    let duplicates = find_duplicates(potential_duplicates);

    // Step 3: Print the results.
    print_duplicates(&duplicates);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    /// Creates a temporary directory with a set of files for testing.
    ///
    /// The directory will contain:
    /// - `file1.txt` (content: "hello")
    /// - `file2.txt` (content: "world")
    /// - `file3.txt` (content: "hello") - a duplicate of file1.txt
    /// - `file4.txt` (content: "different")
    /// - a subdirectory `subdir` with `file5.txt` (content: "hello") - another duplicate
    fn create_test_directory() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create some files with duplicate content.
        File::create(path.join("file1.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        File::create(path.join("file2.txt"))
            .unwrap()
            .write_all(b"world")
            .unwrap();
        File::create(path.join("file3.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        File::create(path.join("file4.txt"))
            .unwrap()
            .write_all(b"different")
            .unwrap();

        // Create a subdirectory with a duplicate file.
        fs::create_dir(path.join("subdir")).unwrap();
        File::create(path.join("subdir/file5.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();

        dir
    }

    #[test]
    fn test_compute_hash() {
        let dir = create_test_directory();
        let path = dir.path().join("file1.txt");
        let hash = compute_hash(&path).unwrap();
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_group_files_by_size() {
        let dir = create_test_directory();
        let path = dir.path().to_path_buf();
        let groups = group_files_by_size(&path);

        // There should be one group of files with the same size (the "hello" files).
        assert_eq!(groups.len(), 1);

        // The group should have a size of 5 bytes (the length of "hello").
        let (size, files) = groups.iter().next().unwrap();
        assert_eq!(*size, 5);

        // There should be four files in this group.
        assert_eq!(files.len(), 4);
    }

    #[test]
    fn test_find_duplicates() {
        let dir = create_test_directory();
        let path = dir.path().to_path_buf();
        let potential_duplicates = group_files_by_size(&path);
        let duplicates = find_duplicates(potential_duplicates);

        // There should be one set of duplicates.
        assert_eq!(duplicates.len(), 1);

        // The duplicate set should contain three files.
        let (_, files) = duplicates.iter().next().unwrap();
        assert_eq!(files.len(), 3);

        // Verify that the correct files are identified as duplicates.
        let mut file_names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        file_names.sort();
        assert_eq!(file_names, vec!["file1.txt", "file3.txt", "file5.txt"]);
    }

    #[test]
    fn test_no_duplicates() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create files with unique content.
        File::create(path.join("file1.txt"))
            .unwrap()
            .write_all(b"one")
            .unwrap();
        File::create(path.join("file2.txt"))
            .unwrap()
            .write_all(b"two")
            .unwrap();

        let potential_duplicates = group_files_by_size(&path.to_path_buf());
        let duplicates = find_duplicates(potential_duplicates);

        // There should be no duplicates found.
        assert!(duplicates.is_empty());
    }
}
