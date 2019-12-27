use async_std::fs::remove_file;
use async_std::io::Result;
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::path::Path;
use async_std::prelude::StreamExt;
use async_std::task;
use crate::models::Module;

pub async fn listen(socket_path: &Path) -> Result<()> {
	// TODO Try to aquire a lock on the lock file first.
	// If lock fails, return an error

	// File lock is aquired. If the unix socket exists, then it's clearly a dangling socket. Feel free to delete it
	if socket_path.exists().await {
		remove_file(socket_path).await?;
	}

	let socket_server = UnixListener::bind(socket_path).await?;
	let mut incoming = socket_server.incoming();

	while let Some(stream) = incoming.next().await {
		task::spawn(async {
			handle_client(stream).await;
		});
	}

	Ok(())
}

async fn handle_client(stream: Result<UnixStream>) {
	if let Err(_) = stream {
		println!("Error occured while opening socket");
		return;
	}

	let stream = stream.unwrap();

	let module = Module::new(stream);
	let read_future = module.read_data_loop();
	let write_future = module.write_data_loop();

	futures::future::join(read_future, write_future);

	println!("Closing socket");
}
