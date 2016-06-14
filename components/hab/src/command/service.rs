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
    pub mod generate {
        use std::path::Path;

        use ansi_term::Colour::{Blue, Yellow};
        use hcore::crypto::BoxKeyPair;
        use hcore::service::ServiceGroup;

        use error::Result;

        pub fn start(org: &str, service_group: &ServiceGroup, cache: &Path) -> Result<()> {
            println!("{}",
                     Yellow.bold().paint(format!("» Generating service key for {} in {}",
                                                 &service_group,
                                                 org)));
            let pair =
                try!(BoxKeyPair::generate_pair_for_service(org, &service_group.to_string(), cache));
            println!("{}",
                     Blue.paint(format!("★ Generated service key pair {}.",
                                        &pair.name_with_rev())));
            Ok(())
        }
    }
}
