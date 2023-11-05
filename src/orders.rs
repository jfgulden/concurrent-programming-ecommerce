use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use crate::{error::FileError, messages::local_purchase::LocalPurchase};

pub struct Orders {
    pub list: Vec<LocalPurchase>,
}

impl Orders {
    pub fn from_file(path: &str) -> Result<Self, FileError> {
        let file = File::open(path).map_err(|_| FileError::NotFound)?;
        Self::from_reader(file)
    }

    fn from_reader<T: Read>(content: T) -> Result<Self, FileError> {
        let reader = BufReader::new(content);

        let mut orders = Self { list: Vec::new() };

        for line in reader.lines() {
            let current_line = line.map_err(|_| FileError::WrongFormat)?;
            let line_slices: Vec<&str> = current_line.split(',').collect();

            if line_slices.len() != 2 {
                return Err(FileError::WrongFormat);
            }

            let order = LocalPurchase {
                product_id: line_slices[0].to_string(),
                quantity: line_slices[1].parse().map_err(|_| FileError::WrongFormat)?,
            };

            orders.list.push(order);
        }

        Ok(orders)
    }
}
