use std::fs::File;
use std::io::{BufRead, BufReader};

const FSTAB_PATH: &'static str = "/etc/fstab";

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum ErrorType {
    FstabNotExist(String),
    NumParseError(String),
    FieldNotExist(usize),
    TooManyFields(String),
}

#[derive(Debug, Clone)]
pub struct Error {
    reason: ErrorType,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        use std::error::Error;
        self::Error {
            reason: ErrorType::FstabNotExist(e.description().to_owned()),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        use std::error::Error;
        self::Error {
            reason: ErrorType::NumParseError(e.description().to_owned()),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self.reason {
            ErrorType::FstabNotExist(_) => "can not open fstab",
            _ => "unkonwn error",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Device {
    Uuid(String),
    Label(String),
    MountPoint(String),
    PartUuid(String),
    PartLabel(String),
}

#[derive(Debug, Clone)]
pub struct Fstab {
    pub device: Device,
    pub dir: String,
    pub device_type: String,
    pub options: Vec<String>,
    pub dump: bool,
    pub fsck: usize,
}

pub fn parse_device(name: &str) -> Device {
    if name.starts_with("UUID=") {
        Device::Uuid(name.split_at(5).1.to_owned())
    } else if name.starts_with("LABEL=") {
        Device::Label(name.split_at(6).1.to_owned())
    } else if name.starts_with("PARTUUID=") {
        Device::PartUuid(name.split_at(5).1.to_owned())
    } else if name.starts_with("PARTLABEL=") {
        Device::PartLabel(name.split_at(6).1.to_owned())
    } else {
        Device::MountPoint(name.to_owned())
    }
}

pub fn open_fstab(path: Option<&str>) -> Result<Vec<Fstab>> {
    let fstab_handle = File::open(match path {
        Some(p) => p,
        _ => FSTAB_PATH,
    })?;

    let reader = BufReader::new(fstab_handle);

    let mut fstab_item_list = Vec::new();

    for l in reader.lines() {
        if let Ok(l) = l {
            let l = l.trim();
            if l.starts_with("#") || l.len() == 0 {
                continue;
            }
            let mut tabs = l.split_whitespace();
            fstab_item_list.push(Fstab {
                device: parse_device(tabs.next().ok_or(Error {
                    reason: ErrorType::FieldNotExist(0),
                })?),
                dir: tabs.next()
                    .ok_or(Error {
                        reason: ErrorType::FieldNotExist(1),
                    })?
                    .to_owned(),
                device_type: tabs.next()
                    .ok_or(Error {
                        reason: ErrorType::FieldNotExist(2),
                    })?
                    .to_owned(),
                options: tabs.next()
                    .ok_or(Error {
                        reason: ErrorType::FieldNotExist(3),
                    })?
                    .split(",")
                    .map(|x| x.to_owned())
                    .collect::<Vec<_>>(),
                dump: match tabs.next() {
                    Some(x) => x.parse::<usize>()
                        .map(|x| if x > 0 { true } else { false })?,
                    _ => false,
                },
                fsck: match tabs.next() {
                    Some(x) => x.parse::<usize>()?,
                    _ => 0,
                },
            });
            if tabs.next().is_some() {
                return Err(Error { reason: ErrorType::TooManyFields(l.to_owned())});
            }
        }
    }
    Ok(fstab_item_list)
}

#[test]
fn read_default_fstab() {
    let fstab = open_fstab(None);
    println!("{:?}", fstab);
    assert!(fstab.is_ok());
}
