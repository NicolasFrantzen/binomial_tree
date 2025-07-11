use statrs::distribution::{ContinuousCDF, Normal};
use crate::instruments::OptionType;

pub fn black_value(
    option_type: OptionType,
    spot: f32,
    strike: f32,
    vol:  f32,
    rate: f32,
    dividends: f32,
    expiry: f32
) -> f32 {
    let n = Normal::new(0., 1.).unwrap();

    let d1 = (((spot/strike).ln() + expiry*(rate - dividends + vol.powi(2)/2.0))/(vol * expiry.sqrt())) as f64;
    let d2 = d1 - (vol * expiry.sqrt()) as f64;

    match option_type {
        OptionType::Put => {
            strike * (-rate*expiry).exp() * (n.cdf(-d2) as f32) - spot * (-dividends*expiry).exp() * (n.cdf(-d1) as f32)
        }
        OptionType::Call => {
            spot * (-dividends*expiry).exp() * (n.cdf(d1) as f32) - strike * (-rate*expiry).exp() * (n.cdf(d2) as f32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_black_call() {
        let val = black_value(OptionType::Call, 100.0, 95.0, 0.3, 0.05, 0.0, 0.5);
        assert_eq!(val, 12.327911);
    }

    #[test]
    fn test_black_put() {
        let val = black_value(OptionType::Put, 95.0, 100.0, 0.3, 0.05, 0.0, 0.5);
        assert_eq!(val, 9.459187);
    }
}