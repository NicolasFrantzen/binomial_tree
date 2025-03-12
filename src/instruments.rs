pub(crate) trait Option_ {
    fn new(strike: f32, expiry: f32) -> Self;
    fn expiry(&self) -> f32;
    fn strike(&self) -> f32;
}

trait Call {
}

trait Put {
}

struct AmericanOption {
    strike: f32,
    expiry: f32,
}

impl Option_ for AmericanOption {
    fn new(strike: f32, expiry: f32) -> Self {
        Self{ strike, expiry }
    }

    fn expiry(&self) -> f32 {
        self.expiry
    }
    fn strike(&self) -> f32 {
        self.strike
    }
}

pub struct EuropeanOption {
    strike: f32,
    expiry: f32,
}

impl Option_ for EuropeanOption {
    fn new(strike: f32, expiry: f32) -> Self {
        Self{ strike, expiry }
    }

    fn expiry(&self) -> f32 {
        self.expiry
    }
    fn strike(&self) -> f32 {
        self.strike
    }
}
