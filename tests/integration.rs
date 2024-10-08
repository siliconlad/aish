use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn canonicalize_path(path: &str) -> String {
    PathBuf::from(path)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn run_shell_command(input: &str) -> (String, String, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let aish_path = env!("CARGO_BIN_EXE_aish");

    let mut child = Command::new(aish_path)
        .current_dir(temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let input = input.to_string() + "\n";
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin
        .write_all(input.as_bytes())
        .expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stdout");

    (
        String::from_utf8_lossy(&output.stdout)
            .to_string()
            .trim()
            .to_string(),
        String::from_utf8_lossy(&output.stderr)
            .to_string()
            .trim()
            .to_string(),
        temp_dir.path().to_owned().to_str().unwrap().to_string(),
    )
}

#[test]
fn test_echo_command() {
    let (stdout, stderr, _) = run_shell_command("echo \"Hello, world!\"");
    assert_eq!(stdout, "Hello, world!");
    assert_eq!(stderr, "");
}

#[test]
fn test_and_sequence() {
    let (stdout, stderr, _) = run_shell_command("echo First && echo Second");
    assert_eq!(stdout, "First\nSecond");
    assert_eq!(stderr, "");
}

#[test]
fn test_and_sequence_error() {
    let (stdout, stderr, _) = run_shell_command("cd /nonexistent && echo Second");
    assert_eq!(stdout, "");
    assert!(!stderr.is_empty());
}

#[test]
fn test_pwd_command() {
    let (stdout, stderr, _) = run_shell_command("pwd");
    assert!(!stdout.is_empty());
    assert_eq!(stderr, "");
}

#[test]
fn test_exit_command() {
    let (stdout, stderr, _) = run_shell_command("exit");
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
}

#[test]
fn test_cd_command() {
    let (stdout, stderr, _) = run_shell_command("cd /tmp && pwd");
    assert_eq!(stdout, canonicalize_path("/tmp"));
    assert_eq!(stderr, "");
}

#[test]
fn test_cd_tilde_command() {
    let (stdout, stderr, _) = run_shell_command("cd ~ && pwd");
    assert_eq!(
        stdout,
        canonicalize_path(env::var("HOME").unwrap().as_str())
    );
    assert_eq!(stderr, "");
}

#[test]
fn test_cd_tilde_path() {
    let (stdout, stderr, _) = run_shell_command("cd ~/ && pwd");
    assert_eq!(
        stdout,
        canonicalize_path(env::var("HOME").unwrap().as_str())
    );
    assert_eq!(stderr, "");
}

#[test]
fn test_tilde_mkdir() {
    let (stdout, stderr, root) = run_shell_command("mkdir ~test && cd ~test && pwd");
    assert_eq!(stdout, format!("{}/~test", root));
    assert_eq!(stderr, "");
}

#[test]
fn test_tilde_echo() {
    let (stdout, stderr, _) = run_shell_command("echo ~");
    assert_eq!(stdout, env::var("HOME").unwrap());
    assert_eq!(stderr, "");
}

#[test]
fn test_tilde_variable() {
    let (stdout, stderr, _) = run_shell_command("export FOO=BAR && echo ~$FOO");
    assert_eq!(stdout, "~BAR");
    assert_eq!(stderr, "");
}

#[test]
fn test_tilde_variable_value() {
    let (stdout, stderr, _) = run_shell_command("export FOO=~ && echo $FOO");
    assert_eq!(
        stdout,
        canonicalize_path(env::var("HOME").unwrap().as_str())
    );
    assert_eq!(stderr, "");
}

#[test]
fn test_tilde_variable_value_path() {
    let (stdout, stderr, _) = run_shell_command("export FOO=~/Documents && echo $FOO");
    assert_eq!(
        stdout,
        format!(
            "{}/Documents",
            canonicalize_path(env::var("HOME").unwrap().as_str())
        )
    );
    assert_eq!(stderr, "");
}

#[test]
fn test_tilde_variable_value_literal() {
    let (stdout, stderr, _) = run_shell_command("export FOO=~Documents && echo $FOO");
    assert_eq!(stdout, "~Documents");
    assert_eq!(stderr, "");
}

#[test]
fn test_pipeline() {
    let (stdout, stderr, _) = run_shell_command("echo Hello | wc -c");
    assert_eq!(stdout, "6");
    assert_eq!(stderr, "");
}

#[test]
fn test_output_redirection() {
    let (stdout, stderr, _) = run_shell_command("echo Hello > test.txt && cat test.txt");
    assert_eq!(stdout, "Hello");
    assert_eq!(stderr, "");
}

#[test]
fn test_output_redirection_append() {
    let (stdout, stderr, _) = run_shell_command(
        "echo Hello > test.txt && \
         echo World >> test.txt && \
         cat test.txt",
    );
    assert_eq!(stdout, "Hello\nWorld");
    assert_eq!(stderr, "");
}

#[test]
fn test_input_redirection() {
    let (stdout, stderr, _) =
        run_shell_command("echo Hello > test.txt && sed 's/H/h/g' < test.txt");
    assert_eq!(stdout, "hello");
    assert_eq!(stderr, "");
}

#[test]
fn test_builtin_error() {
    let (stdout, stderr, _) = run_shell_command("cd /nonexistent");
    assert_eq!(stdout, "");
    assert!(!stderr.is_empty());
}

#[test]
fn test_external_command_error() {
    let (stdout, stderr, _) = run_shell_command("nonexistent_command");
    assert_eq!(stdout, "");
    assert!(!stderr.is_empty());
}

#[test]
fn test_non_utf8_output() {
    let (stdout, stderr, _) = run_shell_command("cat /dev/random | head -n 1");
    assert!(!stdout.is_empty());
    assert_eq!(stderr, "");
}

#[test]
fn test_export_command() {
    let (stdout, stderr, _) = run_shell_command("export FOO=BAR && echo $FOO");
    assert_eq!(stdout, "BAR");
    assert_eq!(stderr, "");
}

#[test]
fn test_export_command_semicolon() {
    let (stdout, stderr, _) = run_shell_command("export FOO=BAR; echo $FOO");
    assert_eq!(stdout, "BAR");
    assert_eq!(stderr, "");
}

#[test]
fn test_export_command_double_quotes() {
    let (stdout, stderr, _) = run_shell_command("export FOO=BAR && echo \"$FOO\"");
    assert_eq!(stdout, "BAR");
    assert_eq!(stderr, "");
}

#[test]
fn test_export_command_double_quotes_with_text() {
    let (stdout, stderr, _) = run_shell_command("export FOO=BAR && echo \"$FOO Hello, World!\"");
    assert_eq!(stdout, "BAR Hello, World!");
    assert_eq!(stderr, "");
}

#[test]
fn test_export_command_escaped() {
    let (stdout, stderr, _) = run_shell_command("export FOO=BAR && echo \\$FOO");
    assert_eq!(stdout, "$FOO");
    assert_eq!(stderr, "");
}

#[test]
fn test_export_command_single_quotes() {
    let (stdout, stderr, _) = run_shell_command("export FOO=BAR && echo '$FOO'");
    assert_eq!(stdout, "$FOO");
    assert_eq!(stderr, "");
}
