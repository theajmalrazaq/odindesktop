use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Files {
    pub bl: Option<String>,
    pub ap: Option<String>,
    pub cp: Option<String>,
    pub csc: Option<String>,
    pub ums: Option<String>,
}
impl Files {
    fn entries(&self) -> [(&str, &Option<String>); 5] {
        [
            ("-b", &self.bl),
            ("-a", &self.ap),
            ("-c", &self.cp),
            ("-s", &self.csc),
            ("-u", &self.ums),
        ]
    }
}
pub fn valid_device(device: &str) -> bool {
    let parts: Vec<_> = device.split('/').collect();
    parts.len() == 6
        && parts[0].is_empty()
        && parts[1] == "dev"
        && parts[2] == "bus"
        && parts[3] == "usb"
        && parts[4..]
            .iter()
            .all(|p| p.len() == 3 && p.bytes().all(|c| c.is_ascii_digit()))
}
pub fn arguments(
    files: &Files,
    device: &str,
    flash: bool,
    reboot: bool,
) -> Result<Vec<String>, String> {
    if !valid_device(device) {
        return Err("Select a detected USB device.".into());
    }
    let mut args = vec!["-d".into(), device.into()];
    let mut count = 0;
    for (flag, path) in files.entries() {
        if let Some(path) = path {
            let p = Path::new(path);
            if !p.is_absolute() || !(path.ends_with(".tar") || path.ends_with(".tar.md5")) {
                return Err("Firmware must be an absolute .tar or .tar.md5 path.".into());
            }
            count += 1;
            args.push(flag.into());
            args.push(path.clone());
        }
    }
    if count == 0 {
        return Err("Select at least one firmware archive.".into());
    }
    if !flash {
        args.push("--check-only".into());
    } else if reboot {
        args.push("--reboot".into());
    }
    Ok(args)
}
#[derive(PartialEq, Eq)]
pub struct Receipt {
    device: String,
    files: Vec<(String, String, [u8; 32])>,
}
impl Receipt {
    pub fn capture(files: &Files, device: &str) -> Result<Self, String> {
        arguments(files, device, false, false)?;
        let mut fingerprints = vec![];
        for (slot, path) in files.entries() {
            if let Some(path) = path {
                let mut file = File::open(path).map_err(|e| format!("Cannot read {slot}: {e}"))?;
                let metadata = file.metadata().map_err(|e| e.to_string())?;
                if !metadata.is_file() || metadata.len() == 0 {
                    return Err(format!("{slot} must be a nonempty regular file"));
                }
                let mut hash = Sha256::new();
                let mut buffer = [0u8; 65536];
                loop {
                    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    hash.update(&buffer[..n]);
                }
                fingerprints.push((slot.into(), path.clone(), hash.finalize().into()));
            }
        }
        Ok(Self {
            device: device.into(),
            files: fingerprints,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn files() -> Files {
        Files {
            bl: None,
            ap: Some("/tmp/AP file.tar.md5".into()),
            cp: None,
            csc: None,
            ums: None,
        }
    }
    #[test]
    fn validation_is_explicit_and_paths_are_single_arguments() {
        let a = arguments(&files(), "/dev/bus/usb/001/002", false, true).unwrap();
        assert_eq!(
            a,
            vec![
                "-d",
                "/dev/bus/usb/001/002",
                "-a",
                "/tmp/AP file.tar.md5",
                "--check-only"
            ]
        );
    }
    #[test]
    fn rejects_invalid_devices_and_empty_inputs() {
        assert!(arguments(&files(), "--reboot", true, false).is_err());
        let mut f = files();
        f.ap = None;
        assert!(arguments(&f, "/dev/bus/usb/001/002", true, false).is_err());
    }
    #[test]
    fn flash_never_adds_destructive_overrides() {
        let a = arguments(&files(), "/dev/bus/usb/001/002", true, true).unwrap();
        assert!(a.contains(&"--reboot".into()));
        assert!(!a.contains(&"--check-only".into()));
        assert!(!a.contains(&"--allow-unknown".into()));
    }
    #[test]
    fn fingerprints_detect_content_changes() {
        let path = std::env::temp_dir().join(format!("odin-receipt-{}.tar", std::process::id()));
        std::fs::write(&path, b"first").unwrap();
        let mut f = files();
        f.ap = Some(path.to_str().unwrap().into());
        let a = Receipt::capture(&f, "/dev/bus/usb/001/002").unwrap();
        std::fs::write(&path, b"other").unwrap();
        let b = Receipt::capture(&f, "/dev/bus/usb/001/002").unwrap();
        assert!(a != b);
        std::fs::remove_file(path).unwrap();
    }
}
