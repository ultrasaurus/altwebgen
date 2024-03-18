use std::{
    path::{Path, PathBuf},
    time::Duration
};
use tokio::sync::mpsc::channel;
use tracing::{info, warn};
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};

pub async fn watch<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<PathBuf>> {
    use notify::RecursiveMode::*;
    let rt = tokio::runtime::Handle::current();

    // Create a channel to receive the events.
    let (tx, mut rx) = channel(1);

    let mut debouncer = new_debouncer(
        Duration::from_secs(1),
        move |result: DebounceEventResult| {
            let tx: tokio::sync::mpsc::Sender<Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>> = tx.clone();

            println!("calling by notify -> {:?}", &result);
            rt.spawn(async move {
                if let Err(e) = tx.send(result).await {
                    println!("Error sending event result: {:?}", e);
                }
            });
        }).or_else(|e| {
            anyhow::bail!("Error while trying to watch for changes:\n\n\t{:?}", e);
        })?;
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
    let v:Vec<PathBuf> = Vec::new(); //events.iter().map(|e| e.path);

    Ok(v)

    // loop {
    //     let first_event = rx.recv().unwrap();
    //     sleep(Duration::from_millis(50));
    //     let other_events = rx.try_iter();

    //     let all_events = std::iter::once(first_event).chain(other_events);

    //     let paths: Vec<_> = all_events
    //         .filter_map(|event| match event {
    //             Ok(events) => Some(events),
    //             Err(error) => {
    //                 warn!("error while watching for changes: {error}");
    //                 None
    //             }
    //         })
    //         .flatten()
    //         .map(|event| event.path)
    //         .collect();

    //     if !paths.is_empty() {
    //         return Ok(paths);
    //     }
    // }

}
