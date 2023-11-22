use rand::{thread_rng, Rng};

use crate::constants::DELIVER_LOST_RATE;

//LocalPurchase solo va a tener 3 estados: CREATED, SOLD o REJECTED.
#[derive(Debug, Clone)]
pub enum LocalPurchaseState {
    CREATED,
    SOLD,
    REJECTED,
}
impl LocalPurchaseState {
    pub fn to_string(&self) -> String {
        match self {
            LocalPurchaseState::CREATED => "CREADO".to_string(),
            LocalPurchaseState::SOLD => "VENDIDO".to_string(),
            LocalPurchaseState::REJECTED => "RECHAZADO".to_string(),
        }
    }
}

//EcomPurchase va a tener 5 estados: CREATED, RESERVED, DELIVERED, REJECTED o LOST.
#[derive(Debug, Clone)]
pub enum OnlinePurchaseState {
    CREATED,
    RESERVED,
    DELIVERED,
    REJECTED,
    LOST,
}
impl OnlinePurchaseState {
    pub fn to_string(&self) -> String {
        match self {
            OnlinePurchaseState::CREATED => "CREADO".to_string(),
            OnlinePurchaseState::RESERVED => "RESERVADO".to_string(),
            OnlinePurchaseState::REJECTED => "NO STOCK".to_string(),
            OnlinePurchaseState::DELIVERED => "ENTREGADO".to_string(),
            OnlinePurchaseState::LOST => "PERDIDO".to_string(),
        }
    }
    pub fn deliver_attempt(&mut self) {
        let is_delivered = thread_rng().gen_bool(DELIVER_LOST_RATE);
        if is_delivered {
            *self = OnlinePurchaseState::DELIVERED;
        } else {
            *self = OnlinePurchaseState::LOST;
        }
    }
    pub fn from_int(int: u8) -> Option<OnlinePurchaseState> {
        match int {
            0 => Some(OnlinePurchaseState::CREATED),
            1 => Some(OnlinePurchaseState::RESERVED),
            2 => Some(OnlinePurchaseState::REJECTED),
            3 => Some(OnlinePurchaseState::DELIVERED),
            4 => Some(OnlinePurchaseState::LOST),
            _ => None,
        }
    }
    pub fn to_int(&self) -> u8 {
        match self {
            OnlinePurchaseState::CREATED => 0,
            OnlinePurchaseState::RESERVED => 1,
            OnlinePurchaseState::REJECTED => 2,
            OnlinePurchaseState::DELIVERED => 3,
            OnlinePurchaseState::LOST => 4,
        }
    }
}

impl PartialEq for OnlinePurchaseState {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
