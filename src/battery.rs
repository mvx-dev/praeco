use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};

use crate::{BatteryConfig, SysFsError, SysFsInstant, UEVENT_ROOT, config};

#[derive(Debug)]
pub struct Battery {
    pub instant: SysFsInstant,
    pub uevent_path: PathBuf,
}

impl Battery {
    // pub fn new(uevent: &PathBuf) -> Self {
    //     Battery {
    //         instant: SysFsInstant::new(),
    //         uevent_path: uevent.to_path_buf(),
    //     }
    // }

    pub fn update(&mut self) -> Result<(), SysFsError> {
        let uevent = fs::read_to_string(&self.uevent_path)?;
        self.instant = uevent.as_str().parse()?;

        Ok(())
    }

    pub fn time_estimate(&self) -> Option<f32> {
        self.instant.time_estimate()
    }

    pub fn capacity(&self) -> f32 {
        self.instant.capacity()
    }

    pub fn get_modification_time(&self) -> Result<SystemTime, SysFsError> {
        let metadata = fs::metadata(&self.uevent_path)?;

        let time = metadata.modified()?;
        Ok(time)
    }
}

impl FromStr for Battery {
    type Err = SysFsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uevent = PathBuf::from(s);
        let mut battery = Battery {
            instant: SysFsInstant::new(),
            uevent_path: uevent,
        };

        battery.update()?;

        Ok(battery)
    }
}

impl TryFrom<&PathBuf> for Battery {
    type Error = SysFsError;
    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let mut battery = Battery {
            instant: SysFsInstant::new(),
            uevent_path: value.to_path_buf(),
        };
        battery.update()?;

        Ok(battery)
    }
}

impl TryFrom<&config::BatteryConfig> for Battery {
    type Error = SysFsError;
    fn try_from(value: &BatteryConfig) -> Result<Self, Self::Error> {
        Battery::try_from(
            &Path::new(UEVENT_ROOT)
                .join(value.uevent_name.as_str())
                .join("uevent"),
        )
    }
}
