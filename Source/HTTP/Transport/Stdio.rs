// STDIO transport for the ConPort MCP server
use std::{
	io::{BufRead, BufReader, Write},
	sync::mpsc::{Receiver, Sender, channel},
	thread,
};

use serde_json::Value;

use crate::HTTP::Protocol::{
	Request::{Method, Request, RequestData, RequestId},
	Response::{Error, Response},
};

pub struct Stdio;

impl Stdio {
	pub fn New() -> Self { Stdio }

	pub fn Run<F>(&self, mut Handler:F)
	where
		F: FnMut(Request) -> Result<Value, Error> + Send + 'static, {
		let stdin = std::io::stdin();
		let stdout = std::io::stdout();
		let mut reader = BufReader::new(stdin.lock());
		let mut writer = stdout.lock();

		loop {
			let mut line = String::new();
			match reader.read_line(&mut line) {
				Ok(0) => break,
				Ok(_) => {
					let line = line.trim();
					if line.is_empty() {
						continue;
					}

					match self.ParseRequest(line) {
						Ok(request) => {
							let response = self.HandleRequest(&request, &mut Handler);
							if let Err(e) = self.SendResponse(&mut writer, &response) {
								tracing::error!("Failed to send response: {:?}", e);
							}
						},
						Err(error) => {
							let response = Response::ParseError(RequestId::default(), &error);
							if let Err(e) = self.SendResponse(&mut writer, &response) {
								tracing::error!("Failed to send response: {:?}", e);
							}
						},
					}
				},
				Err(e) => {
					tracing::error!("Failed to read from stdin: {:?}", e);
					break;
				},
			}
		}
	}

	fn ParseRequest(&self, line:&str) -> Result<Request, String> {
		serde_json::from_str(line).map_err(|e| format!("JSON parse error: {}", e))
	}

	fn HandleRequest<F>(&self, request:&Request, handler:&mut F) -> Response
	where
		F: FnMut(Request) -> Result<Value, Error> + Send + 'static, {
		let id = request.Id().clone();
		let method = request.Method().clone();

		// Handle special system methods
		match method {
			Method::Ping => {
				return Response::Success(id, serde_json::json!({ "pong": true }));
			},
			Method::Shutdown => {
				// For shutdown, return success then exit
				let response = Response::Success(id, serde_json::json!({ "shutdown": true }));
				println!("{}", serde_json::to_string(&response).unwrap_or_default());
				std::process::exit(0);
			},
			_ => {},
		}

		// Call the handler
		match handler(Request::JsonRpc(RequestData::WithId(method, id.clone()))) {
			Ok(result) => Response::Success(id, result),
			Err(error) => Response::Error(id, error),
		}
	}

	fn SendResponse<W:Write>(&self, writer:&mut W, response:&Response) -> Result<(), std::io::Error> {
		let json =
			serde_json::to_string(response).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

		writer.write_all(json.as_bytes())?;
		writer.write_all(b"\n")?;
		writer.flush()?;

		Ok(())
	}
}

impl Default for Stdio {
	fn default() -> Self { Self::New() }
}

// Async STDIO transport using channels
pub struct StdioAsync {
    RequestTx:Option<Sender<Request>>,
    #[allow(dead_code)]
    ResponseRx:Option<Receiver<Response>>,
}

impl StdioAsync {
	pub fn New() -> Self {
		let (request_tx, _request_rx) = channel::<Request>();
		let (response_tx, response_rx) = channel::<Response>();

		// Spawn the read loop in a separate thread
		let stdio_request_tx = request_tx.clone();
		let stdio_response_tx = response_tx;

		thread::spawn(move || {
			Self::ReadLoop(stdio_request_tx, stdio_response_tx);
		});

		Self { RequestTx:Some(request_tx), ResponseRx:Some(response_rx) }
	}

	fn ReadLoop(request_tx:Sender<Request>, response_tx:Sender<Response>) {
		let stdin = std::io::stdin();
		let reader = BufReader::new(stdin.lock());

		for line_result in reader.lines() {
			match line_result {
				Ok(line) => {
					let line = line.trim();
					if line.is_empty() {
						continue;
					}

					match serde_json::from_str::<Request>(line) {
						Ok(request) => {
							if let Err(e) = request_tx.send(request) {
								tracing::error!("Failed to send request: {:?}", e);
								break;
							}
						},
						Err(e) => {
							let _error = Error::ParseError(&format!("JSON parse error: {}", e));
							let response =
								Response::ParseError(RequestId::default(), &format!("JSON parse error: {}", e));
							if let Err(e) = response_tx.send(response) {
								tracing::error!("Failed to send error response: {:?}", e);
								break;
							}
						},
					}
				},
				Err(e) => {
					tracing::error!("Failed to read line: {:?}", e);
					break;
				},
			}
		}
	}

	pub fn ReceiveRequest(&self) -> Option<Request> {
		self.RequestTx.as_ref().and_then(|_tx| {
			// This is a simplified version - in practice you'd want proper async
			None
		})
	}

	pub fn SendResponse(&self, response:Response) -> Result<(), String> {
		// In the async version, responses would be sent through a separate channel
		// For now, we'll just print them directly
		let stdout = std::io::stdout();
		let mut writer = stdout.lock();
		let json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
		writer.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
		writer.write_all(b"\n").map_err(|e| e.to_string())?;
		writer.flush().map_err(|e| e.to_string())?;
		Ok(())
	}
}

impl Default for StdioAsync {
	fn default() -> Self { Self::New() }
}
