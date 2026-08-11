use std::{error::Error, fmt, io, str::FromStr};

#[derive(Debug)]
#[allow(unused)]
pub enum SysFsError {
    MissingField(Option<String>),
    ConversionError(Option<String>),
    Io(io::Error),
}

impl Error for SysFsError {}

impl fmt::Display for SysFsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SysFsError::ConversionError(s) => {
                return write!(
                    f,
                    "conversion error: unable to convert {}",
                    s.clone().unwrap_or("unknown".to_string())
                );
            }
            SysFsError::MissingField(s) => {
                return write!(
                    f,
                    "missing field error: unable to populate field {}",
                    s.clone().unwrap_or("unknown".to_string())
                );
            }
            SysFsError::Io(s) => return s.fmt(f),
        }
    }
}

impl From<io::Error> for SysFsError {
    fn from(value: io::Error) -> Self {
        SysFsError::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SysFsStatus {
    Charging,
    Discharging,
    NotCharging,
    Full,
    Unknown,
}

impl Default for SysFsStatus {
    fn default() -> Self {
        SysFsStatus::Unknown
    }
}

impl fmt::Display for SysFsStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match &self {
            SysFsStatus::Charging => "charging",
            SysFsStatus::Discharging => "discharging",
            SysFsStatus::NotCharging => "not charging",
            SysFsStatus::Full => "full",
            _ => "unknown",
        };
        write!(f, "{}", status)
    }
}

impl FromStr for SysFsStatus {
    type Err = SysFsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match () {
            _ if s.eq_ignore_ascii_case("charging") => SysFsStatus::Charging,
            _ if s.eq_ignore_ascii_case("discharging") => SysFsStatus::Discharging,
            _ if s.eq_ignore_ascii_case("not charging") => SysFsStatus::NotCharging,
            _ if s.eq_ignore_ascii_case("full") => SysFsStatus::Full,
            _ => SysFsStatus::Unknown,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct SysFsInstant {
    status: SysFsStatus,
    current_now: i32,
    charge_now: i32,
    charge_full: i32,
    name: String,
}

impl SysFsInstant {
    pub fn new() -> Self {
        SysFsInstant::default()
    }

    pub fn time_estimate(&self) -> Option<f32> {
        match self.status {
            SysFsStatus::Discharging => Some(self.discharge_estimate()),
            SysFsStatus::Charging => Some(self.charge_estimate()),
            SysFsStatus::Full => Some(0f32),
            _ => None,
        }
    }

    pub fn capacity(&self) -> f32 {
        self.charge_now as f32 / self.charge_full as f32
    }

    fn discharge_estimate(&self) -> f32 {
        self.charge_now as f32 / self.current_now as f32
    }

    fn charge_estimate(&self) -> f32 {
        -(self.charge_full - self.charge_now) as f32 / self.current_now as f32
    }
}

impl FromStr for SysFsInstant {
    type Err = SysFsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut instant = SysFsInstant::new();

        let parse_num = |v: &str| {
            v.parse()
                .map_err(|_| SysFsError::ConversionError(Some(v.to_string())))
        };

        for line in s.lines() {
            let Some((name, value)) = line.split_once('=') else {
                continue;
            };

            match name.to_ascii_lowercase().as_str() {
                "power_supply_name" => instant.name = value.into(),
                "power_supply_status" => instant.status = SysFsStatus::from_str(value)?,
                "power_supply_current_now" => instant.current_now = parse_num(value)?,
                "power_supply_charge_now" => instant.charge_now = parse_num(value)?,
                "power_supply_charge_full" => instant.charge_full = parse_num(value)?,
                _ => {}
            }
        }

        Ok(instant)
    }
}
