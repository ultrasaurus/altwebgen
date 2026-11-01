use anyhow::anyhow;
use notify::RecommendedWatcher;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{error, info};

// Watches paths for as long as it lives, so changes made while a build is
// running are queued and reported by the next call to `next_changes`
pub struct ChangeWatcher {
    _debouncer: Debouncer<RecommendedWatcher>,
    rx: UnboundedReceiver<DebounceEventResult>,
}

impl ChangeWatcher {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> anyhow::Result<Self> {
        use notify::RecursiveMode::*;

        // unbounded, so the notify thread never blocks or drops events
        let (tx, rx) = unbounded_channel();

        let mut debouncer = new_debouncer(
            Duration::from_millis(200),
            move |result: DebounceEventResult| {
                info!("calling by notify -> {:?}", &result);
                if let Err(e) = tx.send(result) {
                    error!("Error sending event result: {:?}", e);
                }
            },
        )
        .map_err(|e| anyhow!("Error while trying to watch for changes:\n\n\t{:?}", e))?;
        let watcher = debouncer.watcher();

        // Add the source directories to the watcher
        for path in paths {
            if let Err(e) = watcher.watch(path.as_ref(), Recursive) {
                anyhow::bail!("Error while watching {:?}:\n    {:?}", path.as_ref(), e);
            }
        }

        Ok(Self {
            _debouncer: debouncer,
            rx,
        })
    }

    // Waits for changes, then also takes any already queued so that
    // a burst of changes results in one build (cancel safe)
    pub async fn next_changes(&mut self) -> anyhow::Result<Vec<PathBuf>> {
        let first = self
            .rx
            .recv()
            .await
            .ok_or_else(|| anyhow!("watcher channel closed"))?;
        let mut paths = Vec::new();
        let mut collect = |result: DebounceEventResult| match result {
            Ok(events) => paths.extend(events.into_iter().map(|e| e.path)),
            Err(e) => error!("watch error: {:?}", e),
        };
        collect(first);
        while let Ok(result) = self.rx.try_recv() {
            collect(result);
        }
        info!("changed paths: {:?}", paths);
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    // a new folder + file created while a build runs (between calls to
    // next_changes) must still be reported
    async fn change_during_build_is_not_lost() {
        let dir = std::env::temp_dir().join(format!("awg-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut watcher = ChangeWatcher::new(&[&dir]).unwrap();

        std::fs::write(dir.join("a.md"), "a").unwrap();
        let first = tokio::time::timeout(Duration::from_secs(4), watcher.next_changes()).await;
        assert!(matches!(first, Ok(Ok(ref p)) if !p.is_empty()));

        // simulated build: changes happen while nobody is awaiting next_changes
        std::fs::create_dir_all(dir.join("category/general")).unwrap();
        std::fs::write(dir.join("category/general/index.md"), "b").unwrap();
        tokio::time::sleep(Duration::from_millis(1500)).await;

        let second = tokio::time::timeout(Duration::from_secs(4), watcher.next_changes()).await;
        // events use the real path (macOS: /private/var/... for /var/...)
        let category = dir.canonicalize().unwrap().join("category");
        std::fs::remove_dir_all(&dir).unwrap();
        let second = second
            .expect("change made during build was never reported")
            .unwrap();
        // macOS reports the new file; Linux (inotify) only starts watching a
        // new folder once it's reported, so a file written right away may
        // show up only as the folder. Either one triggers the rebuild.
        assert!(
            second.iter().any(|p| p.starts_with(&category)),
            "missing new folder or file in {:?}",
            second
        );
    }
}
