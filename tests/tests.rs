use anyhow::bail;
use anyhow::Context;
use glob::glob;
use pretty_assertions::StrComparison;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::{path::Path, process::Output};
use tokio::process::Command;

lazy_static::lazy_static! {
    static ref COREUTILS_PATH: PathBuf = install_local_coreutils();
}

#[tokio::test(flavor = "multi_thread")]
async fn run_examples_checks() -> anyhow::Result<()> {
    let mut set = tokio::task::JoinSet::new();

    for (test_name, file) in list_files(["examples/*/run.toml"]) {
        set.spawn(async move {
            example_check(&file)
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
async fn run_e2e_tests() -> anyhow::Result<()> {
    let mut set = tokio::task::JoinSet::new();

    for (test_name, file) in list_files(["tests/**/*.toml"]) {
        set.spawn(async move {
            e2e_test(&file)
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

async fn example_check<P: AsRef<Path>>(file: P) -> anyhow::Result<()> {
    let output = exec(&file, ["--check"]).await?;

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr)?;
        bail!("unexpectedly failed with: {}", stderr);
    }

    Ok(())
}

async fn e2e_test<P: AsRef<Path>>(file: P) -> anyhow::Result<()> {
    let args = read_file(&file, ".args").await.unwrap_or_default();
    let expected_stdout = read_file(&file, ".stdout").await.map(patch);
    let expected_stderr = read_file(&file, ".stderr").await.map(patch);

    if expected_stdout.is_none() && expected_stderr.is_none() {
        bail!("none of .stdout or .stderr found");
    }

    // exec and get output
    let output = exec(&file, args.lines()).await?;
    let stdout = patch(std::str::from_utf8(&output.stdout)?);
    let stderr = patch(std::str::from_utf8(&output.stderr)?);

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

    cmd.env("PATH", COREUTILS_PATH.to_str().unwrap());

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

/// patch performs a few operations to make sure the tests run nicely on all platforms:
/// 0. it trims the string
/// 1. it replaces $CARGO_MANIFEST_DIR with the actual path. This is useful as some path are
///    actually absolute path, and we don't want to hardcode the absolute path in the test files.
/// 2. it deletes windows extended length marker (\\?\)
/// 3. it replaces the windows EOL with unix EOL
/// 4. it replaces the backslash with forward slash
fn patch<S: AsRef<str>>(s: S) -> String {
    s.as_ref()
        .trim()
        .replace("$CARGO_MANIFEST_DIR", CARGO_MANIFEST_DIR)
        .replace("\\\\?\\", "")
        .replace("\r\n", "\n")
        .replace('\\', "/")
}

fn install_local_coreutils() -> PathBuf {
    const CRATE_NAME: &str = "coreutils";
    const CRATE_VERSION: &str = "0.0.18";
    const CRATE_FEATURES: &[&str] = &["cat", "echo", "ls", "printenv", "timeout"];

    let root = PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join(format!(
            ".bin/{}@{}#{}",
            CRATE_NAME,
            CRATE_VERSION,
            CRATE_FEATURES.join(",")
        ));

    if !root.is_dir() {
        let _ = std::fs::remove_dir_all(&root);
        let mut cmd = std::process::Command::new(env!("CARGO"));
        cmd.arg("install");
        cmd.arg("--debug");
        cmd.arg("--root");
        cmd.arg(&root);
        cmd.arg("--version");
        cmd.arg(CRATE_VERSION);
        cmd.arg("--no-default-features");
        cmd.arg("--features");
        cmd.arg(CRATE_FEATURES.join(" "));
        cmd.arg(CRATE_NAME);
        cmd.output().unwrap();
    }

    root.join("bin")
}
