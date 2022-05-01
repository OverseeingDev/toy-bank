const FIXED_POINT_MAGNITUDE: i64 = 10000;

pub fn string_to_fixed_point(string: &str) -> Result<i64, &'static str> {
    let split_amount: Vec<&str> = string.split(".").collect();

    if split_amount.len() != 2 {
        return Err("Amount contains more than one dot or less than one dot");
    }

    let units: i64 = split_amount[0].parse().expect("Couldn't parse amount");
    if units < 0 {
        return Err("Invalid amount: negative");
    }

    const EXPECTED_PRECISION: usize = 4;
    let digits = split_amount[1].len();

    if digits > EXPECTED_PRECISION {
        return Err("Provided amount exceeds asserted 4 digits past point fixed point precision");
    }

    let decimal_multiplier: i64 = 10i64.pow((EXPECTED_PRECISION - digits).try_into().unwrap());

    let mut ten_thousandths: i64 = split_amount[1]
        .parse()
        .expect("Couldnt parse ten thousandths");

    ten_thousandths *= decimal_multiplier;

    Ok(units * FIXED_POINT_MAGNITUDE + ten_thousandths)
}

pub fn fixed_point_to_string(fixed_point: i64) -> String {
    format!(
        "{}.{:4>0}",
        fixed_point / FIXED_POINT_MAGNITUDE,
        fixed_point.abs() % FIXED_POINT_MAGNITUDE
    )
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;

    mod string_to_fixed_point {
        use super::*;

        #[test]
        fn fixed_point_conversion_1_0() {
            assert_eq!(string_to_fixed_point("1.0"), Ok(10000))
        }
        #[test]
        fn fixed_point_conversion_1_1() {
            assert_eq!(string_to_fixed_point("1.1"), Ok(11000))
        }

        #[test]
        fn fixed_point_conversion_1_10() {
            assert_eq!(string_to_fixed_point("1.10"), Ok(11000))
        }
        #[test]
        fn fixed_point_conversion_1_101() {
            assert_eq!(string_to_fixed_point("1.101"), Ok(11010))
        }
        #[test]
        fn fixed_point_conversion_1_1012() {
            assert_eq!(string_to_fixed_point("1.1012"), Ok(11012))
        }
        #[test]
        fn badly_formatted_more_than_a_dot() {
            assert!(string_to_fixed_point("1.1.01").is_err());
        }
        #[test]
        fn badly_formatted_less_than_one_dot() {
            assert!(string_to_fixed_point("101").is_err());
        }
        #[test]
        fn badly_formatted_too_precise() {
            assert!(string_to_fixed_point("1.0105115").is_err());
        }
        #[test]
        fn badly_formatted_negative() {
            assert!(string_to_fixed_point("-1.010").is_err());
        }
        #[test]
        #[should_panic]
        fn badly_formatted_non_number_decimal() {
            string_to_fixed_point("1.0a10");
        }
        #[test]
        #[should_panic]
        fn badly_formatted_non_number_integer() {
            string_to_fixed_point("1a.010");
        }
    }

    mod fixed_point_to_string {
        use super::*;

        #[test]
        fn positive_to_string() {
            assert_eq!(fixed_point_to_string(10000), "1.0");
        }

        #[test]
        fn positive_to_string_decimal() {
            assert_eq!(fixed_point_to_string(15000), "1.5000");
        }

        #[test]
        fn negative_to_string() {
            assert_eq!(fixed_point_to_string(-10000), "-1.0");
        }

        #[test]
        fn negative_to_string_decimal() {
            assert_eq!(fixed_point_to_string(-15000), "-1.5000");
        }
    }
}
