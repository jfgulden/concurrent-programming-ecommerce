#[derive(Debug)]
pub enum PurchaseError {
    OutOfStock,
    NotDelivered,
}

#[derive(Debug)]
pub enum FileError {
    NotFound,
    WrongFormat,
}

#[derive(Debug)]
pub enum StreamError {
    CannotCall,
    WrongFormat,
    CannotRead,
    CannotWrite,
}
