#![deny(missing_docs)]
//#![deny(warnings)]

//! # basicvideostreamer
//!
//! Simple video streaming library using HTTP 1.1 model.
//!
//! The current version currently allows only jpg images to be sent using multipart content type from HTTP.
//! The implementation uses internal threads without blocking the main program. 
//! It is possible a primitive library, but I think quite useful.
//!
//! If you have any suggestions, comments, requests - don't hesitate to subscribe to me directly (check repository owner) or leave me a ticket on GitHub.
//! Pull request are also welcome. I can make a simple feature request quickly (such as enabling png streaming) - I just need to know if there is a demand.

use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;
use std::net::TcpStream;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::io::Read;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::atomic::AtomicBool;

const BOUNDARY_NAME: &str = "basic_stream_boundary";



/// Configuration of streaming server.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct ServerConfiguration {
    ///TCP server port
    pub port: u16,
    ///Address of server (for example, localhost)
    pub address: String,
    ///Endpoint e.g. in the case of 127.0.0.1:72727/x/point2 the /x/point 2 is the endpoint. Requests for other locations will be rejected.
    pub endpoint: Option<String>
}

impl ServerConfiguration {
    ///The server constructor, parameters correspond to the next configuration fields. None signifies consent to the default value.
    pub fn new(port: u16, address: Option<String>, endpoint: Option<String>) -> ServerConfiguration {
        ServerConfiguration {
            port: port,
            address: address.unwrap_or("127.0.0.1".to_string()),
            endpoint: endpoint
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum ServerInputEvent {
    Stop,
    Data(Vec<u8>)
}

///Server instances.
#[derive(Debug)]
pub struct Server {
    ///The current server configuration. After creation, it is unchanging.
    pub configuration: ServerConfiguration,
    running: Arc<AtomicBool>,
    response_thread: Option< JoinHandle<()>>,
    streamer_channel: Option<Sender<ServerInputEvent>>
}


#[inline]
fn handle_connection(mut stream: &TcpStream, path: &Option<String>) -> Result<bool, std::io::Error> {
    
    #[inline]
    fn fail_response(mut stream: &TcpStream) -> Result<bool, std::io::Error> {
        stream.write("HTTP/1.1 404 Not Found\r\n".as_bytes())?;
        stream.write("\r\n".as_bytes())?;
        stream.flush()?;
        return Ok(false)
    }
    
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let mut headers = [httparse::EMPTY_HEADER; 1024];
    let mut request = httparse::Request::new(&mut headers);
    request.parse(&buffer).map_err(|_| {std::io::Error::new(ErrorKind::AddrNotAvailable, "Unable to use communication channel.") })?;
    if request.method.unwrap_or_default() != "GET" || request.path.unwrap_or_default() != path.as_ref().unwrap_or(&"/".to_string()) {
        return fail_response(stream);
    }

    stream.write("HTTP/1.1 200 OK\r\n".as_bytes())?;
    stream.write(format!("Content-Type: multipart/x-mixed-replace; boundary={}\r\n", BOUNDARY_NAME).as_bytes())?;
    stream.write("Connection: close\r\n".as_bytes())?;
    stream.write("Expires: 0\r\n".as_bytes())?;
    stream.write("Max-Age: 0\r\n".as_bytes())?;
    stream.write("Connection: close\r\n".as_bytes())?;
    stream.write("Cache-Control: no-cache, private\r\n".as_bytes())?;
    stream.write("Accept-Range: bytes\r\n".as_bytes())?;
    stream.write("Pragma: no-cache\r\n".as_bytes())?;
    stream.write("\r\n".as_bytes())?;
    stream.flush()?;
    Ok(true)
}

#[inline]
fn send_stream_data(mut stream: &TcpStream, data: &Vec<u8>) -> Result<(), std::io::Error> {
    stream.write("--myboundary\r\n".as_bytes())?;
    stream.write(format!("--{}\r\n", BOUNDARY_NAME).as_bytes())?;
    stream.write("Content-Type: image/jpeg\r\n".as_bytes())?;
    stream.write(format!("Content-Length: {}\r\n", data.len()).as_bytes())?;
    stream.write("\r\n".as_bytes())?;
    stream.write(&data[..])?;
    stream.flush()?;
    Ok(())
}
    

use std::io::ErrorKind;
use std::io;
use std::time;

impl Server {
    ///The server constructor based on the configuration.
    pub fn new(config: ServerConfiguration) -> Server {
        Server {
            configuration: config,
            running: Arc::new(AtomicBool::new(false)),
            response_thread: None,
            streamer_channel: None,
        }
    }

    ///Start monitoring and answering incoming calls. Return false when the server is already running.
    pub fn start(&mut self) -> Result<bool, std::io::Error> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(false);
        }
        self.running.store(true, Ordering::Relaxed);

        let listener = TcpListener::bind(format!("{0}:{1}", self.configuration.address, self.configuration.port))?;
        listener.set_nonblocking(true)?;

        let (sender, reciver): (Sender<ServerInputEvent>, Receiver<ServerInputEvent>) = mpsc::channel();

        self.streamer_channel = Some(sender.clone());
        let is_running = self.running.clone();
        let path = self.configuration.endpoint.clone();
        self.response_thread = Some(thread::spawn(move || {
            let connetions = Arc::new(Mutex::new(Vec::new()));
            let connetions_handle = connetions.clone();
            
            let streamer = thread::spawn(move || {

                for data in reciver.iter() {
                    match data {

                        ServerInputEvent::Stop => {
                            break;
                        },
                        ServerInputEvent::Data(data) => {
                            match connetions_handle.lock() {
                                Ok(mut connetions) => {
                                    let mut to_remove = Vec::new();
                                    for (i, connection) in connetions.iter().enumerate() {
                                        if send_stream_data(connection, &data).is_err() {
                                            to_remove.push(i);
                                        }
                                    }

                                    //remove exoired connections
                                    for remove_index in to_remove.iter().rev() {
                                        connetions.swap_remove(*remove_index);
                                    }

                                },
                                Err(_) => {
                                    break;
                                }
                            }
                        }
                    }
                }
            });

            loop {
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }


                match listener.accept() {
                    Ok((stream, _) ) => {
                        match connetions.lock() {
                            Ok(mut connetions) => {
                                if handle_connection(&stream, &path).unwrap_or(false) {
                                    connetions.push(stream);
                                }
                            },
                            Err(_) => {
                                break;
                            }
                        }
                    },
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(time::Duration::from_millis(30));
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
            if sender.send(ServerInputEvent::Stop).is_ok() {
                let _ = streamer.join();
            }
        }));
        Ok(true)
    }


    ///Stop monitoring and answering incoming calls. Return false when the server is already suspended.
    pub fn stop(&mut self) -> Result<bool, std::io::Error> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(false)
        }
        self.running.store(false, Ordering::Relaxed);
        self.streamer_channel.as_ref().ok_or(std::io::Error::new(ErrorKind::BrokenPipe, "Unable to use communication channel."))?.
        send(ServerInputEvent::Stop).map_err(|_| { std::io::Error::new(ErrorKind::BrokenPipe, "Unable to send data by channel.")})?;
        self.response_thread.take().ok_or(std::io::Error::new(ErrorKind::BrokenPipe, "Unable to use communication channel."))?.
        join().map_err(|_| { std::io::Error::new(ErrorKind::BrokenPipe, "Unable to send data by channel.")})?;
        self.response_thread = None;
        self.streamer_channel = None;
        Ok(true)
    }

    ///Sends image data to the server which will be sent to all registered clients. Returns false when the server is down.
    /// 
    /// The byte string must be a valid jpg image. Other codecs are not currently supported. The server is not validating the correct encoding.
    pub fn send(&mut self, data: &Vec<u8>) -> Result<bool, std::io::Error>  {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(false)
        }
        self.streamer_channel.as_ref().ok_or(std::io::Error::new(ErrorKind::BrokenPipe, "Unable to use communication channel."))?.
        send(ServerInputEvent::Data(data.to_vec())).map_err(|_| { std::io::Error::new(ErrorKind::BrokenPipe, "Unable to send data by channel.")})?;
        Ok(true)
    }

    ///Checking if the server is running. Due to its multi-threaded nature, it can sometimes give an incorrect result (at the time of the query).
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}


impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}