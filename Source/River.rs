#![allow(non_snake_case)]

use Echo::Fn::Job::{Action, ActionResult, Work, Worker};

use std::sync::Arc;

/// Represents a working site.
struct Site;

/// Implements the `Worker` trait for the `Site` struct, allowing it to process actions.
///
/// This implementation handles the `Read` action by attempting to read the content of the specified file path asynchronously.
/// If successful, it returns an `ActionResult` with the content. If an error occurs, it returns an `ActionResult` with an error message.
/// For any other action, it returns a generic error message indicating that the action cannot be processed.
///
#[async_trait::async_trait]
impl Worker for Site {
	/// # Arguments
	///
	/// * `Action` - The action to be processed. This can be a `Read` action with a specified file path.
	///
	/// # Returns
	///
	/// An `ActionResult` containing the result of the action. If the action is a `Read` action and the file is read successfully,
	/// the result will contain the file content. If an error occurs during the read operation, the result will contain an error message.
	/// For any other action, the result will contain a generic error message.
	///
	/// This implementation ensures that the `Site` struct can handle `Read` actions by reading the file content asynchronously and returning the result.
	/// For any other actions, it returns a generic error message indicating that the action cannot be processed.
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

/// The main entry point of the application.
///
/// This function initializes the work queue, sets up a channel for approval messages,
/// spawns worker tasks to process actions, and listens for incoming TCP connections
/// to handle WebSocket communication.
///
/// # Behavior
///
/// 1. Initializes the work queue.
/// 2. Creates an unbounded channel for approval messages.
/// 3. Spawns worker tasks to process actions from the work queue.
/// 4. Binds to a TCP listener and accepts incoming connections.
/// 5. For each accepted connection, spawns a task to handle the WebSocket connection.
/// 6. Waits for all worker tasks to complete.
///
/// # Panics
///
/// This function will panic if:
/// - The TCP listener cannot be bound to the specified address.
/// - A WebSocket connection cannot be accepted.
///
/// # TODO
///
/// - Auto-calculate the number of workers.
///
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
