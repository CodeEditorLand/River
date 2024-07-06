#![allow(non_snake_case)]

use Echo::Fn::Job::{Action, ActionResult, Work, Worker};

use std::sync::Arc;

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
	let (Approval, Receipt) = tokio::sync::mpsc::unbounded_channel();
	let Receipt = Arc::new(tokio::sync::Mutex::new(Receipt));

	// @TODO: Auto-calc number of workers on the force
	let Force: Vec<_> = (0..4)
		.map(|_| tokio::spawn(Echo::Fn::Job::Fn(Arc::new(Site), Work.clone(), Approval.clone())))
		.collect();

	while let Ok((stream, _)) = tokio::net::TcpListener::bind("127.0.0.1:9998")
		.await
		.expect("Cannot TcpListener.")
		.accept()
		.await
	{
		tokio::spawn(Echo::Fn::Job::Yell::Fn(
			tokio_tungstenite::accept_async(stream).await.expect("Cannot accept_async."),
			Work.clone(),
			Arc::clone(&Receipt),
		));
	}

	futures::future::join_all(Force).await;
}
