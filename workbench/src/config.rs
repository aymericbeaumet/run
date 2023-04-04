use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    pub tmux: Tmux,

    #[serde(rename = "run")]
    pub runs: Vec<Run>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Sequential,
    Parallel,
    Tmux,
}

#[derive(Debug, Deserialize)]
pub struct Tmux {
    pub program: String,
    pub session_prefix: String,
    pub socket_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Run {
    pub cmd: Cmd,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Cmd {
    CmdString(String),
    CmdVec(Vec<String>),
}

impl Config {
    pub fn load<P: AsRef<Path>>(rel_path: Option<P>) -> anyhow::Result<(Config, PathBuf)> {
        let mut abs_path = std::env::current_dir()?;
        if let Some(rel_path) = rel_path {
            abs_path.push(rel_path);
        }
        if std::fs::metadata(&abs_path)?.is_dir() {
            abs_path.push("workbench.toml");
        }
        let abs_path = abs_path.canonicalize()?;

        let content = std::fs::read_to_string(&abs_path)?;
        let config = toml::from_str(&content)?;

        Ok((config, abs_path))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mode: Mode::Sequential,
            tmux: Tmux {
                program: "tmux".to_string(),
                session_prefix: "workbench-".to_string(),
                socket_path: "/tmp/tmux.workbench.sock".to_string(),
            },
            runs: vec![],
        }
    }
}

impl Cmd {
    pub fn parse(&self) -> anyhow::Result<(String, Vec<String>)> {
        let as_vec = match self {
            Cmd::CmdString(s) => shell_words::split(s)?
                .iter()
                .map(|word| word.to_string())
                .collect(),
            Cmd::CmdVec(v) => v.to_vec(),
        };
        Ok((as_vec[0].clone(), as_vec[1..].to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::Cmd::{CmdString, CmdVec};

    #[test]
    fn test_cmd_vec() {
        let tests = [
            (vec!["ls"], ("ls", vec![])),
            (vec!["ls", "-la"], ("ls", vec!["-la"])),
        ];

        for test in &tests {
            let cmd = CmdVec(test.0.iter().map(|s| s.to_string()).collect());
            let expected = (
                (test.1).0.to_string(),
                (test.1).1.iter().map(|s| s.to_string()).collect(),
            );
            let out = cmd
                .parse()
                .expect("implementation error: should not fail in these tests");
            assert_eq!(out, expected);
        }
    }

    #[test]
    fn test_cmd_string_parsing_ok() {
        let tests = [
            ("ls", ("ls", vec![])),
            ("ls -la", ("ls", vec!["-la"])),
            ("ls -la -- what?", ("ls", vec!["-la", "--", "what?"])),
            (" ls  -la ", ("ls", vec!["-la"])),
        ];

        for test in &tests {
            let cmd = CmdString(test.0.to_string());
            let expected = (
                (test.1).0.to_string(),
                (test.1).1.iter().map(|s| s.to_string()).collect(),
            );
            let out = cmd
                .parse()
                .expect("implementation error: should not fail in these tests");
            assert_eq!(out, expected);
        }
    }

    //#[test]
    //fn test_cmd_string_parsing_err() {
    //let tests = [
    //"missing single quote '",
    //"missing double quote \"",
    //"missing magic quote `",
    //"single back quote \\",
    //];

    //for test in &tests {
    //let cmd = CmdString(test.to_string());
    //let out = cmd.parse();
    //assert!(out.is_err());
    //}
    //}
}
