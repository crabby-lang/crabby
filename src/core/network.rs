use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::utils::CrabbyError;

pub struct NetworkHandler {
    listener: Option<Arc<Mutex<TcpListener>>>,
    connections: Vec<Arc<Mutex<TcpStream>>>,
    event_sender: Sender<NetworkEvent>,
    event_receiver: Receiver<NetworkEvent>,
}

impl Clone for NetworkHandler {
    fn clone(&self) -> Self {
        let (tx, rx) = channel(100);
        NetworkHandler {
            listener: None,
            connections: Vec::new(), 
            event_sender: tx,
            event_receiver: rx,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Connected(String),
    Received(Vec<u8>),
    Error(String),
}

impl NetworkHandler {
    pub fn new() -> Self {
        let (tx, rx) = channel(100);
        Self {
            listener: None,
            connections: Vec::new(),
            event_sender: tx,
            event_receiver: rx,
        }
    }

    pub async fn listen(&mut self, addr: &str, port: u16) -> Result<(), CrabbyError> {
        let addr = format!("{}:{}", addr, port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| CrabbyError::NetworkError(e.to_string()))?;
        self.listener = Some(Arc::new(Mutex::new(listener)));
        Ok(())
    }

    pub async fn accept(&mut self) -> Result<(), CrabbyError> {
        if let Some(listener) = &self.listener {
            let (stream, _) = listener.lock().await
                .accept().await
                .map_err(|e| CrabbyError::NetworkError(e.to_string()))?;
            self.connections.push(Arc::new(Mutex::new(stream)));
        }
        Ok(())
    }

    pub async fn connect(&mut self, addr: &str, port: u16) -> Result<(), CrabbyError> {
        let addr = format!("{}:{}", addr, port);
        let stream = TcpStream::connect(&addr).await
            .map_err(|e| CrabbyError::NetworkError(e.to_string()))?;
        self.connections.push(Arc::new(Mutex::new(stream)));
        Ok(())
    }

    pub async fn send(&mut self, data: &[u8], conn_index: usize) -> Result<(), CrabbyError> {
        if let Some(conn) = self.connections.get(conn_index) {
            conn.lock().await
                .write_all(data).await
                .map_err(|e| CrabbyError::NetworkError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn receive(&mut self, conn_index: usize) -> Result<Vec<u8>, CrabbyError> {
        if let Some(conn) = self.connections.get(conn_index) {
            let mut buffer = Vec::new();
            conn.lock().await
                .read_to_end(&mut buffer).await
                .map_err(|e| CrabbyError::NetworkError(e.to_string()))?;
            Ok(buffer)
        } else {
            Err(CrabbyError::NetworkError("Invalid connection index".to_string()))
        }
    }
}
