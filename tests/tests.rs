use anyhow::bail;
use anyhow::Context;
use glob::glob;
use pretty_assertions::StrComparison;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::{path::Path, process::Output};
use tokio::process::Command;

#[tokio::test(flavor = "multi_thread")]
async fn check_examples() -> anyhow::Result<()> {
    let mut set = tokio::task::JoinSet::new();

    for (test_name, file) in list_files(["examples/*/run.toml"]) {
        set.spawn(async move {
            check_example(&file)
                .await
                .with_context(|| format!("example failed: {}", &test_name))?;
            Ok::<String, anyhow::Error>(test_name)
        });
    }

    while let Some(Ok(joined)) = set.join_next().await {
        let test_name = joined?;
        println!("[ok] {}", &test_name);
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn run_tests() -> anyhow::Result<()> {
    let mut set = tokio::task::JoinSet::new();

    for (test_name, file) in list_files(["tests/**/*.toml"]) {
        set.spawn(async move {
            run_test(&file)
                .await
                .with_context(|| format!("test failed: {}", &test_name))?;
            Ok::<String, anyhow::Error>(test_name)
        });
    }

    while let Some(Ok(joined)) = set.join_next().await {
        let test_name = joined?;
        println!("[ok] {}", &test_name);
    }

    Ok(())
}

async fn check_example<P: AsRef<Path>>(file: P) -> anyhow::Result<()> {
    let output = exec(&file, ["--check"]).await?;

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr)?;
        bail!("unexpectedly failed with: {}", stderr);
    }

    Ok(())
}

async fn run_test<P: AsRef<Path>>(file: P) -> anyhow::Result<()> {
    let args = read_file(&file, ".args").await.unwrap_or_default();
    let expected_stdout = read_file(&file, ".stdout")
        .await
        .map(patch_env)
        .map(patch_eol);
    let expected_stderr = read_file(&file, ".stderr")
        .await
        .map(patch_env)
        .map(patch_eol);

    if expected_stdout.is_none() && expected_stderr.is_none() {
        bail!("none of .stdout or .stderr found");
    }

    // exec and get output
    let output = exec(&file, args.lines()).await?;
    let stdout = patch_eol(std::str::from_utf8(&output.stdout)?);
    let stderr = patch_eol(std::str::from_utf8(&output.stderr)?);

    if !output.status.success() && expected_stderr.is_none() {
        bail!("unexpectedly failed with: {}", stderr);
    }

    // assert stdout
    if let Some(expected) = expected_stdout {
        if expected != stdout {
            bail!(format!(
                "stdout does not match: {}",
                StrComparison::new(&expected, &stdout)
            ));
        }
    }

    // assert stderr
    if let Some(expected) = expected_stderr {
        if expected != stderr {
            bail!(format!(
                "stderr does not match: {}",
                StrComparison::new(&expected, &stderr)
            ));
        }
    }

    Ok(())
}

async fn exec<P, I, S>(file: P, args: I) -> anyhow::Result<Output>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_run"));
    cmd.env_clear();
    cmd.arg("-f");
    cmd.arg(file.as_ref());
    cmd.args(args);
    Ok(cmd.output().await?)
}

async fn read_file<P: AsRef<Path>>(filepath: P, suffix: &str) -> Option<String> {
    let filepath = filepath.as_ref().to_str()?.to_string() + suffix;
    tokio::fs::read_to_string(&filepath).await.ok()
}

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn list_files<I, D>(patterns: I) -> impl Iterator<Item = (String, PathBuf)>
where
    I: IntoIterator<Item = D>,
    D: std::fmt::Display,
{
    patterns
        .into_iter()
        .map(|pattern| format!("{}/{}", CARGO_MANIFEST_DIR, pattern))
        .flat_map(|pattern| glob(&pattern).unwrap().map(|entry| entry.unwrap()))
        .map(|file| {
            let test_name = &file.strip_prefix(CARGO_MANIFEST_DIR).unwrap();
            let test_name = test_name.to_str().unwrap().to_string();
            (test_name, file)
        })
}

/// patch_env replaces $CARGO_MANIFEST_DIR with the actual path. This is useful as some path are
/// actually absolute path, and we don't want to hardcode the absolute path in the test files.
fn patch_env<S: AsRef<str>>(s: S) -> String {
    s.as_ref()
        .replace("$CARGO_MANIFEST_DIR", CARGO_MANIFEST_DIR)
}

fn patch_eol<S: AsRef<str>>(s: S) -> String {
    s.as_ref().replace("\r\n", "\n")
}
