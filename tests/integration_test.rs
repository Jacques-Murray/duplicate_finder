use assert_cmd::Command;
use duplicate_finder::test_utils::create_test_directory;
use predicates::prelude::*;

#[test]
fn test_interactive_delete_files() {
    let dir = create_test_directory();
    let dir_path = dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("duplicate_finder").unwrap();
    let input = "1 3\ny\n"; // Keep file 1 and 3, delete the others from the set.

    cmd.arg(dir_path)
        .arg("--delete")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted 1 files from this set."));

    // file1.txt, file3.txt from the duplicate set should exist.
    // file5.txt from the duplicate set should be deleted.
    assert!(dir.path().join("file1.txt").exists());
    assert!(dir.path().join("file3.txt").exists());
    assert!(!dir.path().join("subdir/file5.txt").exists());

    // Other files should also still exist.
    assert!(dir.path().join("file2.txt").exists());
    assert!(dir.path().join("file4.txt").exists());
}

#[test]
fn test_interactive_delete_cancel() {
    let dir = create_test_directory();
    let dir_path = dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("duplicate_finder").unwrap();
    let input = "1\nn\n"; // Try to delete files 2 and 3, but cancel.

    cmd.arg(dir_path)
        .arg("--delete")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deletion cancelled for this set."));

    // All original files should still exist.
    assert!(dir.path().join("file1.txt").exists());
    assert!(dir.path().join("file2.txt").exists());
    assert!(dir.path().join("file3.txt").exists());
    assert!(dir.path().join("file4.txt").exists());
    assert!(dir.path().join("subdir/file5.txt").exists());
}

#[test]
fn test_interactive_delete_all_from_set() {
    let dir = create_test_directory();
    let dir_path = dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("duplicate_finder").unwrap();
    let input = "none\ny\n"; // Keep no files from the set by providing non-numeric input.

    cmd.arg(dir_path)
        .arg("--delete")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted 3 files from this set."));

    // All files from the duplicate set should be deleted.
    assert!(!dir.path().join("file1.txt").exists());
    assert!(!dir.path().join("file3.txt").exists());
    assert!(!dir.path().join("subdir/file5.txt").exists());

    // Other files should also still exist.
    assert!(dir.path().join("file2.txt").exists());
    assert!(dir.path().join("file4.txt").exists());
}
