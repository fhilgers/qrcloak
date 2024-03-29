use std::{any::Any, ffi::c_void, sync::Arc};

use uniffi::FfiConverter;

extern "C" fn foo<UT, T: FfiConverter<UT>>(value: T) {}
