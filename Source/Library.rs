// src/bin/file_reader.rs
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{fs::File, io::AsyncReadExt, net::TcpListener};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Serialize, Deserialize)]
struct FileRequest {
	path: String,
}

#[derive(Serialize, Deserialize)]
struct FileResponse {
	path: String,
	content: String,
}

#[tokio::main]
async fn main() {
	let cache = Arc::new(DashMap::new());
	let listener = TcpListener::bind("127.0.0.1:9999").await.unwrap();

	while let Ok((stream, _)) = listener.accept().await {
		let cache = cache.clone();
		tokio::spawn(handle_connection(stream, cache));
	}
}

async fn handle_connection(stream: tokio::net::TcpStream, cache: Arc<DashMap<String, String>>) {
	let ws_stream = accept_async(stream).await.unwrap();
	let (write, read) = ws_stream.split();

	read.for_each(|message| async {
		if let Ok(msg) = message {
			if msg.is_text() {
				let file_request: FileRequest =
					serde_json::from_str(msg.to_text().unwrap()).unwrap();
				let response = if let Some(content) = cache.get(&file_request.path) {
					FileResponse { path: file_request.path.clone(), content: content.clone() }
				} else {
					let mut file = File::open(&file_request.path).await.unwrap();
					let mut content = String::new();
					file.read_to_string(&mut content).await.unwrap();
					cache.insert(file_request.path.clone(), content.clone());
					FileResponse { path: file_request.path, content }
				};
				let response_msg = Message::text(serde_json::to_string(&response).unwrap());
				write.send(response_msg).await.unwrap();
			}
		}
	})
	.await;
}
