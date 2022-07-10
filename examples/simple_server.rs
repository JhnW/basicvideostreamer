extern crate basicvideostreamer;
use basicvideostreamer::Server;
use basicvideostreamer::ServerConfiguration;
use std::{thread, time};
use image::imageops::rotate180_in_place;
use std::io::Cursor;
use image::io::Reader as ImageReader;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 { args[1].clone() } else {"in.jpg".to_string()};

    let config = ServerConfiguration::new(7879, None, Some("/img".to_string()));
    let mut server = Server::new(config);
    if server.start().is_err() {
        println!("Error: create server.");
        return;
    }

    let mut img = ImageReader::open(path).unwrap().decode().unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    loop {
        rotate180_in_place(&mut img);
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Jpeg(100)).unwrap();
        thread::sleep(time::Duration::from_millis(17));
        if server.send(&bytes).is_err() {
            println!("Error: send data.");
            break;
        }
    }
}