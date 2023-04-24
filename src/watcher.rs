use std::{path::Path, sync::Arc};
use tokio::sync::mpsc;
use watchexec::{
    action::Action,
    config::{InitConfig, RuntimeConfig},
    Watchexec,
};

pub struct Watcher {
    we: Arc<Watchexec>,
}

impl Watcher {
    pub fn new<I, P>(tx: mpsc::Sender<()>, roots: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let init = InitConfig::default();

        let mut runtime = RuntimeConfig::default();
        runtime.pathset(roots);
        runtime.on_action(move |_: Action| {
            let tx = tx.clone();
            async move {
                tx.send(()).await.unwrap();
                Ok::<(), std::io::Error>(())
            }
        });

        Ok(Self {
            we: Watchexec::new(init, runtime)?,
        })
    }

    pub async fn watch(self) -> anyhow::Result<()> {
        let handle = self.we.main().await?;
        Ok(handle?)
    }
}
