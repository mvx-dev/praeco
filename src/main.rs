// This work is licensed under Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International.
// To view a copy of this license, visit https://creativecommons.org/licenses/by-nc-sa/4.0/

use std::collections::HashMap;
use std::error;
use std::path::{Path, PathBuf};
use std::thread::sleep;

mod battery;
mod config;
mod sysfs;
pub use battery::*;
pub use config::*;
pub use sysfs::*;
use zbus::blocking::Connection;
use zbus::zvariant::Value;

pub const UEVENT_ROOT: &str = "/sys/class/power_supply";

fn main() -> Result<(), Box<dyn error::Error>> {
    let config_file = Path::new("/home/cheshire/.config/praeco/config.toml");
    let config = load(config_file)?;
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
    let mut battery = match config.batteries.get("laptop") {
        Some(b) => Battery::try_from(b)?,
        None => panic!("not loaded correctly"),
    };

    // let uevent = fs::read_to_string(Path::join(&entries[0], "uevent"))?;
    // let instant = SysFsInstant::from_str(&uevent.to_string()).unwrap();
    // dbg!(&instant);
    // dbg!(&instant.time_estimate());

    let connection = Connection::session()?;

    let hints: HashMap<&str, Value> = HashMap::new();
    let actions: Vec<&str> = Vec::new();

    let threshold = 0.41 as f32;
    let _ = dbg!(battery.get_modification_time());
    loop {
        battery.update()?;
        println!("Percentage: {}", battery.capacity());
        dbg!(&battery.instant);
        if battery.capacity() <= threshold {
            let reply = connection.call_method(
                Some("org.freedesktop.Notifications"),
                "/org/freedesktop/Notifications",
                Some("org.freedesktop.Notifications"),
                "Notify",
                &(
                    "praeco",
                    0u32,
                    "",
                    "Battery Low",
                    "your battery is low :(",
                    actions,
                    hints,
                    5000i32,
                ),
            )?;
            let notification_id: u32 = reply.body().deserialize()?;
            println!("Notification sent (id {})", notification_id);
            return Ok(());
        }
        sleep(std::time::Duration::from_secs(1));
    }
}
