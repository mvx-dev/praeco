// This work is licensed under Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International.
// To view a copy of this license, visit https://creativecommons.org/licenses/by-nc-sa/4.0/

use std::error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread::sleep;

mod battery;
mod sysfs;
pub use battery::*;
pub use sysfs::*;
use zbus::blocking::Connection;
use zbus::zvariant::Value;

fn main() -> Result<(), Box<dyn error::Error>> {
    let ps_root = PathBuf::from("/sys/class/power_supply/BAT1/uevent");

    // let entries: Vec<_> = fs::read_dir(ps_root)?
    //     .map(|res| res.map(|e| e.path()))
    //     .collect::<Result<Vec<_>, io::Error>>()?
    //     .into_iter()
    //     .filter(|p| {
    //         p.file_name()
    //             .and_then(|n| n.to_str())
    //             .is_some_and(|n| n.contains("BAT"))
    //     })
    //     .collect();
    let mut battery = Battery::try_from(&ps_root)?;

    // let uevent = fs::read_to_string(Path::join(&entries[0], "uevent"))?;
    // let instant = SysFsInstant::from_str(&uevent.to_string()).unwrap();
    // dbg!(&instant);
    // dbg!(&instant.time_estimate());

    loop {
        battery.update()?;
        println!("Percentage: {}", battery.capacity());
        dbg!(&battery.instant);
        sleep(std::time::Duration::from_secs(1));
    }
}
