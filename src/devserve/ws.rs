use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tracing::info;
use warp::ws::Message;
use warp::Filter;

pub fn ws_receiver(endpoint: &str, from_server_tx: broadcast::Sender<Message>)
-> impl Filter<Extract = (impl warp::Reply + '_,), Error = warp::Rejection> + Clone  + '_ {

    // A warp Filter which captures `from_server_tx`
    // and provides an `rx` copy to receive messages from other server code
    let recv_from_server = warp::any().map(move || from_server_tx.subscribe());

    // A warp Filter to handle the messaging endpoint. This upgrades to a
    // websocket, and then waits for from_server_tx to send a message, which
    // relays them over the websocket.
    warp::path(endpoint)
        .and(warp::ws())
        .and(recv_from_server)
        .map(|ws: warp::ws::Ws, mut rx: broadcast::Receiver<Message>| {
            ws.on_upgrade(move |ws| async move {
                let (mut user_ws_tx, _user_ws_rx) = ws.split();
                info!("websocket got connection");
                if let Ok(m) = rx.recv().await {
                    info!("message to client javasscript: {:?}", m);
                    let _ = user_ws_tx.send(m).await;
                }
            })
        })
}
