use futures::{StreamExt, FutureExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::Filter;

struct Client {
    name: String,
    sender: Option<mpsc::UnboundedSender<Result<warp::ws::Message, warp::Error>>>,
}

impl Client {
    fn new(name: String) -> Self {
        Client {
            name: name,
            sender: None,
        }
    }
}

type Clients = Arc<RwLock<HashMap<Uuid, Client>>>;

#[derive(Deserialize, Debug)]
struct RegisterRequest {
    name: String,
}

#[derive(Serialize, Debug)]
struct RegisterResponse {
    uuid: Uuid,
}

async fn register_client(rreq: RegisterRequest, clients: Clients) -> std::result::Result<impl warp::Reply, warp::reject::Rejection>  {
    println!("register_client!");
    let uuid = Uuid::new_v4();
    clients.write().await.insert(uuid, Client::new(rreq.name));
    Ok(warp::reply::json(&RegisterResponse { uuid }))
}

async fn broadcast_message(id: &Uuid, msg: warp::ws::Message, clients: &Clients) {
    let msg_txt = match msg.to_str() {
        Ok(m) => m,
        Err(_) => return,
    };

    let client = clients.read().await;
    let client = client.get(id);
    if client.is_none() {
        return;
    }
    let client = client.unwrap();
    let msg_txt = format!("<{}> {}", client.name, msg_txt);

    for (&uid, client) in clients.read().await.iter() {
        if uid != *id {
            if let Some(sender) = &client.sender {
                if let Err(_) = sender.send(Ok(warp::ws::Message::text(msg_txt.clone()))) {
                    // disconnect should be handled in client_connected.  Just ignore here.
                }
            }
        }
    }
}

async fn client_connected(socket: warp::ws::WebSocket, id: Uuid, clients: Clients) {
    println!("client connected");
    let (ws_tx, mut ws_rx) = socket.split();

    let (tx, rx) = mpsc::unbounded_channel();

    {
        let mut client = clients.write().await;
        let client = client.get_mut(&id);
        if client.is_none() {
            return;
        }
        let mut client = client.unwrap();
        client.sender = Some(tx);
    }

    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => {
                println!("Message({:#?}): {:#?}", id, msg);
                msg
            },
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", id, e);
                break;
            }
        };
        broadcast_message(&id, msg, &clients).await;
    }

    client_disconnected(&id, &clients).await;
}

async fn client_disconnected(id: &Uuid, clients: &Clients) {
    eprintln!("Disconnected: {}", id);

    clients.write().await.remove(id);
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    println!("with_clients!");
    warp::any().map(move || clients.clone())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| warp::reply::html(format!("<em>Hello, {}!</em>", name)));

    let index = warp::get().and(warp::path::end()).and(warp::fs::file("../../webdata/index.html"));

    let webdata_files = warp::get().and(warp::path("webdata")).and(warp::fs::dir("../../webdata/"));
    let pkg_files = warp::get().and(warp::path("pkg")).and(warp::fs::dir("../../pkg/"));

    let clients = Clients::default();
    let registration = warp::post()
        .and(warp::path("register"))
        .and(warp::body::content_length_limit(100))
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(register_client);
    let chat = warp::get()
        .and(warp::path("chat"))
        .and(warp::ws())  // WebSocket Handshake
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .map(|ws: warp::ws::Ws, id, clients| {
            ws.on_upgrade(move |socket| client_connected(socket, id, clients))
        });

    let routes = hello.or(webdata_files.or(pkg_files.or(index.or(registration.or(chat)))));

    println!("Starting server: http://127.0.0.1:3030/");

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
