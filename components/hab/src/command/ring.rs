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

pub mod key {
    pub mod export {
        use std::io;
        use std::fs::File;
        use std::path::Path;

        use hcore::crypto::SymKey;

        use error::Result;

        pub fn start(ring: &str, cache: &Path) -> Result<()> {
            let latest = try!(SymKey::get_latest_pair_for(ring, cache));
            let path = try!(SymKey::get_secret_key_path(&latest.name_with_rev(), cache));
            let mut file = try!(File::open(&path));
            debug!("Streaming file contents of {} to standard out",
                   &path.display());
            try!(io::copy(&mut file, &mut io::stdout()));
            Ok(())
        }
    }

    pub mod generate {
        use std::path::Path;

        use ansi_term::Colour::{Blue, Yellow};
        use hcore::crypto::SymKey;

        use error::Result;

        pub fn start(ring: &str, cache: &Path) -> Result<()> {
            println!("{}",
                     Yellow.bold().paint(format!("» Generating ring key for {}", &ring)));
            let pair = try!(SymKey::generate_pair_for_ring(ring, cache));
            println!("{}",
                     Blue.paint(format!("★ Generated ring key pair {}.", &pair.name_with_rev())));
            Ok(())
        }
    }

    pub mod import {
        use std::path::Path;

        use ansi_term::Colour::{Blue, Yellow};
        use hcore::crypto::SymKey;

        use error::Result;

        pub fn start(content: &str, cache: &Path) -> Result<()> {
            println!("{}",
                     Yellow.bold().paint(format!("» Importing ring key from standard input")));
            let (pair, pair_type) = try!(SymKey::write_file_from_str(content, cache));
            println!("{}",
                     Blue.paint(format!("★ Imported {} ring key {}.",
                                        &pair_type,
                                        &pair.name_with_rev())));
            Ok(())
        }
    }
}
