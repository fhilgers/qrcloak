// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{convert, fmt::Display, str::FromStr};

pub fn get_env<T: FromStr<Err = impl Display>>(name: &str) -> Result<T, clap::Error> {
    std::env::var(name)
        .map_err(|_| {
            let mut error = clap::Error::new(clap::error::ErrorKind::MissingRequiredArgument);
            error.insert(
                clap::error::ContextKind::InvalidArg,
                clap::error::ContextValue::Strings(vec![format!("env: {name}=")]),
            );
            error
        })
        .map(|v| {
            T::from_str(&v).map_err(|err| {
                let mut error = clap::Error::new(clap::error::ErrorKind::InvalidValue);
                error.insert(
                    clap::error::ContextKind::InvalidArg,
                    clap::error::ContextValue::String(name.to_string()),
                );
                error.insert(
                    clap::error::ContextKind::InvalidValue,
                    clap::error::ContextValue::String(format!("{err}")),
                );
                error
            })
        })
        .and_then(convert::identity)
}
