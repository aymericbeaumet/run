use assert_cmd::{assert::Assert, Command};
use glob::glob;
use pretty_assertions::assert_str_eq;
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

const PATTERNS: [&str; 2] = [
    // try to match all workbench.toml files in the examples directory
    "examples/*/workbench.toml",
    // run the workbench.toml files in the tests directory
    "tests/*.toml",
];

#[tokio::test(flavor = "multi_thread")]
async fn test_invocations() {
    let tasks = PATTERNS
        .iter()
        .map(|pattern| format!("{}/{}", ROOT, pattern))
        .flat_map(|pattern| glob(&pattern).unwrap().map(|entry| entry.unwrap()))
        .map(|file| {
            // TODO: limit concurrency
            tokio::spawn(async move {
                let pretty_file = file.strip_prefix(ROOT).unwrap();

                if read_file(&file, ".skip").await.is_some() {
                    println!("[skipped] {:?}", &pretty_file);
                    return;
                }

                let stdout = read_file(&file, ".stdout").await.unwrap_or_default();
                let stderr = read_file(&file, ".stderr").await.unwrap_or_default();

                // run and assert success
                let assert = workbench(Some(&file)).success();

                // assert stdout
                assert_str_eq!(
                    std::str::from_utf8(&assert.get_output().stdout).unwrap(),
                    stdout
                );

                // assert stderr
                assert_str_eq!(
                    std::str::from_utf8(&assert.get_output().stderr).unwrap(),
                    stderr,
                );

                println!("[ok]      {:?}", &pretty_file);
            })
        });

    futures::future::join_all(tasks).await;
}

async fn read_file<P: AsRef<Path>>(filepath: P, suffix: &str) -> Option<String> {
    let filepath = filepath.as_ref().to_str().unwrap().to_string() + suffix;
    tokio::fs::read_to_string(&filepath).await.ok()
}

fn workbench<P>(file: Option<P>) -> Assert
where
    P: AsRef<Path>,
{
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

    if let Some(file) = file {
        cmd.arg("-f");
        cmd.arg(file.as_ref());
    }

    cmd.assert()
}
