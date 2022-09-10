use tokio::net::{TcpListener, TcpStream};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug)]
struct PrimeTimeErr {}

impl From<std::io::Error> for PrimeTimeErr {
    fn from(_: std::io::Error) -> Self { PrimeTimeErr {} }
}

impl From<serde_json::Error> for PrimeTimeErr {
    fn from(_: serde_json::Error) -> Self { PrimeTimeErr {} }
}

#[derive(Deserialize, Debug)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Serialize, Debug)]
struct Response {
    method: String,
    prime: bool,
}

impl Response {
    fn new() -> Self {
        return Response {
            method: "isPrime".to_string(),
            prime: false,
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), PrimeTimeErr> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), PrimeTimeErr> {
    let mut reader = BufReader::new(&mut socket);
    loop {
        let mut current_line = String::new();
        let line = reader.read_line(&mut current_line).await?;

        if line == 0 {
            return Ok(());
        }

        if let Ok(req) = serde_json::from_str::<Request>(current_line.as_str()) {
            if req.method != "isPrime" {
                let _ = reader.get_mut().write("{,\n".as_ref()).await;
                return Ok(());
            }

            let mut resp = Response::new();
            if req.number.fract() == 0.00 {
                resp.prime = prime_test(req.number as i64);
            }

            let mut resp = serde_json::to_string(&resp)?;
            resp.push('\n');

            let _ = reader.get_mut().write(resp.as_bytes()).await;
        } else {
            let _ = reader.get_mut().write("{,\n".as_ref()).await;
        }
    }
}


fn prime_test(n: i64) -> bool {
    if n <= 1 { return false; }
    let limit = (n as f64).sqrt() as i64 + 1;
    !(2..limit).any(|x| n % x == 0)
}