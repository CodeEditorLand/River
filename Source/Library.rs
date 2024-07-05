use echo::{Action, ActionResult, Job, WorkQueue, Worker, Yell};

use futures::future::join_all;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_tungstenite::accept_async;

struct Worker;

#[async_trait::async_trait]
impl Worker for Worker {
	async fn Receive(&self, Action: Action) -> ActionResult {
		Box::pin(async move {
			match Action {
				Action::Read { path } => match tokio::fs::read_to_string(&path).await {
					Ok(Content) => ActionResult { Action, result: Ok(Content) },
					Err(Error) => {
						ActionResult { Action, result: Err(format!("Cannot Action: {}", Error)) }
					}
				},
				_ => ActionResult { Action, result: Err("Cannot Action.".to_string()) },
			}
		})
	}
}

#[tokio::main]
async fn main() {
	let Work = Arc::new(WorkQueue::new());
	let (Acceptance, Receipt) = mpsc::channel(100);

	// @TODO: Auto-calc number of workers in the force
	let Force: Vec<_> = (0..4)
		.map(|_| {
			tokio::spawn(Job(Arc::new(Worker) as Arc<dyn Worker>, Work.clone(), Acceptance.clone()))
		})
		.collect();

	while let Ok((stream, _)) =
		TcpListener::bind("127.0.0.1:9999").await.expect("Cannot TcpListener.").accept().await
	{
		tokio::spawn(Yell(
			accept_async(stream).await.expect("Cannot accept_async."),
			Work.clone(),
			Receipt.clone(),
		));
	}

	join_all(Force).await;
}
