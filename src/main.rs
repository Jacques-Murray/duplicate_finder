use clap::Parser;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

type FileGroups = HashMap<u64, Vec<PathBuf>>;
type DuplicateGroups = HashMap<String, Vec<PathBuf>>;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The directory to scan for duplicate files
    #[arg(required = true, value_name = "DIRECTORY")]
    path: PathBuf,
}

fn compute_hash(path: &PathBuf) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn group_files_by_size(path: &PathBuf) -> FileGroups {
    let mut files_by_size: FileGroups = HashMap::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.len() > 0 {
                    files_by_size
                        .entry(metadata.len())
                        .or_default()
                        .push(entry.into_path());
                }
            }
        }
    }

    files_by_size
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect()
}

fn find_duplicates(potential_duplicates: FileGroups) -> DuplicateGroups {
    let mut duplicates: DuplicateGroups = HashMap::new();

    let hash_results: Vec<_> = potential_duplicates
        .into_par_iter()
        .flat_map(|(_, files)| {
            files
                .into_par_iter()
                .filter_map(|path| compute_hash(&path).ok().map(|hash| (hash, path)))
        })
        .collect();

    for (hash, path) in hash_results {
        duplicates.entry(hash).or_default().push(path);
    }

    duplicates
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect()
}

fn print_duplicates(duplicates: &DuplicateGroups) {
    if duplicates.is_empty() {
        println!("No duplicate files found.");
        return;
    }

    for (index, files) in duplicates.values().enumerate() {
        println!("--- Duplicate Set {} ---", index + 1);
        for path in files {
            println!("{}", path.display());
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if !cli.path.is_dir() {
        eprintln!("Error: Provided path is not a directory.");
        std::process::exit(1);
    }

    println!("Scanning directory: {}", cli.path.display());

    let potential_duplicates = group_files_by_size(&cli.path);

    let duplicates = find_duplicates(potential_duplicates);

    print_duplicates(&duplicates);
}
