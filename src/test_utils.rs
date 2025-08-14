use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

/// Creates a temporary directory with a set of files for testing.
///
/// This function is in its own module so it can be shared between unit tests
/// and integration tests.
///
/// The directory will contain:
/// - `file1.txt` (content: "hello")
/// - `file2.txt` (content: "world!!")
/// - `file3.txt` (content: "hello") - a duplicate of file1.txt
/// - `file4.txt` (content: "different")
/// - a subdirectory `subdir` with `file5.txt` (content: "hello") - another duplicate
pub fn create_test_directory() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    // Create some files with duplicate content.
    File::create(path.join("file1.txt"))
        .unwrap()
        .write_all(b"hello")
        .unwrap();
    File::create(path.join("file2.txt"))
        .unwrap()
        .write_all(b"world!!")
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
