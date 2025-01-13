use std::net::Shutdown;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

type OnMessageReceivedCallback = Arc<Mutex<Box<dyn Fn(&[u8]) + Send>>>;

pub struct TcpClientData {
    socket: TcpStream,
    on_message_received: OnMessageReceivedCallback,
    sender: Sender<bool>,
    receiver: Mutex<Receiver<bool>>,
}

pub struct TcpClient {
    data: Arc<TcpClientData>,
    nonblocking: bool,
}

impl TcpClientData {
    fn new(address: &str) -> Result<Self, String> {
        let socket_result = TcpStream::connect(address);

        match socket_result {
            Ok(socket) => {
                if socket.set_nonblocking(true).is_err() {
                    return Err("Couldn't set socket to nonblocking mode".to_string());
                };

                let (sender, receiver) = channel::<bool>();

                Ok(Self {
                    socket,
                    on_message_received: Arc::new(Mutex::new(Box::new(|_| {}))),
                    sender,
                    receiver: Mutex::new(receiver),
                })
            }
            Err(e) => Err(format!("Error on connection: {e}")),
        }
    }
}

impl TcpClient {
    pub fn connect(address: &str) -> Result<Self, String> {
        let data = TcpClientData::new(address);

        match data {
            Ok(data) => Ok(Self {
                data: Arc::new(data),
                nonblocking: true,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn disconnect(&self) -> bool {
        let result = self.data.socket.shutdown(Shutdown::Both);
        let data_ref = self.data.clone();

        if result.is_ok() {
            let _ = data_ref.sender.send(true);
            true
        } else {
            false
        }
    }

    pub fn send(&self, data: &[u8]) -> Result<(), String> {
        let data_ref = self.data.clone();
        let mut socket = &data_ref.socket;

        let header = (data.len() as u64).to_le_bytes();
        let mut header_written: usize = 0;
        let mut body_written: usize = 0;

        while header_written < 8 {
            match socket.write(&header[header_written..]) {
                Ok(size) => {
                    if size > 0 {
                        header_written += size;
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        thread::sleep(Duration::from_millis(50));
                    } else {
                        return Err(e.kind().to_string());
                    }
                }
            }
        }

        while body_written < data.len() {
            match socket.write(&data[body_written..]) {
                Ok(size) => {
                    if size > 0 {
                        body_written += size;
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        thread::sleep(Duration::from_millis(50));
                    } else {
                        return Err(e.kind().to_string());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn receive(&self) {
        let data_ref = self.data.clone();

        thread::spawn(move || {
            let mut socket = &data_ref.socket;
            let mut buffer: Vec<u8> = vec![0; 8];
            let mut read_bytes: usize = 0;
            let mut amount_to_read: usize = 0;
            let header_size = std::mem::size_of::<u64>();

            loop {
                if let Ok(receiver) = data_ref.receiver.lock() {
                    if let Ok(data) = receiver.try_recv() {
                        if data {
                            break;
                        }
                    }
                }

                if read_bytes >= 8 {
                    let arr: [u8; 8] = buffer[0..header_size].try_into().unwrap();
                    amount_to_read = usize::from_le_bytes(arr);

                    if buffer.len() != header_size + amount_to_read {
                        buffer.resize(header_size + amount_to_read, 0);
                    }
                }

                match socket.read(&mut buffer[read_bytes..]) {
                    Ok(size) => {
                        if size == 0 {
                            if let Ok(on_message_received) = data_ref.on_message_received.lock() {
                                on_message_received(&[]);
                                // exit(0);
                            }
                        } else {
                            read_bytes += size;

                            if amount_to_read > 0 && read_bytes == header_size + amount_to_read {
                                if let Ok(on_message_received) = data_ref.on_message_received.lock()
                                {
                                    on_message_received(&buffer[header_size..]);
                                    buffer.resize(8, 0);
                                    read_bytes = 0;
                                    amount_to_read = 0;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::WouldBlock {
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                }
            }
        });
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) {
        self.nonblocking = nonblocking;
    }

    pub fn set_on_message_received<F>(&mut self, callback: F)
    where
        F: Fn(&[u8]) + Send + 'static,
    {
        if let Ok(mut cb) = self.data.on_message_received.lock() {
            *cb = Box::new(callback);
        }
    }
}
