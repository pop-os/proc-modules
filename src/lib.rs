//! Provides easy access to active kernel modules in `/proc/modules`.
//!
//! ```rust,no_run
//! extern crate proc_modules;
//!
//! use proc_modules::ModuleIter;
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     for module in ModuleIter::new()? {
//!         println!("{:#?}", module?);
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// A Linux kernel module.
#[derive(Debug, PartialEq)]
pub struct Module {
    /// The name of the module.
    pub module: String,
    /// The size of the module.
    pub size: u64,
    /// What is using this module.
    pub used_by: Vec<String>
}

impl Module {
    /// Parse an individual /proc/modules-like line.
    pub fn parse(line: &str) -> io::Result<Module> {
        let mut parts = line.split(' ');

        let name = parts.next().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "module name not found"
        ))?;

        let size = parts.next().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "size not found"
        ))?;

        let used_by = parts.nth(1).ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "used_by not found"
        ))?;

        Ok(Module {
            module: name.to_string(),
            size: size.parse::<u64>().map_err(|_| io::Error::new(
                io::ErrorKind::InvalidData,
                "module size is not a number"
            ))?,
            used_by: if used_by == "-" {
                vec![]
            } else {
                used_by.split(',')
                    .map(String::from)
                    .filter(|x| !x.is_empty())
                    .collect()
            }
        })
    }

    /// Iteratively parse lines from a /proc/modules-like source.
    pub fn parse_from<'a, I: Iterator<Item = &'a str>>(lines: I) -> io::Result<Vec<Module>> {
        lines.map(Self::parse).collect()
    }

    /// Collect a list of modules active on the system
    pub fn all() -> io::Result<Vec<Module>> {
        ModuleIter::new()?.collect()
    }
}

/// Read module entries iteratively.
pub struct ModuleIter {
    file: BufReader<File>,
    buffer: String,
}

impl ModuleIter {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            file: BufReader::new(File::open("/proc/modules")?),
            buffer: String::with_capacity(512),
        })
    }
}

impl Iterator for ModuleIter {
    type Item = io::Result<Module>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        match self.file.read_line(&mut self.buffer) {
            Ok(read) if read == 0 => None,
            Ok(_) => Some(Module::parse(&self.buffer)),
            Err(why) => Some(Err(why))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"snd_hda_intel 40960 9 - Live 0x0000000000000000
snd_hda_codec 126976 4 snd_hda_codec_hdmi,snd_hda_codec_realtek,snd_hda_codec_generic,snd_hda_intel, Live 0x0000000000000000
snd_hda_core 81920 5 snd_hda_codec_hdmi,snd_hda_codec_realtek,snd_hda_codec_generic,snd_hda_intel,snd_hda_codec, Live 0x0000000000000000
nvidia_drm 40960 11 - Live 0x0000000000000000 (POE)"#;

    #[test]
    fn modules() {
        assert_eq!(
            Module::parse_from(SAMPLE.lines()).unwrap(),
            vec![
                Module {
                    module: "snd_hda_intel".into(),
                    size: 40960,
                    used_by: vec![]
                },
                Module {
                    module: "snd_hda_codec".into(),
                    size: 126_976,
                    used_by: vec![
                        "snd_hda_codec_hdmi".into(),
                        "snd_hda_codec_realtek".into(),
                        "snd_hda_codec_generic".into(),
                        "snd_hda_intel".into(),
                    ]
                },
                Module {
                    module: "snd_hda_core".into(),
                    size: 81920,
                    used_by: vec![
                        "snd_hda_codec_hdmi".into(),
                        "snd_hda_codec_realtek".into(),
                        "snd_hda_codec_generic".into(),
                        "snd_hda_intel".into(),
                        "snd_hda_codec".into(),
                    ]
                },
                Module {
                    module: "nvidia_drm".into(),
                    size: 40960,
                    used_by: vec![]
                },
            ]
        )
    }
}
