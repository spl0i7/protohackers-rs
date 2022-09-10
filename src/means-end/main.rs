use std::collections::BTreeMap;
use std::ops::Bound::Included;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[derive(Debug)]
struct MeansEndErr {}

impl From<std::io::Error> for MeansEndErr {
    fn from(_: std::io::Error) -> Self { MeansEndErr {} }
}

impl From<serde_json::Error> for MeansEndErr {
    fn from(_: serde_json::Error) -> Self { MeansEndErr {} }
}

struct Request {
    query: u8,
    arg1: i32,
    arg2: i32,
}

impl Request {
    fn from_bytes(raw: [u8; 9]) -> Self {
        let query = raw[0];

        Request {
            query,
            arg1: i32::from_be_bytes(Request::_to_arr(&raw[1..5])),
            arg2: i32::from_be_bytes(Request::_to_arr(&raw[5..9])),
        }
    }


    fn _to_arr(array: &[u8]) -> [u8; 4] {
        [array[0], array[1], array[2], array[3]]
    }
}

#[tokio::main]
async fn main() -> Result<(), MeansEndErr> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), MeansEndErr> {
    let mut storage = BTreeMap::new();
    let mut reader = BufReader::new(&mut socket);
    loop {
        let mut buffer = [0 as u8; 9];
        let line = reader.read_exact(&mut buffer).await?;

        if line == 0 {
            return Ok(());
        }

        let req = Request::from_bytes(buffer);


        match req.query {
            73 => storage.insert(req.arg1, req.arg2),
            81 => {
                let mut avg = 0;

                if req.arg1 <= req.arg2 {
                    let (count, sum) = storage.range((Included(req.arg1), Included(req.arg2)))
                        .into_iter()
                        .map(|x| *x.1 as i64)
                        .fold((0, 0), |mut x, y| {
                            x.0 += 1;
                            x.1 += y;
                            x
                        });

                    if count > 0 {
                        avg = (sum / count) as i32;
                    }
                }

                let result: [u8; 4] = i32::to_be_bytes(avg);

                let _ = reader.get_mut().write(&result).await;
            }

            _ => {}
        }
    }
}