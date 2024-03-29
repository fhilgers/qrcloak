use std::any::TypeId;
use std::collections::btree_map::Values;
use std::ffi::c_uchar;
use std::ops::Add;
use std::pin::Pin;
use std::slice::from_raw_parts;
use std::{sync::Mutex, vec::IntoIter};

use std::{
    any::Any,
    ffi::c_void,
    fmt::Debug,
    marker::{self, PhantomData},
    num::Wrapping,
    str::from_utf8,
    sync::{Arc, RwLock},
};

use const_format::formatcp;
use dyn_clone::DynClone;
use futures::stream::Iter;
use futures::{Future, StreamExt};
use futures_signals::signal::{
    Mutable, MutableSignal, MutableSignalCloned, Signal, SignalExt, SignalStream,
};
use log::{Metadata, MetadataBuilder};
use qrypt_core::*;
use quircs::{Code, DecodeError, ExtractError, Quirc};
use uniffi::{
    custom_newtype, custom_type, deps::bytes::BufMut, derive_ffi_traits, metadata, FfiConverter,
    FfiConverterArc, FfiDefault, Lift, LiftRef, LiftReturn, Lower, LowerReturn, MetadataBuffer,
    Result, RustBuffer,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum MyError {
    #[error(transparent)]
    MyLibError(#[from] MyLibError),

    #[error(transparent)]
    ExtractError(#[from] ExtractError),

    #[error(transparent)]
    DecodeError(#[from] DecodeError),
}

#[uniffi::export]
pub fn is_payload(input: &str) -> bool {
    qrypt_core::is_payload(input)
}

#[uniffi::export]
pub fn encrypt_to_b64png(input: &str, password: &str) -> Result<String, MyError> {
    Ok(qrypt_core::encrypt_to_b64png(input, password)?)
}

#[uniffi::export]
pub fn encrypt_to_b64(input: &str, password: &str) -> Result<String, MyError> {
    Ok(qrypt_core::encrypt_to_b64(input, password)?)
}

#[uniffi::export]
pub fn decrypt_b64payload(input: &str, password: &str) -> Result<String, MyError> {
    Ok(qrypt_core::decrypt_b64payload(input, password)?)
}

#[derive(Debug, uniffi::Object)]
pub struct Decoder {
    inner: Mutex<Quirc>,
}

#[derive(Debug, uniffi::Enum)]
pub enum QrCode {
    Decoded { corners: Corners, payload: Vec<u8> },
    NotDecodable { corners: Corners },
}

impl From<&Code> for QrCode {
    fn from(value: &Code) -> Self {
        match value.decode() {
            Ok(v) => QrCode::Decoded {
                corners: value.corners.into(),
                payload: v.payload,
            },
            Err(_) => QrCode::NotDecodable {
                corners: value.corners.into(),
            },
        }
    }
}

impl From<Code> for QrCode {
    fn from(value: Code) -> Self {
        Self::from(&value)
    }
}

#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct Corners {
    pub top_left: Point,
    pub top_right: Point,
    pub bottom_right: Point,
    pub bottom_left: Point,
}

impl From<&[Point; 4]> for Corners {
    fn from(value: &[Point; 4]) -> Self {
        Self {
            top_left: value[0],
            top_right: value[1],
            bottom_right: value[2],
            bottom_left: value[3],
        }
    }
}

impl From<[Point; 4]> for Corners {
    fn from(value: [Point; 4]) -> Self {
        Self::from(&value)
    }
}

impl From<&[quircs::Point; 4]> for Corners {
    fn from(value: &[quircs::Point; 4]) -> Self {
        Self {
            top_left: value[0].into(),
            top_right: value[1].into(),
            bottom_right: value[2].into(),
            bottom_left: value[3].into(),
        }
    }
}

impl From<[quircs::Point; 4]> for Corners {
    fn from(value: [quircs::Point; 4]) -> Self {
        Self::from(&value)
    }
}

#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl From<&quircs::Point> for Point {
    fn from(value: &quircs::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<quircs::Point> for Point {
    fn from(value: quircs::Point) -> Self {
        Self::from(&value)
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl Decoder {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Quirc::default()),
        }
    }

    pub fn decode(&self, image: &[u8], width: u32, height: u32) -> Vec<QrCode> {
        let mut guard = self.inner.lock().unwrap();

        let codes = guard.identify(width as usize, height as usize, image);

        // We can unwrap as the only error that is thrown is when the count is out of
        // bounds. CodeIter<'_> checks that for us.
        codes
            .map(Result::unwrap)
            .map(QrCode::from)
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, uniffi::Record)]
pub struct EncodedQrCode {
    data: Vec<u8>,
    size: u16,
}

#[uniffi::export]
pub fn qr_encode(data: &str) -> Result<EncodedQrCode, MyError> {
    let qrcode = qrypt_core::qr_encode(data)?;

    let mut raw_data = Vec::with_capacity(qrcode.size() as usize * qrcode.size() as usize);

    for y in 0..qrcode.size() {
        for x in 0..qrcode.size() {
            if qrcode.get_module(x, y) {
                raw_data.push(1u8)
            } else {
                raw_data.push(0u8)
            }
        }
    }

    Ok(EncodedQrCode {
        data: raw_data,
        size: qrcode.size() as u16,
    })
}

uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
struct Object {
    inner: Vec<u8>,
}

//#[derive(uniffi::Object)]
//struct IteratorWrapperIntoIterU16(IteratorWrapper<IntoIter<u16>>);

/*
#[uniffi::export]
impl IteratorWrapperIntoIterU16 {
    fn next(&self) -> Option<u16> {
        self.0.next()
    }
}
*/

#[macro_export]
macro_rules! struct_inner {
    ($head:ident) => {
        $head
    };
    ($head:ident, $($tail:ident),*) => {
        $head<struct_inner!($($tail),*)>
    };
}

#[macro_export]
macro_rules! struct_name {
    ($($id:ident),*) => {
        paste::paste! {
            [< $( $id  )* >]
        }
    };
}

#[macro_export]
macro_rules! struct_definition {
    ($($id:ident),*) => {
        paste::paste! {
            #[derive(uniffi::Object)]
            struct  [< $( $id  )* >] (struct_inner!($($id),*));
        }


    };
}

#[macro_export]
macro_rules! gen {
    ($($id:ident),*) => {


        struct_definition!($($id),*);

        const _: () = {

            type Inner = struct_inner!($($id),*);
            type Outer = struct_name!($($id),*);

            impl UniffiCustomTypeConverter for Inner {
                type Builtin = Arc<Outer>;

                fn into_custom(val:Self::Builtin) -> uniffi::Result<Self>where Self:Sized {
                    Ok(Arc::into_inner(val).unwrap().0)
                }

                fn from_custom(obj:Self) -> Self::Builtin {
                    Arc::new(struct_name!($($id),*)(obj))
                }
            }

            unsafe impl ::uniffi::FfiConverter<crate::UniFfiTag> for Inner {
                type FfiType = <Arc<Outer> as ::uniffi::Lower<
                    crate::UniFfiTag,
                >>::FfiType;
                fn lower(obj: Inner) -> Self::FfiType {
                    <Arc<Outer> as ::uniffi::Lower<
                        crate::UniFfiTag,
                    >>::lower(
                        <Inner as crate::UniffiCustomTypeConverter>::from_custom(obj),
                    )
                }
                fn try_lift(v: Self::FfiType) -> uniffi::Result<Inner> {
                    <Inner as crate::UniffiCustomTypeConverter>::into_custom(
                        <Arc<Outer> as ::uniffi::Lift<
                            crate::UniFfiTag,
                        >>::try_lift(v)?,
                    )
                }
                fn write(obj: Inner, buf: &mut Vec<u8>) {
                    <Arc<Outer> as ::uniffi::Lower<
                        crate::UniFfiTag,
                    >>::write(
                        <Inner  as crate::UniffiCustomTypeConverter>::from_custom(obj),
                        buf,
                    );
                }
                fn try_read(buf: &mut &[u8]) -> uniffi::Result<Inner> {
                    <Inner as crate::UniffiCustomTypeConverter>::into_custom(
                        <Arc<Outer> as ::uniffi::Lift<
                            crate::UniFfiTag,
                        >>::try_read(buf)?,
                    )
                }


                const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
                        ::uniffi::metadata::codes::TYPE_CUSTOM,
                    )
                    .concat_str("qrypt_uniffi")
                    .concat_str(paste::paste!( stringify!([< $( $id  )* Inner>])))
                    .concat(
                        <Arc<Outer> as ::uniffi::Lower<
                            crate::UniFfiTag,
                        >>::TYPE_ID_META,
                    );
            }

            unsafe impl ::uniffi::Lower<crate::UniFfiTag> for Inner {
                type FfiType = <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::FfiType;
                fn lower(obj: Self) -> Self::FfiType {
                    <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::lower(obj)
                }
                fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                    <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(obj, buf)
                }
                const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::FfiConverter<
                    crate::UniFfiTag,
                >>::TYPE_ID_META;
            }
            unsafe impl ::uniffi::Lift<crate::UniFfiTag> for Inner {
                type FfiType = <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::FfiType;
                fn try_lift(v: Self::FfiType) -> ::uniffi::deps::anyhow::Result<Self> {
                    <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_lift(v)
                }
                fn try_read(buf: &mut &[u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                    <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_read(buf)
                }
                const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::FfiConverter<
                    crate::UniFfiTag,
                >>::TYPE_ID_META;
            }
            unsafe impl ::uniffi::LowerReturn<crate::UniFfiTag> for Inner {
                type ReturnType = <Self as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
                fn lower_return(
                    obj: Self,
                ) -> ::uniffi::deps::anyhow::Result<
                    Self::ReturnType,
                    ::uniffi::RustBuffer,
                > {
                    Ok(<Self as ::uniffi::Lower<crate::UniFfiTag>>::lower(obj))
                }
                const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::Lower<
                    crate::UniFfiTag,
                >>::TYPE_ID_META;
            }
            unsafe impl ::uniffi::LiftReturn<crate::UniFfiTag> for Inner {
                fn lift_callback_return(buf: ::uniffi::RustBuffer) -> Self {
                    <Self as ::uniffi::Lift<crate::UniFfiTag>>::try_lift_from_rust_buffer(buf)
                        .expect("Error reading callback interface result")
                }
                const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::Lift<
                    crate::UniFfiTag,
                >>::TYPE_ID_META;
            }
            unsafe impl ::uniffi::LiftRef<crate::UniFfiTag> for Inner {
                type LiftType = Self;
            }
            impl ::uniffi::ConvertError<crate::UniFfiTag> for Inner {
                fn try_convert_unexpected_callback_error(
                    e: ::uniffi::UnexpectedUniFFICallbackError,
                ) -> ::uniffi::deps::anyhow::Result<Self> {
                    {
                        pub trait GetConverterGeneric {
                            fn get_converter(
                                &self,
                            ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric;
                        }
                        impl<T> GetConverterGeneric for &T {
                            fn get_converter(
                                &self,
                            ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric {
                                ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric
                            }
                        }
                        pub trait GetConverterSpecialized {
                            fn get_converter(
                                &self,
                            ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized;
                        }
                        impl<T: Into<Inner>> GetConverterSpecialized for T {
                            fn get_converter(
                                &self,
                            ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized {
                                ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized
                            }
                        }
                        (&e).get_converter().try_convert_unexpected_callback_error(e)
                    }
                }
            }
        };
    };
}

struct Wrapper<T>(T);

impl<T> From<T> for Wrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

struct MutexWrapper<T>(Mutex<T>);

impl<T> From<T> for MutexWrapper<T> {
    fn from(value: T) -> Self {
        Self(Mutex::new(value))
    }
}

impl<T> From<Mutex<T>> for MutexWrapper<T> {
    fn from(value: Mutex<T>) -> Self {
        Self(value)
    }
}

struct IteratorWrapper<I: Iterator>(MutexWrapper<I>);

impl<I: Iterator> IteratorWrapper<I> {
    fn next(&self) -> Option<I::Item> {
        self.0 .0.lock().ok()?.next()
    }
}

impl<I: Iterator> From<I> for IteratorWrapper<I> {
    fn from(value: I) -> Self {
        Self(MutexWrapper::from(value))
    }
}

//gen!(IteratorWrapper, IntoIter, u16);
//gen!(IteratorWrapper, IntoIter, u8);

mod any_experiments;

struct AnyLowerReturn {
    lower_return: fn(Box<dyn Any>) -> uniffi::deps::anyhow::Result<RustBuffer, RustBuffer>,
}

impl AnyLowerReturn {
    fn new<UT, T: LowerReturn<UT, ReturnType = RustBuffer> + 'static>() -> Self {
        let lower_return = |any: Box<dyn Any>| T::lower_return(*any.downcast::<T>().unwrap());

        Self {
            lower_return: lower_return,
        }
    }
}

struct AnyLift {
    try_lift: fn(buf: RustBuffer) -> AnyLowerReturn,
}

fn do_stuff(input: Vec<u8>) -> Vec<String> {
    vec!["hello".to_string()]
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn uniffi_qrypt_uniffi_fn_func_do_stuff(
    input: RustBuffer,
    call_status: &mut ::uniffi::RustCallStatus,
) -> <Vec<String> as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
    {
        let lvl = ::log::Level::Debug;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api::log(
                format_args!("do_stuff"),
                lvl,
                &("qrypt_uniffi", "qrypt_uniffi", "qrypt-uniffi/src/lib.rs"),
                507u32,
                (),
            );
        }
    };
    let uniffi_lift_args = move || {
        Ok((
            match <Vec<u8> as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(input) {
                Ok(v) => v,
                Err(e) => return Err(("input", e)),
            },
        ))
    };

    let ret: AnyLowerReturn = AnyLowerReturn::new::<crate::UniFfiTag, Vec<String>>();

    ::uniffi::rust_call(call_status, || {
        (ret.lower_return)(match uniffi_lift_args() {
            Ok(uniffi_args) => Box::new(do_stuff(uniffi_args.0)),
            Err((arg_name, anyhow_error)) => Box::new(<Vec<String> as ::uniffi::LowerReturn<
                crate::UniFfiTag,
            >>::handle_failed_lift(
                arg_name, anyhow_error
            )),
        })
    })
}
const UNIFFI_META_CONST_QRYPT_UNIFFI_FUNC_DO_STUFF: ::uniffi::MetadataBuffer =
    ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::FUNC)
        .concat_str("qrypt_uniffi")
        .concat_str("do_stuff")
        .concat_bool(false)
        .concat_value(1u8)
        .concat_str("input")
        .concat(<Vec<u8> as ::uniffi::Lift<crate::UniFfiTag>>::TYPE_ID_META)
        .concat(<Vec<String> as ::uniffi::LowerReturn<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_long_str("");
#[no_mangle]
#[doc(hidden)]
pub static UNIFFI_META_QRYPT_UNIFFI_FUNC_DO_STUFF: [u8;
    UNIFFI_META_CONST_QRYPT_UNIFFI_FUNC_DO_STUFF.size] =
    UNIFFI_META_CONST_QRYPT_UNIFFI_FUNC_DO_STUFF.into_array();
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn uniffi_qrypt_uniffi_checksum_func_do_stuff() -> u16 {
    UNIFFI_META_CONST_QRYPT_UNIFFI_FUNC_DO_STUFF.checksum()
}

impl AnyWrapper {
    fn new<UT, T: LowerReturn<UT, ReturnType = RustBuffer> + 'static>(
        inner: impl Any + Send + Sync + 'static,
    ) -> Self {
        let lower_return = AnyLowerReturn::new::<UT, T>();
        let boxed = Box::new(Mutex::new(inner));

        Self(boxed, lower_return)
    }
}

#[derive(uniffi::Object)]
struct AnyWrapper(Box<Mutex<dyn Any + Send + Sync + 'static>>, AnyLowerReturn);

/*
unsafe impl<I: Iterator + Send + Sync + 'static> ::uniffi::FfiConverter<crate::UniFfiTag> for FfiIterator<I> {
    type FfiType = <Arc<AnyWrapper> as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
    fn lower(obj: FfiIterator<I>) -> Self::FfiType {
        <Arc<
            AnyWrapper,
        > as ::uniffi::Lower<
            crate::UniFfiTag,
        >>::lower(<FfiIterator<I> as crate::UniffiCustomTypeConverter>::from_custom(obj))
    }
    fn try_lift(v: Self::FfiType) -> uniffi::Result<FfiIterator<I>> {
        <FfiIterator<I> as crate::UniffiCustomTypeConverter>::into_custom(
            <Arc<AnyWrapper> as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(v)?,
        )
    }
    fn write(obj: FfiIterator<I>, buf: &mut Vec<u8>) {
        <Arc<
            AnyWrapper,
        > as ::uniffi::Lower<
            crate::UniFfiTag,
        >>::write(
            <FfiIterator<I> as crate::UniffiCustomTypeConverter>::from_custom(obj),
            buf,
        );
    }
    fn try_read(buf: &mut &[u8]) -> uniffi::Result<FfiIterator<I>> {
        <FfiIterator<I> as crate::UniffiCustomTypeConverter>::into_custom(
            <Arc<AnyWrapper> as ::uniffi::Lift<crate::UniFfiTag>>::try_read(buf)?,
        )
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::TYPE_CUSTOM,
        )
        .concat_str("qrypt_uniffi")
        .concat_str("FfiIterator")
        .concat(<Arc<AnyWrapper> as ::uniffi::Lower<crate::UniFfiTag>>::TYPE_ID_META);
}


unsafe impl<I: Iterator + Send + Sync + 'static> ::uniffi::Lower<crate::UniFfiTag> for FfiIterator<I> {
    type FfiType = <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::FfiType;
    fn lower(obj: Self) -> Self::FfiType {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::lower(obj)
    }
    fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(obj, buf)
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::FfiConverter<
        crate::UniFfiTag,
    >>::TYPE_ID_META;
}
unsafe impl<I: Iterator + Send + Sync + 'static> ::uniffi::Lift<crate::UniFfiTag> for FfiIterator<I> {
    type FfiType = <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::FfiType;
    fn try_lift(v: Self::FfiType) -> ::uniffi::deps::anyhow::Result<Self> {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_lift(v)
    }
    fn try_read(buf: &mut &[u8]) -> ::uniffi::deps::anyhow::Result<Self> {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_read(buf)
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::FfiConverter<
        crate::UniFfiTag,
    >>::TYPE_ID_META;
}
unsafe impl<I: Iterator + Send + Sync + 'static> ::uniffi::LowerReturn<crate::UniFfiTag> for FfiIterator<I> {
    type ReturnType = <Self as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
    fn lower_return(
        obj: Self,
    ) -> ::uniffi::deps::anyhow::Result<
        Self::ReturnType,
        ::uniffi::RustBuffer,
    > {
        Ok(<Self as ::uniffi::Lower<crate::UniFfiTag>>::lower(obj))
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::Lower<
        crate::UniFfiTag,
    >>::TYPE_ID_META;
}
unsafe impl<I: Iterator + Send + Sync + 'static> ::uniffi::LiftReturn<crate::UniFfiTag> for FfiIterator<I> {
    fn lift_callback_return(buf: ::uniffi::RustBuffer) -> Self {
        <Self as ::uniffi::Lift<crate::UniFfiTag>>::try_lift_from_rust_buffer(buf)
            .expect("Error reading callback interface result")
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer = <Self as ::uniffi::Lift<
        crate::UniFfiTag,
    >>::TYPE_ID_META;
}
unsafe impl<I: Iterator + Send + Sync + 'static> ::uniffi::LiftRef<crate::UniFfiTag> for FfiIterator<I> {
    type LiftType = Self;
}
impl<I: Iterator + Send + Sync + 'static> ::uniffi::ConvertError<crate::UniFfiTag> for FfiIterator<I> {
    fn try_convert_unexpected_callback_error(
        e: ::uniffi::UnexpectedUniFFICallbackError,
    ) -> ::uniffi::deps::anyhow::Result<Self> {
        {
            pub trait GetConverterGeneric {
                fn get_converter(
                    &self,
                ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric;
            }
            impl<T> GetConverterGeneric for &T {
                fn get_converter(
                    &self,
                ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric {
                    ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric
                }
            }
            pub trait GetConverterSpecialized<I> {
                fn get_converter(
                    &self,
                ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized;
            }
            impl<I: Iterator + Send + Sync + 'static, T: Into<FfiIterator<I>>> GetConverterSpecialized<I> for T {
                fn get_converter(
                    &self,
                ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized {
                    ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized
                }
            }
            (&e).get_converter().try_convert_unexpected_callback_error(e)
        }
    }
}


#[derive(uniffi::Object)]
struct Blubs(Vec<String>);

#[uniffi::export]
impl Blubs {
    #[uniffi::constructor]
    pub fn new(data: &[String]) -> Self {
        Self (data.to_vec())
    }

    pub fn iter(&self) -> FfiIterator<IntoIter<String>> {
        FfiIterator::new(self.0.clone().into_iter())
    }
}


struct FfiIterator<I: Iterator>(Arc<AnyWrapper>, PhantomData<I>);



impl<I: Iterator + Send + Sync + 'static> FfiIterator<I> {
    fn next(&self) -> Option<<I as Iterator>::Item> {
        let mut guard = self.0.0.lock().unwrap();
        let iter = guard.downcast_mut::<I>().unwrap();

        iter.next()
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn uniffi_qrypt_uniffi_fn_method_ffiiterator_next(
    uniffi_self_lowered: *const c_void,
    call_status: &mut ::uniffi::RustCallStatus,
) -> RustBuffer {

    {
        let lvl = ::log::Level::Debug;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api::log(
                format_args!("next"),
                lvl,
                &("qrypt_uniffi", "qrypt_uniffi", "qrypt-uniffi/src/lib.rs"),
                768u32,
                (),
            );
        }
    };

    let uniffi_lift_args = move || Ok((
        match <Arc<AnyWrapper>
        as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
            Ok(v) => v,
            Err(e) => return Err(("self", e)),
        },
    ));


    ::uniffi::rust_call(
        call_status,
        || {
                match uniffi_lift_args() {
                    Ok(uniffi_args) => {
                        let lower = uniffi_args.0.1;
                        let guard = uniffi_args.0.0.lock().unwrap();

                        let iter = guard.downcast_mut::<&dyn Iterator<Item = &dyn Any>>().unwrap();

                        (lower.lower_return)(iter.next())
                    },
                    Err((arg_name, anyhow_error)) => {
                        Err(RustBuffer::new())
                    }
                }


        },
    )
}
const UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_FFIITERATOR_NEXT: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
        ::uniffi::metadata::codes::METHOD,
    )
    .concat_str("qrypt_uniffi")
    .concat_str("FfiIterator")
    .concat_str("next")
    .concat_bool(false)
    .concat_value(0u8)
    .concat(
        <Option<AnyWrapper,
        > as ::uniffi::LowerReturn<crate::UniFfiTag>>::TYPE_ID_META,
    )
    .concat_long_str("");
#[no_mangle]
#[doc(hidden)]
pub static UNIFFI_META_QRYPT_UNIFFI_METHOD_FFIITERATOR_NEXT: [u8; UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_FFIITERATOR_NEXT
    .size] = UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_FFIITERATOR_NEXT.into_array();
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn uniffi_qrypt_uniffi_checksum_method_ffiiterator_next() -> u16 {
    UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_FFIITERATOR_NEXT.checksum()
}


impl<I: Iterator + Send + Sync + 'static> FfiIterator<I> where I::Item: LowerReturn<crate::UniFfiTag, ReturnType = RustBuffer>{
    fn new(iter: I) -> Self {
        Self(Arc::new(AnyWrapper::new::<crate::UniFfiTag, Option<I::Item>>(iter)), PhantomData)
    }
}


impl<I: Iterator + Send + Sync + 'static> UniffiCustomTypeConverter for FfiIterator<I> {
    type Builtin = Arc<AnyWrapper>;

    fn into_custom(val:Self::Builtin) -> uniffi::Result<Self>where Self:Sized {
        Ok(Self(val, PhantomData))
    }

    fn from_custom(obj:Self) -> Self::Builtin {
        obj.0
    }
}

*/

/*
struct MutableWrapper<T: Clone>(Mutable<T>);



impl<T: Clone> MutableWrapper<T> {

    fn new(value: T) -> Self {
        Self(Mutable::new(value))
    }

    fn set(&self, value: T) {
        self.0.set(value)
    }

    fn get(&self) -> T {
        self.0.get_cloned()
    }

    fn stream(&self) -> StreamWrapper<T> {
        self.0.signal_cloned().to_stream().into()
    }
}




impl<T: Clone> From<SignalStream<MutableSignalCloned<T>>> for StreamWrapper<T> {
    fn from(value: SignalStream<MutableSignalCloned<T>>) -> Self {
        Self(futures::lock::Mutex::new(value))
    }
}



#[uniffi::export]
impl StreamWrapperString {
    async fn poll_next(&self) -> Option<String> {
        self.0.lock().await.next().await
    }
}

#[derive(uniffi::Object)]
pub struct MutableWrapperString(MutableWrapper<String>);

*/

/*
#[uniffi::export]
impl MutableWrapperString {

    #[uniffi::constructor]
    fn new(value: String) -> Self {
        Self(MutableWrapper::new(value))
    }

    fn set(&self, value: String) {
        self.0.set(value)
    }

    fn get(&self) -> String {
        self.0.get()
    }

    fn stream(&self) -> StreamWrapper<String> {
        self.0.stream()
    }
}
*/

macro_rules! custom_type_generic {
    ($from:ty, $to:ty) => {
        #[doc(hidden)]
        const _: () = {
            type Wrapper = $to;
            type Inner = $from;

            #[automatically_derived]
            impl crate::UniffiCustomTypeConverter for Inner {
                type Builtin = Arc<Wrapper>;

                fn into_custom(val: Self::Builtin) -> uniffi::Result<Self>
                where
                    Self: Sized,
                {
                    match Arc::into_inner(val) {
                        Some(inner) => Ok(Self::from(inner)),
                        None => ::uniffi::deps::anyhow::bail!("Could not consume Arc"),
                    }
                }

                fn from_custom(obj: Self) -> Self::Builtin {
                    Arc::new(obj.into())
                }
            }

            #[automatically_derived]
            unsafe impl ::uniffi::FfiConverter<crate::UniFfiTag> for Inner {
                type FfiType = <Arc<Wrapper> as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
                fn lower(obj: Inner) -> Self::FfiType {
                    <Arc<Wrapper> as ::uniffi::Lower<crate::UniFfiTag>>::lower(
                        <Inner as crate::UniffiCustomTypeConverter>::from_custom(obj),
                    )
                }
                fn try_lift(v: Self::FfiType) -> uniffi::Result<Inner> {
                    <Inner as crate::UniffiCustomTypeConverter>::into_custom(
                        <Arc<Wrapper> as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(v)?,
                    )
                }
                fn write(obj: Inner, buf: &mut Vec<u8>) {
                    <Arc<Wrapper> as ::uniffi::Lower<crate::UniFfiTag>>::write(
                        <Inner as crate::UniffiCustomTypeConverter>::from_custom(obj),
                        buf,
                    );
                }
                fn try_read(buf: &mut &[u8]) -> uniffi::Result<Inner> {
                    <Inner as crate::UniffiCustomTypeConverter>::into_custom(
                        <Arc<Wrapper> as ::uniffi::Lift<crate::UniFfiTag>>::try_read(buf)?,
                    )
                }

                const TYPE_ID_META: ::uniffi::MetadataBuffer =
                    ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_CUSTOM)
                        .concat_str("qrypt_uniffi")
                        .concat_str(paste::paste! { stringify!([< Wrapper $to >]) })
                        .concat(<Arc<Wrapper> as ::uniffi::Lower<crate::UniFfiTag>>::TYPE_ID_META);
            }

            uniffi::derive_ffi_traits!(impl Lower<crate::UniFfiTag> for Inner);
            uniffi::derive_ffi_traits!(impl Lift<crate::UniFfiTag> for Inner);
            uniffi::derive_ffi_traits!(impl LowerReturn<crate::UniFfiTag> for Inner);
            uniffi::derive_ffi_traits!(impl LiftReturn<crate::UniFfiTag> for Inner);
            uniffi::derive_ffi_traits!(impl LiftRef<crate::UniFfiTag> for Inner);

            uniffi::derive_ffi_traits!(impl ConvertError<crate::UniFfiTag> for Inner);
        };
    };
}

macro_rules! gen_wrapper_no_mutex {
    ($type:ty => $name:path) => {
        paste::paste! {
            #[derive(::uniffi::Object)]
            pub struct $name ($type);
        }

        #[doc(hidden)]
        const _: () = {
            type Inner = $type;
            type Wrapper = $name;

            #[automatically_derived]
            impl From<Inner> for Wrapper {
                fn from(value: Inner) -> Self {
                    Self(value)
                }
            }

            #[automatically_derived]
            impl From<Wrapper> for Inner {
                fn from(value: Wrapper) -> Self {
                    value.0
                }
            }
        };

        custom_type_generic!($type, $name);
    };
}

macro_rules! gen_wrapper_mutex {
    ($type:ty => $name:path) => {
        paste::paste! {

            #[derive(::uniffi::Object)]
            pub struct $name (::futures::lock::Mutex<$type>);

        }

        #[doc(hidden)]
        const _: () = {
            type Inner = $type;
            type Wrapper = $name;

            #[automatically_derived]
            impl From<Inner> for Wrapper {
                fn from(value: Inner) -> Self {
                    Self(::futures::lock::Mutex::new(value))
                }
            }

            #[automatically_derived]
            impl From<Wrapper> for Inner {
                fn from(value: Wrapper) -> Self {
                    value.0.into_inner()
                }
            }
        };

        custom_type_generic!($type, $name);
    };
}

macro_rules! gen_wrapper {

    (Mutex, $type:ty => $name:path, $b:expr) => {

        gen_wrapper_mutex!($type => $name);

        paste::paste! {
            #[uniffi::export]
            impl $name $b
        }

    };


    ($type:ty => $name:path, $b:expr) => {

        gen_wrapper_no_mutex!($type => $name);

        paste::paste! {
            #[uniffi::export]
            impl $name $b
        }
    };





}

gen_wrapper!(Mutex,
    SignalStream<MutableSignalCloned<String>> => StringStream,
    {
        async fn poll_next(&self) -> Option<String> {
            self.0.lock().await.next().await
        }
    }
);

gen_wrapper!(Mutex,
    SignalStream<MutableSignal<u8>> => U8Stream,
    {
        async fn poll_next(&self) -> Option<u8> {
            self.0.lock().await.next().await
        }
    }
);

gen_wrapper!(Mutable<String> => MutableString, {
    #[uniffi::constructor]
    fn new() -> Self {
        Self(Mutable::new("hello".to_string()))
    }
});

#[derive(uniffi::Enum)]
enum Msg {
    Increment,
    Decrement,
}

gen_wrapper!(
    Mutex,
    SignalStream<MutableSignal<i32>> => I32Stream,
    {
        async fn poll_next(&self) -> Option<i32> {
            self.0.lock().await.next().await
        }
    }
);

#[derive(uniffi::Object)]
struct WrapperViewModel(ViewModelInner);

#[uniffi::export]
impl WrapperViewModel {
    fn counter(&self) -> SignalStream<MutableSignal<i32>> {
        self.0.counter.signal().to_stream()
    }

    fn update(&self, msg: Msg) {
        match msg {
            Msg::Increment => {
                self.0.counter.replace_with(|v| *v + 1);
            }
            Msg::Decrement => {
                self.0.counter.replace_with(|v| *v - 1);
            }
        };
    }
}

custom_type!(ViewModelInner, Arc<WrapperViewModel>);

struct ViewModelInner {
    counter: Mutable<i32>,
}

impl UniffiCustomTypeConverter for ViewModelInner {
    type Builtin = Arc<WrapperViewModel>;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self>
    where
        Self: Sized,
    {
        Ok(Arc::into_inner(val).unwrap().0)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        Arc::new(WrapperViewModel(obj))
    }
}

#[uniffi::export]
fn ViewModel(initial: i32) -> ViewModelInner {
    ViewModelInner {
        counter: Mutable::new(initial),
    }
}

//struct WrapperViewModel(Arc<ViewModel>);

#[uniffi::export]
fn do_stuff123() -> () {}

struct GenericObject<I>(I);

#[derive(uniffi::Object)]
struct GenericObjectString(GenericObject<String>);

#[derive(uniffi::Object)]
struct GenericObjectU8(GenericObject<u8>);

#[uniffi::export]
impl GenericObjectString {
    fn get(&self) -> String {
        self.0 .0.clone()
    }
}

#[uniffi::export]
impl GenericObjectU8 {
    fn get(&self) -> u8 {
        self.0 .0
    }
}
