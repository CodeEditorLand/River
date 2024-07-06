use Echo::Fn::Job::{Action, ActionResult, Fn as Job, Work, Worker, Yell::Fn as Yell};

use futures::future::join_all;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_tungstenite::accept_async;

struct Site;

#[async_trait::async_trait]
impl Worker for Site {
	async fn Receive(&self, Action: Action) -> ActionResult {
		match Action {
			Action::Read { ref Path } => match tokio::fs::read_to_string(&Path).await {
				Ok(Content) => ActionResult { Action, Result: Ok(Content) },
				Err(Error) => {
					ActionResult { Action, Result: Err(format!("Cannot Action: {}", Error)) }
				}
			},
			_ => ActionResult { Action, Result: Err("Cannot Action.".to_string()) },
		}
	}
}

#[tokio::main]
async fn main() {
	let Work = Arc::new(Work::Begin());
	let (Approval, Receipt) = mpsc::unbounded_channel();

	// @TODO: Auto-calc number of workers on the force
	let Force: Vec<_> = (0..4).map(|_| tokio::spawn(Job(Arc::new(Site), Work, Approval))).collect();

	while let Ok((stream, _)) =
		TcpListener::bind("127.0.0.1:9998").await.expect("Cannot TcpListener.").accept().await
	{
		tokio::spawn(Yell(
			accept_async(stream).await.expect("Cannot accept_async."),
			Work.clone(),
			Receipt,
		));
	}

	join_all(Force).await;
}
