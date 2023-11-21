use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    sync::Arc,
};

use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

use crate::error::FileError;

#[derive(Debug, Clone)]

pub struct ConnectedShop {
    pub name: String,
    pub zone_id: i32,
    pub stream: Option<Arc<Mutex<WriteHalf<TcpStream>>>>,
}

impl ConnectedShop {
    pub fn from_file() -> Result<Vec<(String, i32, TcpStream)>, FileError> {
        let mut streams: Vec<(String, i32, TcpStream)> = Vec::new();
        let location_files = fs::read_dir("tiendas").map_err(|_| FileError::NotFound)?;

        for dir_entry in location_files {
            let file = File::open(dir_entry.map_err(|_| FileError::NotFound)?.path());
            let reader = BufReader::new(file.map_err(|_| FileError::NotFound)?);
            let mut lines = reader.lines();

            let shop_info_string = match lines.next() {
                Some(string) => string.map_err(|_| FileError::WrongFormat)?,
                None => return Err(FileError::WrongFormat),
            };
            let shop_info: Vec<&str> = shop_info_string.split(',').collect();
            println!("{:?}", shop_info);
            if shop_info.len() != 3 {
                return Err(FileError::WrongFormat);
            }

            if let Ok(stream) = std::net::TcpStream::connect(shop_info[1]) {
                let location: i32 = shop_info[2].parse().map_err(|_| FileError::WrongFormat)?;
                if let Ok(stream_tokio) = TcpStream::from_std(stream) {
                    streams.push((shop_info[0].to_string(), location, stream_tokio));
                }
            }
        }

        Ok(streams)
    }
}
