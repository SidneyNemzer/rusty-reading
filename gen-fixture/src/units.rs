use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes {
    value: usize,
}

impl Bytes {
    pub const fn from_bytes(value: usize) -> Self {
        Bytes { value }
    }

    pub fn from_unit(value: usize, unit: ByteUnit1024) -> Self {
        Bytes {
            value: value * unit.size_in_bytes(),
        }
    }

    pub fn to_bytes(&self) -> usize {
        self.value
    }

    pub fn format_bytes(&self) -> (usize, ByteUnit1024) {
        ByteUnit1024::format_bytes(self.value)
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (value, unit) = self.format_bytes();
        write!(f, "{}{}", value, unit.abbreviation())
    }
}

impl FromStr for Bytes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        #[derive(PartialEq)]
        enum ParsingState {
            Value,
            Unit,
        }

        let mut state = ParsingState::Value;
        let mut value_string = String::new();
        let mut unit_string = String::new();

        for c in s.chars() {
            if c.is_digit(10) {
                if state == ParsingState::Unit {
                    return Err(format!(
                        "Invalid unit '{}'. Valid units are 'B', 'KiB', 'MiB', and 'GiB'.",
                        s
                    ));
                }

                value_string.push(c);
            } else {
                if state == ParsingState::Value {
                    state = ParsingState::Unit;
                }

                unit_string.push(c);
            }
        }

        Ok(Bytes::from_unit(
            value_string.parse::<usize>().unwrap(),
            ByteUnit1024::from_abbreviation(&unit_string).ok_or_else(|| {
                format!(
                    "Invalid unit '{}'. Valid units are 'B', 'KiB', 'MiB', and 'GiB'.",
                    s
                )
            })?,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub enum ByteUnit1024 {
    Bytes,
    Kibibytes,
    Mebibytes,
    Gibibytes,
}

impl ByteUnit1024 {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            ByteUnit1024::Bytes => 1,
            ByteUnit1024::Kibibytes => 1024,
            ByteUnit1024::Mebibytes => 1024 * 1024,
            ByteUnit1024::Gibibytes => 1024 * 1024 * 1024,
        }
    }

    pub fn abbreviation(&self) -> &str {
        match self {
            ByteUnit1024::Bytes => "B",
            ByteUnit1024::Kibibytes => "KiB",
            ByteUnit1024::Mebibytes => "MiB",
            ByteUnit1024::Gibibytes => "GiB",
        }
    }

    pub fn from_abbreviation(abbreviation: &str) -> Option<Self> {
        match abbreviation.to_ascii_lowercase().as_str() {
            "b" => Some(ByteUnit1024::Bytes),
            "kib" | "k" => Some(ByteUnit1024::Kibibytes),
            "mib" | "m" => Some(ByteUnit1024::Mebibytes),
            "gib" | "g" => Some(ByteUnit1024::Gibibytes),
            _ => None,
        }
    }

    pub fn ordered() -> Vec<ByteUnit1024> {
        vec![
            ByteUnit1024::Bytes,
            ByteUnit1024::Kibibytes,
            ByteUnit1024::Mebibytes,
            ByteUnit1024::Gibibytes,
        ]
    }

    /// Given a number of bytes, converts them to the largest unit that
    /// can represent the size of `bytes` without being less than 1.
    pub fn format_bytes(bytes: usize) -> (usize, ByteUnit1024) {
        // 922193827938
        // 922193827938 / 1024 / 1024 / 1024 = 858.86
        // 922193827938 / 1024 / 1024 / 1024 / 1024 = 0.84
        //
        // x / (1024^n) >= 1
        // x >= 1024^n

        let units = ByteUnit1024::ordered();
        let mut closest = ByteUnit1024::Bytes;

        for unit in units {
            let size_in_units = bytes / unit.size_in_bytes();
            if size_in_units < 1 {
                break;
            }

            closest = unit;
        }

        (bytes / closest.size_in_bytes(), closest)
    }
}

#[cfg(test)]
mod tests {
    // This macro lets us write tests omitting assert_eq and ByteUnit1024::,
    // which makes most assertions short enough to fit on one line.
    macro_rules! test_case {
        (format_bytes($value:expr) => ($expected_value:expr, $expected_unit:ident)) => {
            assert_eq!(
                ByteUnit1024::format_bytes($value),
                ($expected_value, ByteUnit1024::$expected_unit)
            );
        };
    }

    #[test]
    fn test_format_bytes() {
        use super::ByteUnit1024;

        test_case!(format_bytes(1) => (1, Bytes));
        test_case!(format_bytes(1023) => (1023, Bytes));
        test_case!(format_bytes(1024) => (1, Kibibytes));
        test_case!(format_bytes(1024 * 1024 - 1) => (1023, Kibibytes));
        test_case!(format_bytes(1024 * 1024) => (1, Mebibytes));
        test_case!(format_bytes(1024 * 1024 * 1024 - 1) => (1023, Mebibytes));
        test_case!(format_bytes(1024 * 1024 * 1024) => (1, Gibibytes));
        // Gibibytes is the largest supported unit for now
        test_case!(format_bytes(1024 * 1024 * 1024 * 1024) => (1024, Gibibytes));
    }
}
