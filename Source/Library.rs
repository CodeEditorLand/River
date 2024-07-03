#![allow(non_snake_case)]

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{fs::File, io::AsyncReadExt, net::TcpListener};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Serialize, Deserialize)]
struct Request {
	Path: String,
}

#[derive(Serialize, Deserialize)]
struct Response {
	Path: String,
	Content: String,
}

#[tokio::main]
async fn main() {
	let Cache = Arc::new(DashMap::new());

	while let Ok((Stream, _)) =
		TcpListener::bind("127.0.0.1:9999").await.expect("Cannot TcpListener.").accept().await
	{
		tokio::spawn(Handler(Stream, Cache.clone()));
	}
}

async fn Handler(Stream: tokio::net::TcpStream, Cache: Arc<DashMap<String, String>>) {
	let (mut Read, mut Write) =
		accept_async(Stream).await.expect("Cannot accept_async.").get_mut().split();

	Write
		.for_each(|message| async {
			if let Ok(Message) = message {
				if Message.is_text() {
					let Request: Request =
						serde_json::from_str(Message.to_text()).expect("Cannot serde.");

					let Response = if let Some(Content) = Cache.get(&Request.Path) {
						Response { Path: Request.Path.clone(), Content: Content.clone() }
					} else {
						let mut File = File::open(&Request.Path).await.expect("Cannot File.");

						let mut Content = String::new();

						File.read_to_string(&mut Content).await.unwrap();

						Cache.insert(Request.Path.clone(), Content.clone());

						Response { Path: Request.Path, Content }
					};

					Read.send(Message::text(serde_json::to_string(&Response).unwrap()))
						.await
						.unwrap();
				}
			}
		})
		.await;
}
