use glob::glob;
use pretty_assertions::assert_str_eq;
use std::{
    path::{Path, PathBuf},
    process::Output,
};
use tokio::process::Command;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

const PATTERNS: [&str; 2] = [
    // try to match all workbench.toml files in the examples directory
    "examples/*/workbench.toml",
    // run the workbench.toml files in the tests directory
    "tests/*.toml",
];

#[tokio::test(flavor = "multi_thread")]
async fn test_all() {
    let mut set = tokio::task::JoinSet::new();

    PATTERNS
        .iter()
        .map(|pattern| format!("{}/{}", ROOT, pattern))
        .flat_map(|pattern| glob(&pattern).unwrap().map(|entry| entry.unwrap()))
        .for_each(|file| {
            set.spawn(test_one(file));
        });

    while set.join_next().await.is_some() {}
}

async fn test_one(file: PathBuf) {
    let pretty_file = file.strip_prefix(ROOT).unwrap();

    if read_file(&file, ".skip").await.is_some() {
        println!("[skipped] {:?}", &pretty_file);
        return;
    }

    let expected_stdout = read_file(&file, ".stdout").await;
    let expected_stderr = read_file(&file, ".stderr").await;

    if expected_stdout.is_none() && expected_stderr.is_none() {
        panic!("none one of .stdout/.stderr found for {:?}", pretty_file);
    }

    // run and get output
    let output = workbench(Some(&file)).await;

    // assert stdout
    if let Some(expected_stdout) = expected_stdout {
        assert_str_eq!(
            std::str::from_utf8(&output.stdout).unwrap(),
            expected_stdout,
            "{:?}",
            pretty_file
        );
    }

    // assert stderr
    if let Some(expected_stderr) = expected_stderr {
        assert_str_eq!(
            std::str::from_utf8(&output.stderr).unwrap(),
            expected_stderr,
            "{:?}",
            pretty_file
        );
    }

    println!("[ok]      {:?}", &pretty_file);
}

async fn read_file<P: AsRef<Path>>(filepath: P, suffix: &str) -> Option<String> {
    let filepath = filepath.as_ref().to_str().unwrap().to_string() + suffix;
    tokio::fs::read_to_string(&filepath).await.ok()
}

async fn workbench<P>(file: Option<P>) -> Output
where
    P: AsRef<Path>,
{
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_workbench"));

    if let Some(file) = file {
        cmd.arg("-f");
        cmd.arg(file.as_ref());
    }

    cmd.output().await.unwrap()
}
