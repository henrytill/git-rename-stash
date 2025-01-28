use assert_cmd::Command;
use tempfile::TempDir;

fn setup_git_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Initialize repo
    Command::new("git")
        .arg("init")
        .current_dir(&temp_dir)
        .assert()
        .success();

    // Configure git user for commit
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .assert()
        .success();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    if cfg!(windows) {
        Command::new("git")
            .args(["config", "core.autocrlf", "false"])
            .current_dir(&temp_dir)
            .assert()
            .success();
    };

    // Create initial commit
    std::fs::write(temp_dir.path().join("file.txt"), "initial\n").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(&temp_dir)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    temp_dir
}

fn create_stash(dir: &TempDir, content: &str, message: &str) {
    std::fs::write(dir.path().join("file.txt"), content).unwrap();
    Command::new("git")
        .args(["stash", "push", "-m", message])
        .current_dir(dir)
        .assert()
        .success();
}

fn get_stash_list(dir: &TempDir) -> Vec<String> {
    let output = Command::new("git")
        .args(["stash", "list"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

fn get_stash_message(dir: &TempDir, ref_name: &str) -> String {
    let output = Command::new("git")
        .args(["show", "-s", "--format=%s", ref_name])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn test_rename_most_recent_stash() {
    let temp_dir = setup_git_repo();

    create_stash(&temp_dir, "stash1\n", "first stash");

    Command::cargo_bin("git-rename-stash")
        .unwrap()
        .args(["stash@{0}", "renamed stash"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let stashes = get_stash_list(&temp_dir);
    assert_eq!(stashes.len(), 1);
    assert!(stashes[0].contains("renamed stash"));

    // Check the actual commit message was updated too
    let commit_msg = get_stash_message(&temp_dir, "stash@{0}");
    assert!(commit_msg.contains("renamed stash"));
}

#[test]
fn test_rename_older_stash() {
    let temp_dir = setup_git_repo();

    create_stash(&temp_dir, "stash1\n", "first stash");
    create_stash(&temp_dir, "stash2\n", "second stash");
    create_stash(&temp_dir, "stash3\n", "third stash");

    Command::cargo_bin("git-rename-stash")
        .unwrap()
        .args(["stash@{1}", "renamed second"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let stashes = get_stash_list(&temp_dir);
    println!("Actual stashes:");
    for stash in &stashes {
        println!("{}", stash);
    }
    assert_eq!(stashes.len(), 3);
    assert!(stashes[0].contains("renamed second"));
    assert!(stashes[1].contains("third stash"));
    assert!(stashes[2].contains("first stash"));

    // Check the actual commit message was updated
    let commit_msg = get_stash_message(&temp_dir, "stash@{0}");
    assert!(commit_msg.contains("renamed second"));
}

#[test]
fn test_error_invalid_stash() {
    let temp_dir = setup_git_repo();

    create_stash(&temp_dir, "stash1\n", "first stash");

    Command::cargo_bin("git-rename-stash")
        .unwrap()
        .args(["stash@{1}", "should fail"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn test_error_no_stashes() {
    let temp_dir = setup_git_repo();

    Command::cargo_bin("git-rename-stash")
        .unwrap()
        .args(["stash@{0}", "should fail"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn test_error_empty_message() {
    let temp_dir = setup_git_repo();

    create_stash(&temp_dir, "stash1\n", "first stash");

    Command::cargo_bin("git-rename-stash")
        .unwrap()
        .args(["stash@{0}", ""])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn test_stash_can_be_applied_after_rename() {
    let temp_dir = setup_git_repo();

    create_stash(&temp_dir, "stash1\n", "first stash");

    Command::cargo_bin("git-rename-stash")
        .unwrap()
        .args(["stash@{0}", "renamed stash"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // Try to apply the renamed stash
    Command::new("git")
        .args(["stash", "apply"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // Verify content was restored
    let content = std::fs::read_to_string(temp_dir.path().join("file.txt")).unwrap();
    assert_eq!(content, "stash1\n");
}

#[test]
fn test_whitespace_message() {
    let temp_dir = setup_git_repo();

    create_stash(&temp_dir, "stash1\n", "first stash");

    Command::cargo_bin("git-rename-stash")
        .unwrap()
        .args(["stash@{0}", "  spaces  "])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let stashes = get_stash_list(&temp_dir);
    assert_eq!(stashes.len(), 1);
    assert!(stashes[0].contains("spaces"));
}
