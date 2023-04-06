use anyhow::bail;
use anyhow::Context;
use glob::glob;
use pretty_assertions::StrComparison;
use std::{path::Path, process::Output};
use tokio::process::Command;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

const PATTERNS: [&str; 2] = [
    // try to match all workbench.toml files in the examples directory
    "examples/*/workbench.toml",
    // run the workbench.toml files in the tests directory
    "tests/**/*.toml",
];

#[tokio::test(flavor = "multi_thread")]
async fn test_all() -> anyhow::Result<()> {
    let mut set = tokio::task::JoinSet::new();

    PATTERNS
        .iter()
        .map(|pattern| format!("{}/{}", ROOT, pattern))
        .flat_map(|pattern| glob(&pattern).unwrap().map(|entry| entry.unwrap()))
        .for_each(|file| {
            set.spawn(async move {
                let test_name = &file.strip_prefix(ROOT)?.with_extension("");

                test_one(&file)
                    .await
                    .with_context(|| format!("test failed: {:?}", test_name))?;

                println!("[ok] {:?}", &test_name);
                Ok::<(), anyhow::Error>(())
            });
        });

    while let Some(Ok(joined)) = set.join_next().await {
        joined?;
    }

    Ok(())
}

async fn test_one<P: AsRef<Path>>(file: P) -> anyhow::Result<()> {
    if read_file(&file, ".skip").await.is_some() {
        return Ok(());
    }

    let expected_stdout = read_file(&file, ".stdout").await;
    let expected_stderr = read_file(&file, ".stderr").await;

    if expected_stdout.is_none() && expected_stderr.is_none() {
        bail!("none of .stdout or .stderr found");
    }

    // run and get output
    let output = workbench(Some(&file)).await?;
    let stdout = std::str::from_utf8(&output.stdout)?;
    let stderr = std::str::from_utf8(&output.stderr)?;

    if !output.status.success() && expected_stderr.is_none() {
        bail!("unexpectedly failed with: {}", stderr);
    }

    // assert stdout
    if let Some(expected) = expected_stdout {
        if expected != stdout {
            bail!(format!(
                "stdout does not match: {}",
                StrComparison::new(&expected, stdout)
            ));
        }
    }

    // assert stderr
    if let Some(expected) = expected_stderr {
        if expected != stderr {
            bail!(format!(
                "stderr does not match: {}",
                StrComparison::new(&expected, stderr)
            ));
        }
    }

    Ok(())
}

async fn workbench<P>(file: Option<P>) -> anyhow::Result<Output>
where
    P: AsRef<Path>,
{
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_workbench"));

    if let Some(file) = file {
        cmd.arg("-f");
        cmd.arg(file.as_ref());
    }

    Ok(cmd.output().await?)
}

async fn read_file<P: AsRef<Path>>(filepath: P, suffix: &str) -> Option<String> {
    let filepath = filepath.as_ref().to_str()?.to_string() + suffix;
    tokio::fs::read_to_string(&filepath).await.ok()
}
