use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
enum BudgetChatErr {
    #[error("network error")]
    NetworkIOError(#[from] Error),
    #[error("...")]
    InvalidMessage(String),
}


type User = String;

struct MessageHandler {
    user_connection: Arc<Mutex<HashMap<String, OwnedWriteHalf>>>,
}


impl<'a> MessageHandler {
    fn new() -> Self {
        MessageHandler {
            user_connection: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn read_loop(&self, conn: TcpStream) -> Result<(), BudgetChatErr> {
        let (buffer_reader, mut buffer_writer) = conn.into_split();

        let mut user = String::new();

        buffer_writer.write("Welcome to the crab land, what should I call you?\n".as_bytes()).await?;

        let mut reader = BufReader::new(buffer_reader);

        match reader.read_line(&mut user).await {
            Ok(0) | Err(_) => {
                return Ok(());
            }
            Ok(_) => {
                user = user.trim().to_string();
                if let Err(e) = self.validate_username(&user).await {
                    return Err(e);
                }

                {
                    let connection_arc = self.user_connection.clone();
                    let mut connection = connection_arc.lock().await;
                    connection.insert(user.clone(), buffer_writer);
                }

                let _ = self.send_room_details(&user).await;
                let _ = self.send_user_joins_message(&user).await;
            }
        }

        loop {
            let mut current_message = String::new();
            match reader.read_line(&mut current_message).await {
                Ok(0) | Err(_) => {
                    let _ = self.disconnect_user(&user).await;
                    let _ = self.send_user_left_message(&user).await;
                    return Ok(());
                }
                Ok(_) => {
                    let _ = self.send_user_message(&user, current_message.trim().to_string()).await;
                }
            }
        }
    }

    async fn send_user_message(&self, from: &User, message: String) -> Result<(), BudgetChatErr> {
        let message = format!("[{}] {}", from, message);
        self.send_all_except(from, message).await
    }

    async fn send_user_joins_message(&self, from: &User) -> Result<(), BudgetChatErr> {
        let message = format!("* {} has entered the room", from);
        self.send_all_except(from, message).await
    }

    async fn send_user_left_message(&self, from: &User) -> Result<(), BudgetChatErr> {
        let message = format!("* {} has left the room", from);
        self.send_all_except(from, message).await
    }

    async fn send_room_details(&self, from: &User) -> Result<(), BudgetChatErr> {
        let room_users: String;

        {
            let users_arc = self.user_connection.clone();
            let users = users_arc.lock().await;
            room_users = users.iter()
                .filter(|(k, _)| *k != from)
                .map(|(k, _)| k.clone())
                .collect::<Vec<String>>().join(", ");
        }

        let message = format!("* The room contains: {}", room_users);
        self.send_one(from, message).await
    }

    async fn send_one(&self, from: &User, mut message: String) -> Result<(), BudgetChatErr> {
        let connection_arc = self.user_connection.clone();
        let mut connection = connection_arc.lock().await;

        message.push_str("\n");

        if let Some(writer) = connection.get_mut(from) {
            writer.write_all(message.as_bytes()).await?;
        }

        Ok(())
    }

    async fn send_all_except(&self, from: &User, mut message: String) -> Result<(), BudgetChatErr> {
        let connection_arc = self.user_connection.clone();
        let mut connection = connection_arc.lock().await;


        message.push_str("\n");

        for (user, conn) in connection.iter_mut() {
            if from == user {
                continue;
            }
            conn.write_all(message.as_bytes()).await?;
        }
        Ok(())
    }

    async fn validate_username(&self, username: &str) -> Result<(), BudgetChatErr> {
        if username.len() == 0 || username.len() > 20 { return Err(BudgetChatErr::InvalidMessage(String::from("longer than expected username"))); }

        let connection_arc = self.user_connection.clone();
        let connection = connection_arc.lock().await;
        if connection.contains_key(username) { return Err(BudgetChatErr::InvalidMessage(String::from("username already exists"))); }

        Ok(())
    }

    async fn disconnect_user(&self, user: &User) -> Result<(), BudgetChatErr> {
        let connection_arc = self.user_connection.clone();
        let mut connection = connection_arc.lock().await;
        connection.remove(user);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), BudgetChatErr> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let server = Arc::new(MessageHandler::new());
    loop {
        let (socket, _) = listener.accept().await?;
        let handler_clone = server.clone();
        tokio::spawn(async move {
            if let Err(e) = handler_clone.read_loop(socket).await {
                println!("{:?}", e);
            };
        });
    }
}