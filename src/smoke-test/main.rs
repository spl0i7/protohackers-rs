use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buffer_until_eof = Vec::new();
    loop {
        let mut buf = [0 as u8; 1024];
        match socket.read(&mut buf).await {
            Ok(0) => break,
            Err(_) => break,
            Ok(n) => {
                buffer_until_eof.extend_from_slice(&buf[0..n]);
            }
        }
    }

    let _ = socket.write_all(&buffer_until_eof).await;
}