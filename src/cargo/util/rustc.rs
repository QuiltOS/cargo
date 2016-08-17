use std::path::PathBuf;

use regex::Regex;

use util::{self, CargoResult, internal, ProcessBuilder};

pub struct Rustc {
    pub path: PathBuf,
    pub verbose_version: String,
    pub host: String,
    /// Backwards compatibility: does this compiler support `--cap-lints` flag?
    pub cap_lints: bool,
}

impl Rustc {
    /// Run the compiler at `path` to learn various pieces of information about
    /// it.
    ///
    /// If successful this function returns a description of the compiler along
    /// with a list of its capabilities.
    pub fn new(path: PathBuf) -> CargoResult<Rustc> {
        let mut cmd = util::process(&path);
        cmd.arg("-vV");

        let mut first = cmd.clone();
        first.arg("--cap-lints").arg("allow");

        let (cap_lints, output) =
            try!(first.exec_with_output().map(|output| (true, output))
            .or_else(|_| cmd.exec_with_output().map(|output| (false, output))));

        let verbose_version = try!(String::from_utf8(output.stdout).map_err(|_| {
            internal("rustc -v didn't return utf8 output")
        }));

        let host = try!(version_get(&verbose_version[..], "host").ok_or_else(|| {
            internal("rustc -v didn't have a line for `host:`")
        })).to_string();

        Ok(Rustc {
            path: path,
            verbose_version: verbose_version,
            host: host,
            cap_lints: cap_lints,
        })
    }

    pub fn version_get<'a>(&'a self, key: &str) -> Option<&'a str> {
        version_get(&self.verbose_version, key)
    }

    pub fn process(&self) -> ProcessBuilder {
        util::process(&self.path)
    }
}

fn version_get<'a>(verbose_version: &'a str, key: &str) -> Option<&'a str> {
    let regex = Regex::new(&*format!(r"^{}: (.*)$", key)).unwrap();

    verbose_version
        .lines()
        .filter_map(|l| regex.captures(l))
        .next()
        .and_then(|caps| caps.at(1))
}
