use clap::Parser;
use duplicate_finder::{find_duplicates, group_files_by_size, DuplicateGroups};
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;

/// Defines the command-line arguments for the application.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The directory to scan for duplicate files.
    #[arg(required = true, value_name = "DIRECTORY")]
    path: PathBuf,

    /// Interactively delete duplicate files.
    #[arg(long)]
    delete: bool,
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
    println!("Found {} sets of duplicate files.", duplicates.len());
    for (index, files) in duplicates.values().enumerate() {
        println!("\n--- Duplicate Set {} ---", index + 1);
        for path in files {
            println!("{}", path.display());
        }
    }
}

/// Interactively deletes duplicate files.
///
/// # Arguments
///
/// * `duplicates` - A `DuplicateGroups` map containing the duplicate files.
fn interactive_delete(duplicates: &DuplicateGroups) {
    if duplicates.is_empty() {
        println!("No duplicate files found to delete.");
        return;
    }

    println!("Found {} sets of duplicate files.", duplicates.len());
    println!("Starting interactive deletion process...");

    let mut total_deleted = 0;

    for (index, files) in duplicates.values().enumerate() {
        println!("\n--- Duplicate Set {} ---", index + 1);
        for (i, path) in files.iter().enumerate() {
            println!("[{}] {}", i + 1, path.display());
        }

        'set_loop: loop {
            print!("Enter the numbers of files to KEEP (e.g., '1 3', 'all', or press Enter to skip): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.is_empty() {
                println!("Skipping this set.");
                break 'set_loop;
            }

            let files_to_keep: Vec<usize> = if input.eq_ignore_ascii_case("all") {
                (1..=files.len()).collect()
            } else {
                input
                    .split_whitespace()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .filter(|&n| n > 0 && n <= files.len())
                    .collect()
            };

            let mut files_to_delete = Vec::new();
            for (i, file_path) in files.iter().enumerate() {
                if !files_to_keep.contains(&(i + 1)) {
                    files_to_delete.push(file_path);
                }
            }

            if files_to_delete.is_empty() {
                println!("No files selected for deletion in this set.");
                break 'set_loop;
            }

            println!("\nFiles to be DELETED:");
            for path in &files_to_delete {
                println!("- {}", path.display());
            }

            print!("Are you sure you want to delete these {} files? [y/N]: ", files_to_delete.len());
            io::stdout().flush().unwrap();
            let mut confirmation = String::new();
            io::stdin().read_line(&mut confirmation).unwrap();

            if confirmation.trim().eq_ignore_ascii_case("y") {
                let mut deleted_count = 0;
                for path in files_to_delete {
                    match fs::remove_file(path) {
                        Ok(_) => {
                            println!("Deleted: {}", path.display());
                            deleted_count += 1;
                        }
                        Err(e) => eprintln!("Error deleting {}: {}", path.display(), e),
                    }
                }
                println!("Deleted {} files from this set.", deleted_count);
                total_deleted += deleted_count;
                break 'set_loop;
            } else {
                println!("Deletion cancelled for this set. Please re-enter your selection.");
            }
        }
    }

    if total_deleted > 0 {
        println!("\nTotal files deleted: {}", total_deleted);
    } else {
        println!("\nNo files were deleted.");
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

    // Step 3: Print results or start interactive deletion.
    if cli.delete {
        interactive_delete(&duplicates);
    } else {
        print_duplicates(&duplicates);
    }
}
