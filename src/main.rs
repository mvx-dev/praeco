// This work is licensed under Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International.
// To view a copy of this license, visit https://creativecommons.org/licenses/by-nc-sa/4.0/

use std::error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod sysfs;
use sysfs::*;

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
        let instant = SysFsInstant::from_str(&uevent.to_string()).unwrap();
        dbg!(&instant);
        dbg!(&instant.time_estimate());
    }
fn main() -> Result<(), Box<dyn error::Error>> {

    Ok(())
}
