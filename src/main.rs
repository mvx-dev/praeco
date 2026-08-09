use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
#[allow(unused)]
enum SysFsParseError {
    MissingField(Option<String>),
    ConversionError(Option<String>),
}

#[derive(Debug)]
enum SysFsStatus {
    Charging,
    Discharing,
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
        let val = match &self {
            SysFsStatus::Charging => "charging",
            SysFsStatus::Discharing => "discharging",
            SysFsStatus::NotCharging => "not charging",
            SysFsStatus::Full => "full",
            _ => "unknown",
        };
        write!(f, "{}", val)
    }
}

impl FromStr for SysFsStatus {
    type Err = SysFsParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Ok(match () {
            _ if s.eq_ignore_ascii_case("charging") => SysFsStatus::Charging,
            _ if s.eq_ignore_ascii_case("discharging") => SysFsStatus::Discharing,
            _ if s.eq_ignore_ascii_case("notcharging") => SysFsStatus::NotCharging,
            _ if s.eq_ignore_ascii_case("full") => SysFsStatus::Full,
            _ => SysFsStatus::Unknown,
        });
    }
}

// Data from a snapshot of the uevent file. All units uA
#[derive(Debug, Default)]
struct SysFsInstant {
    status: SysFsStatus,
    current_now: i32,
    charge_now: i32,
    charge_full: i32,
    name: String,
}

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

        for line in sys_fs_iterator {
            if let Some((name, value)) = line.split_once("=") {
                match () {
                    _ if name.eq_ignore_ascii_case("power_supply_name") => {
                        instant.name = value.into()
                    }
                    _ if name.eq_ignore_ascii_case("power_supply_status") => {
                        instant.status = match SysFsStatus::from_str(value) {
                            Ok(s) => s,
                            Err(e) => return Err(e),
                        }
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
                    _ => continue,
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
        let _ = dbg!(instant);
    }

    Ok(())
}
