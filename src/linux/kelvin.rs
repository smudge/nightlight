const WARMEST: f64 = 3500.0;
const COOLEST: f64 = 6000.0;

pub(crate) fn from_strength(strength: i32) -> u32 {
    let ratio = strength.clamp(0, 100) as f64 / 100.0;
    ((1.0 - ratio) * (COOLEST - WARMEST) + WARMEST).round() as u32
}

pub(crate) fn to_strength(kelvin: f64) -> i32 {
    let ratio = 1.0 - (kelvin - WARMEST) / (COOLEST - WARMEST);
    ((ratio * 100.0).round() as i32).clamp(0, 100)
}

#[cfg(test)]
mod tests {
    use super::{from_strength, to_strength};

    #[test]
    fn strength_and_kelvin_round_trip() {
        for strength in 0..=100 {
            assert_eq!(to_strength(from_strength(strength) as f64), strength);
        }
        assert_eq!(from_strength(0), 6000);
        assert_eq!(from_strength(50), 4750);
        assert_eq!(from_strength(100), 3500);
    }

    #[test]
    fn out_of_range_temperatures_clamp() {
        assert_eq!(to_strength(2700.0), 100);
        assert_eq!(to_strength(6500.0), 0);
    }
}
