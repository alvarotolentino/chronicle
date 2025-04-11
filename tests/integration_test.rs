use std::process::Command;
use std::fs;

#[test]
#[ignore] // This requires a git repository to run, so we'll ignore it by default
fn test_basic_changelog_generation() {
    // Assuming we're running from the project root
    let output = Command::new("cargo")
        .args(["run", "--", "-r", ".", "-o", "test_changelog.md"])
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());
    
    let content = fs::read_to_string("test_changelog.md")
        .expect("Failed to read generated changelog");
    
    assert!(content.contains("# Changelog"));
    
    // Clean up
    fs::remove_file("test_changelog.md").ok();
}

#[test]
#[ignore]
fn test_html_output_format() {
    let output = Command::new("cargo")
        .args(["run", "--", "-r", ".", "-o", "test_changelog.html", "-f", "html"])
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());
    
    let content = fs::read_to_string("test_changelog.html")
        .expect("Failed to read generated changelog");
    
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("<title>Changelog</title>"));
    
    // Clean up
    fs::remove_file("test_changelog.html").ok();
}