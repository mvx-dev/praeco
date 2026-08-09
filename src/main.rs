use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
enum SysFsParseError {
    MissingField(Option<String>),
    ConversionError(Option<String>),
}

#[derive(Debug)]
enum BatteryStatus {
    Charging,
    Discharing,
    NotCharging,
    Full,
    Unknown,
}

impl Default for BatteryStatus {
    fn default() -> Self {
        BatteryStatus::Unknown
    }
}

impl fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match &self {
            BatteryStatus::Charging => "charging",
            BatteryStatus::Discharing => "discharging",
            BatteryStatus::NotCharging => "not charging",
            BatteryStatus::Full => "full",
            _ => "unknown",
        };
        write!(f, "{}", val)
    }
}

impl FromStr for BatteryStatus {
    type Err = SysFsParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Ok(match () {
            _ if s.eq_ignore_ascii_case("charging") => BatteryStatus::Charging,

            _ => BatteryStatus::Unknown,
        });
    }
}

// Data from a snapshot of the uevent file. All units uA
#[derive(Debug, Default)]
struct SysFsInstant {
    status: BatteryStatus,
    current_now: i32,
    charge_now: i32,
    charge_full: i32,
    name: String,
}

// #[derive(Debug)]
// struct SysFsBattery {
//     instant: SysFsInstant,
// }

impl SysFsInstant {
    fn new() -> Self {
        SysFsInstant::default()
    }
}

impl FromStr for SysFsInstant {
    type Err = SysFsParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut instant = SysFsInstant::new();

        let sys_fs_iterator: Vec<_> = s.lines().collect();
        dbg!(&sys_fs_iterator);

        for line in sys_fs_iterator {
            if let Some((name, value)) = line.split_once("=") {
                dbg!(name, value);
                match () {
                    _ if name.eq_ignore_ascii_case("power_supply_name") => {
                        instant.name = value.into()
                    }
                    _ if name.eq_ignore_ascii_case("power_supply_current_now") => {
                        instant.current_now = match value.parse() {
                            Ok(v) => v,
                            Err(_) => {
                                return Err(SysFsParseError::ConversionError(Some(
                                    value.to_string(),
                                )));
                            }
                        }
                    }
                    _ if name.eq_ignore_ascii_case("power_supply_charge_now") => {
                        instant.charge_now = match value.parse() {
                            Ok(v) => v,
                            Err(_) => {
                                return Err(SysFsParseError::ConversionError(Some(
                                    value.to_string(),
                                )));
                            }
                        }
                    }
                    _ if name.eq_ignore_ascii_case("power_supply_charge_full") => {
                        instant.charge_full = match value.parse() {
                            Ok(v) => v,
                            Err(_) => {
                                return Err(SysFsParseError::ConversionError(Some(
                                    value.to_string(),
                                )));
                            }
                        }
                    }
                    _ => println!("not required"),
                }
            }
        }

        Ok(instant)
    }
}

fn main() -> io::Result<()> {
    let ps_root = PathBuf::from("/sys/class/power_supply");

    let entries: Vec<_> = fs::read_dir(ps_root)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("BAT"))
        })
        .collect();

    for entry in entries {
        let uevent = fs::read_to_string(Path::join(&entry, "uevent"))?;
        let instant = SysFsInstant::from_str(&uevent.to_string());
        dbg!(instant);
    }

    Ok(())
}
