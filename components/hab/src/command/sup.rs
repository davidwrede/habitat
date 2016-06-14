// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ffi::OsString;

use error::Result;

pub fn start(args: Vec<OsString>) -> Result<()> {
    inner::start(args)
}

#[cfg(target_os = "linux")]
mod inner {
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::str::FromStr;

    use hcore::crypto::{init, default_cache_key_path};
    use hcore::env as henv;
    use hcore::fs::find_command;
    use hcore::package::PackageIdent;

    use error::{Error, Result};
    use exec;

    const SUP_CMD: &'static str = "hab-sup";
    const SUP_CMD_ENVVAR: &'static str = "HAB_SUP_BINARY";
    const SUP_PACKAGE_IDENT: &'static str = "core/hab-sup";

    const FEAT_STATIC: &'static str = "HAB_FEAT_SUP_STATIC";
    const SUP_STATIC_PACKAGE_IDENT: &'static str = "core/hab-sup-static";

    pub fn start(args: Vec<OsString>) -> Result<()> {
        let sup_ident = match henv::var(FEAT_STATIC) {
            Ok(_) => {
                debug!("Enabling statically compiled Supervisor from {}",
                       SUP_STATIC_PACKAGE_IDENT);
                SUP_STATIC_PACKAGE_IDENT
            }
            Err(_) => SUP_PACKAGE_IDENT,
        };
        let command = match henv::var(SUP_CMD_ENVVAR) {
            Ok(command) => PathBuf::from(command),
            Err(_) => {
                init();
                let ident = try!(PackageIdent::from_str(sup_ident));
                try!(exec::command_from_pkg(SUP_CMD, &ident, &default_cache_key_path(None), 0))
            }
        };

        if let Some(cmd) = find_command(command.to_string_lossy().as_ref()) {
            exec::exec_command(cmd, args)
        } else {
            Err(Error::ExecCommandNotFound(command.to_string_lossy().into_owned()))
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod inner {
    use std::env;
    use std::ffi::OsString;

    use ansi_term::Colour::Yellow;

    use error::{Error, Result};

    pub fn start(_args: Vec<OsString>) -> Result<()> {
        let subcmd = env::args().nth(1).unwrap_or("<unknown>".to_string());
        let msg = format!("∅ Launching a native Supervisor on this operating system is not yet \
                           supported. Try running this command again on a 64-bit Linux \
                           operating system.\n");
        println!("{}", Yellow.bold().paint(msg));
        Err(Error::SubcommandNotSupported(subcmd))
    }
}
