use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use crate::utils::CrabbyError;

pub struct NetworkHandler {
    listeners: Vec<TcpListener>,
    connections: Vec<TcpStream>,
    event_sender: Sender<NetworkEvent>,
    event_receiver: Receiver<NetworkEvent>,
}

impl Clone for NetworkHandler {
    fn clone(&self) -> Self {
        let (tx, rx) = channel();
        NetworkHandler {
            listeners: Vec::new(),
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
        let (tx, rx) = channel();
        Self {
            listeners: Vec::new(),
            connections: Vec::new(),
            event_sender: tx,
            event_receiver: rx,
        }
    }

    pub fn listen(&mut self, addr: &str) -> Result<(), CrabbyError> {
        let listener = TcpListener::bind(&addr)
            .map_err(|e| CrabbyError::NetworkError(format!("Failed to bind: {}", e)))?;

        self.listeners.push(listener.try_clone()?);

        let tx = self.event_sender.clone();
        let addr = addr.to_string();

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        tx.send(NetworkEvent::Connected(addr.clone())).unwrap();
                    }
                    Err(e) => {
                        tx.send(NetworkEvent::Error(e.to_string())).unwrap();
                    }
                }
            }
        });

        Ok(())
    }

    pub fn connect(&mut self, address: &str, port: u16) -> Result<(), CrabbyError> {
        let addr = format!("{}:{}", address, port);
        let stream = TcpStream::connect(&addr)
            .map_err(|e| CrabbyError::NetworkError(format!("Failed to connect: {}", e)))?;

        self.connections.push(stream);
        Ok(())
    }

    pub fn send(&mut self, data: &[u8], conn_index: usize) -> Result<(), CrabbyError> {
        if let Some(stream) = self.connections.get_mut(conn_index) {
            stream.write_all(data)
                .map_err(|e| CrabbyError::NetworkError(format!("Failed to send: {}", e)))?;
            Ok(())
        } else {
            Err(CrabbyError::NetworkError("Invalid connection index".to_string()))
        }
    }

    pub fn receive(&self) -> Result<NetworkEvent, CrabbyError> {
        self.event_receiver.recv()
            .map_err(|e| CrabbyError::NetworkError(format!("Failed to receive: {}", e)))
    }

    pub fn bind(&mut self, address: &str, port: u16) -> Result<(), CrabbyError> {
        let addr = format!("{}:{}", address, port);
        let listener = TcpListener::bind(&addr)
            .map_err(|e| CrabbyError::NetworkError(format!("Failed to bind: {}", e)))?;
        self.listeners.push(listener);
        Ok(())
    }
}
