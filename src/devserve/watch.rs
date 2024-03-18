use std::{
    path::{Path, PathBuf},
    time::Duration
};
use tokio::sync::mpsc::channel;
use tracing::{info, error};
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use anyhow::anyhow;

pub async fn watch<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<PathBuf>> {
    use notify::RecursiveMode::*;
    let rt = tokio::runtime::Handle::current();

    // Create a channel to receive the events.
    let (tx, mut rx) = channel(1);

    let mut debouncer = new_debouncer(
        Duration::from_secs(1),
        move |result: DebounceEventResult| {
            let tx: tokio::sync::mpsc::Sender<Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>> = tx.clone();

            info!("calling by notify -> {:?}", &result);
            rt.spawn(async move {
                if let Err(e) = tx.send(result).await {
                    error!("Error sending event result: {:?}", e);
                }
            });
        }).map_err(|e| anyhow!("Error while trying to watch for changes:\n\n\t{:?}", e))?;
    let watcher = debouncer.watcher();

    // Add the source directory to the watcher
    if let Err(e) = watcher.watch(path.as_ref(), Recursive) {
        anyhow::bail!("Error while watching {:?}:\n    {:?}", path.as_ref(), e);
    };

    info!("Listening for changes...");
    let result = tokio::spawn(async move {
       loop {
         if let Some(result) = rx.recv().await {
            break result
        }
       }
    }).await;
    let recv_result = result.or_else(|e| anyhow::bail!("listen failure: {:?}", e))?;
    let events = recv_result.or_else(|e| anyhow::bail!("listen recv failure: {:?}", e))?;
    info!("events: {:?}", events);
    Ok(events.iter().map(|e| e.path.clone()).collect())
}
