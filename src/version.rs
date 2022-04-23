// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

const APP_MAJOR: u8 = 0;
const APP_MINOR: u8 = 2;
const APP_PATCH: u8 = 1;
const APP_PRE_RELEASE: &str = "";

pub fn get_version() -> String {
    let mut version = format!("{}.{}.{}", APP_MAJOR, APP_MINOR, APP_PATCH);

    if !APP_PRE_RELEASE.is_empty() {
        version = format!("{}-{}", version, APP_PRE_RELEASE);
    }

    version
}
