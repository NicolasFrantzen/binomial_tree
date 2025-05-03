pub trait Option_ {
    fn new(option_type: OptionType, strike: f32, expiry: f32) -> Self;
    fn expiry(&self) -> f32;
    fn strike(&self) -> f32;
    fn option_type(&self) -> OptionType;
    fn value(&self, value: f32, price: f32) -> f32;

    fn intrinsic_value(&self, price: f32) -> f32 {
        match self.option_type() {
            OptionType::Put => (self.strike() - price).max(0.0),
            OptionType::Call => (price - self.strike()).max(0.0),
        }
    }
}

#[derive(Copy, Clone)]
pub enum OptionType {
    Put = -1,
    Call = 1,
}

pub struct AmericanOption {
    option_type: OptionType,
    strike: f32,
    expiry: f32,
}

impl Option_ for AmericanOption {
    fn new(option_type: OptionType, strike: f32, expiry: f32) -> Self {
        Self{ option_type, strike, expiry }
    }

    fn expiry(&self) -> f32 {
        self.expiry
    }
    fn strike(&self) -> f32 {
        self.strike
    }
    fn option_type(&self) -> OptionType { self.option_type }

    fn value(&self, value: f32, price: f32) -> f32 {
        let payoff = self.intrinsic_value(price);
        payoff.max(value)
    }
}

pub struct EuropeanOption {
    option_type: OptionType,
    strike: f32,
    expiry: f32,
}

impl Option_ for EuropeanOption {
    fn new(option_type: OptionType, strike: f32, expiry: f32) -> Self {
        Self { option_type, strike, expiry }
    }

    fn expiry(&self) -> f32 {
        self.expiry
    }
    fn strike(&self) -> f32 {
        self.strike
    }
    fn option_type(&self) -> OptionType { self.option_type }

    fn value(&self, value: f32, _: f32) -> f32 {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_american() {
        let option = AmericanOption{
            option_type: OptionType::Put,
            strike: 50.0,
            expiry: 0.5,
        };

        assert_eq!(option.intrinsic_value(30.0), 20.0);
        assert_eq!(option.intrinsic_value(60.0), 0.0);
        assert_eq!(option.value(10.0, 30.0), 20.0);
        assert_eq!(option.value(30.0, 40.0), 30.0);
        assert_eq!(option.value(40.0, 30.0), 40.0);
        assert_eq!(option.value(20.0, 10.0), 30.0);
    }

    #[test]
    fn test_european() {
        let option = EuropeanOption{
            option_type: OptionType::Put,
            strike: 50.0,
            expiry: 0.5,
        };

        assert_eq!(option.intrinsic_value(30.0), 20.0);
        assert_eq!(option.intrinsic_value(60.0), 0.0);
        assert_eq!(option.value(10.0, 30.0), 10.0);
        assert_eq!(option.value(30.0, 40.0), 30.0);
        assert_eq!(option.value(40.0, 30.0), 40.0);
        assert_eq!(option.value(20.0, 10.0), 20.0);
    }
}