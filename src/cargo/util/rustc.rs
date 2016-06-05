use std::path::Path;

use regex::Regex;

use util::{self, CargoResult, internal, ChainError};

pub struct Rustc {
    pub verbose_version: String,
    pub host: String,
    pub cap_lints: bool,
}

impl Rustc {
    /// Run the compiler at `path` to learn varioues pieces of information about
    /// it.
    ///
    /// If successful this function returns a description of the compiler along
    /// with a list of its capabilities.
    pub fn new<P: AsRef<Path>>(path: P) -> CargoResult<Rustc> {
        let mut cmd = util::process(path.as_ref());
        cmd.arg("-vV");

        let mut ret = Rustc::blank();
        let mut first = cmd.clone();
        first.arg("--cap-lints").arg("allow");
        let output = match first.exec_with_output() {
            Ok(output) => { ret.cap_lints = true; output }
            Err(..) => try!(cmd.exec_with_output()),
        };
        ret.verbose_version = try!(String::from_utf8(output.stdout).map_err(|_| {
            internal("rustc -v didn't return utf8 output")
        }));
        ret.host = try!({
            version_get(&*ret.verbose_version, "host")
                .chain_error(|| {
                    internal("rustc -v didn't have a line for `host:`")
                })
        }).to_string();
        Ok(ret)
    }

    pub fn version_get<'a>(&'a self, key: &str) -> Option<&'a str> {
        version_get(&self.verbose_version, key)
    }

    pub fn blank() -> Rustc {
        Rustc {
            verbose_version: String::new(),
            host: String::new(),
            cap_lints: false,
        }
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
