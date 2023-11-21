use std::ops::RangeInclusive;

pub const PURCHASE_MILLIS: u64 = 200; // blocking

pub const LOCAL_PROCESING_MILLIS: RangeInclusive<u64> = 100..=300; // non-blocking

pub const DELIVER_MILLIS: RangeInclusive<u64> = 500..=700; // non-blocking

pub const ECOM_PROCESING_MILLIS: RangeInclusive<u64> = 250..=400; // non-blocking

pub const ECOM_MAX_WAITING_MILLIS: u64 = 5000; // non-blocking

pub const DELIVER_LOST_RATE: f64 = 0.5;
