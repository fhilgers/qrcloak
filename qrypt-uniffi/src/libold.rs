use std::{
    any::Any,
    ffi::c_void,
    fmt::Debug,
    marker::{self, PhantomData},
    num::Wrapping,
    str::from_utf8,
    sync::{Arc, Mutex, RwLock},
};

use const_format::formatcp;
use dyn_clone::DynClone;
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

struct Stream<T: Clone>(futures::lock::Mutex<SignalStream<MutableSignalCloned<T>>>);

impl<T: Clone> From<&Mutable<T>> for Stream<T> {
    fn from(value: &Mutable<T>) -> Self {
        Self(futures::lock::Mutex::new(value.signal_cloned().to_stream()))
    }
}

impl<T: Clone> Stream<T> {
    async fn poll(&self) -> T {
        self.0.lock().await.next().await.unwrap()
    }
}

macro_rules! generate {
    ($id:ident, $type:ty) => {
        paste::item! {
            #[derive(uniffi::Object)]
            struct [<$id Stream>](Stream<$type>);

            #[uniffi::export]
            impl [<$id Stream>] {
                async fn poll(&self) -> $type {
                    self.0.poll().await
                }
            }

            impl From<Stream<$type>> for [<$id Stream>] {
                fn from(value: Stream<$type>) -> Self {
                    Self(value)
                }
            }
        }
    };
}

macro_rules! stream {
    ($id:ident) => {
        paste::item! {
            [<$id Stream>]
        }
    };
}

generate!(String, String);
generate!(U8, u8);

/*
#[derive(Clone)]
struct Everything(Box<dyn FfiConverter<UT>>);

unsafe impl<T: FfiConverter<UT>, UT> FfiConverter<UT> for Everything<T> {
    type FfiType = T::FfiType;

    fn lower(obj: Self) -> Self::FfiType {
        T::lower(obj.0)
    }

    fn try_lift(v: Self::FfiType) -> uniffi::Result<Self> {
        T::try_lift(v).map(|v| Self(v))
    }

    fn write(obj: Self, buf: &mut Vec<u8>) {
        T::write(obj.0, buf)
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::Result<Self> {
        T::try_read(buf).map(|v| Self(v))
    }

    const TYPE_ID_META: uniffi::MetadataBuffer = T::TYPE_ID_META;
}

unsafe impl<T: Lower<UT>, UT> Lower<UT> for Everything<T> {
    type FfiType = T::FfiType;

    fn lower(obj: Self) -> Self::FfiType {
        T::lower(obj.0)
    }

    fn write(obj: Self, buf: &mut Vec<u8>) {
        T::write(obj.0, buf)
    }

    const TYPE_ID_META: uniffi::MetadataBuffer = T::TYPE_ID_META;
}

unsafe impl<T: Lift<UT>, UT> Lift<UT> for Everything<T> {
    type FfiType = T::FfiType;



    const TYPE_ID_META: uniffi::MetadataBuffer = T::TYPE_ID_META;

    fn try_lift(v: Self::FfiType) -> uniffi::Result<Self> {
        T::try_lift(v).map(|v| Self(v))
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::Result<Self> {
        T::try_read(buf).map(|v| Self(v))
    }
}
*/

/*
#[uniffi::export]
impl<T: FfiConverterArc<UT>, UT> Everything<T, UT> {
    #[uniffi::constructor]
    fn new(value: Arc<dyn Any + Send + Sync>) -> Self {
        Self(value)
    }
}
*/

/*
#[derive(Clone, uniffi::Object)]
struct Everything(Arc<dyn Any+ Send + Sync + 'static>);
*/

type Callback<T> = Box<dyn Fn(T) -> () + Send + Sync + 'static>;

struct MutableState<T: Clone, C: Fn(T) -> ()> {
    value: T,
    callback: C,
}

impl<T: Clone, C: Fn(T) -> ()> MutableState<T, C> {
    fn set_value(&mut self, value: T) {
        (self.callback)(value.clone());
        self.value = value;
    }

    fn get_value(&self) -> T {
        self.value.clone()
    }
}

struct MutableWrapper<T: Clone>(
    Mutable<T>,
    futures::lock::Mutex<SignalStream<MutableSignalCloned<T>>>,
);

impl<T: Clone> MutableWrapper<T> {
    async fn poll(&self) -> Option<T> {
        self.1.lock().await.next().await
    }
    fn set(&self, value: T) {
        self.0.set(value)
    }
}

impl<T: Clone> From<Mutable<T>> for MutableWrapper<T> {
    fn from(value: Mutable<T>) -> Self {
        let mutable_cloned = value.clone();
        let mutable_stream = futures::lock::Mutex::new(value.signal_cloned().to_stream());

        Self(mutable_cloned, mutable_stream)
    }
}

pub struct ImmutableWrapper<T: DynClone>(
    futures::lock::Mutex<SignalStream<MutableSignalCloned<T>>>,
);

impl<T: Clone> ImmutableWrapper<T> {
    async fn poll(&self) -> Option<T> {
        //panic!("{:?}", self.0.lock().await.next().await);
        self.0.lock().await.next().await
    }
}

impl<T: Clone> From<Mutable<T>> for ImmutableWrapper<T> {
    fn from(value: Mutable<T>) -> Self {
        let mutable_stream = futures::lock::Mutex::new(value.signal_cloned().to_stream());

        Self(mutable_stream)
    }
}

macro_rules! immutable_impl {
    ($name:ident, $type:ty) => {
        paste::paste! {
            #[derive(uniffi::Object)]
            pub struct [<ImmutableWrapper $name>](ImmutableWrapper<$type>);

            #[uniffi::export]
            impl [<ImmutableWrapper $name>] {
                async fn poll(&self) -> Option<$type> {
                    self.0.poll().await
                }
            }

            impl From<ImmutableWrapper<$type>> for [<ImmutableWrapper $name>] {
                fn from(value: ImmutableWrapper<$type>) -> Self {
                    Self(value)
                }
            }
        }
    };
}

macro_rules! mutable_impl {
    ($name:ident, $type:ty) => {
        paste::paste! {
            #[derive(uniffi::Object)]
            struct [<MutableWrapper $name>](MutableWrapper<$type>);

            #[uniffi::export]
            impl [<MutableWrapper $name>] {
                async fn poll(&self) -> Option<$type> {
                    self.0.poll().await
                }
                fn set(&self, value: $type) {
                    self.0.set(value)
                }
            }

            impl From<MutableWrapper<$type>> for [<MutableWrapper $name>] {
                fn from(value: MutableWrapper<$type>) -> Self {
                    Self(value)
                }
            }
        }
    };
}

macro_rules! wrapper {
    ($type:ty) => {
        paste::paste! {
            [<MutableWrapper $type>]
        }
    };
}

macro_rules! wrapper_immutable {
    ($type:ty) => {
        paste::paste! {
            [<ImmutableWrapper $type>]
        }
    };
}

macro_rules! wrap {
    ($e:expr) => {
        MutableWrapper::from($e).into()
    };
}

macro_rules! wrap_immutable {
    ($e:expr) => {
        ImmutableWrapper::from($e).into()
    };
}

#[derive(uniffi::Object)]
struct ViewModel {
    counter: Mutable<i32>,
    text: Mutable<String>,
}

mutable_impl!(I32, i32);
immutable_impl!(I32, i32);

#[uniffi::export]
impl ViewModel {
    #[uniffi::constructor]
    fn new(initial_value: i32) -> Arc<Self> {
        let counter = Mutable::new(initial_value);
        let text = Mutable::new("hello".to_string());

        Arc::new(Self { counter, text })
    }

    fn increment(&self) {
        self.counter.replace_with(|v| *v + 1);
    }

    fn decrement(&self) {
        self.counter.replace_with(|v| *v - 1);
    }

    fn counter(&self) -> ImmutableI32 {
        ImmutableI32(Arc::new(wrap_immutable!(self.counter.clone())))
    }

    fn text(&self) -> Immutable<String> {
        self.text.clone().into()
    }
}

impl Nameable for String {
    const name: &'static str = "String";
}

pub struct ImmutableI32(Arc<ImmutableWrapperI32>);

custom_newtype!(ImmutableI32, Arc<ImmutableWrapperI32>);

struct Uuid(String);
custom_newtype!(Uuid, String);

impl<T: Clone + Nameable> Nameable for ImmutableWrapper<T> {
    const name: &'static str = buf_and_len_to_str(&concat_buf("ImmutableWrapper", T::name));
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn uniffi_qrypt_uniffi_fn_clone_immutablewrapper(
    ptr: *const ::std::ffi::c_void,
    call_status: &mut ::uniffi::RustCallStatus,
) -> *const ::std::ffi::c_void {
    uniffi::rust_call(call_status, || {
        unsafe { ::std::sync::Arc::increment_strong_count(ptr) };
        Ok(ptr)
    })
}
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn uniffi_qrypt_uniffi_fn_free_immutablewrapper(
    ptr: *const ::std::ffi::c_void,
    call_status: &mut ::uniffi::RustCallStatus,
) {
    uniffi::rust_call(call_status, || {
        if !!ptr.is_null() {
            panic!("assertion failed: !ptr.is_null()")
        }
        let ptr = ptr.cast::<ImmutableWrapper<String>>();
        unsafe {
            ::std::sync::Arc::decrement_strong_count(ptr);
        }
        Ok(())
    });
}
const _: fn() = || {
    fn assert_impl_all<T: ?Sized + ::core::marker::Sync + ::core::marker::Send>() {}
    assert_impl_all::<ImmutableWrapper<String>>();
};

#[doc(hidden)]
#[automatically_derived]
/// Support for passing reference-counted shared objects via the FFI.
///
/// To avoid dealing with complex lifetime semantics over the FFI, any data passed
/// by reference must be encapsulated in an `Arc`, and must be safe to share
/// across threads.
unsafe impl<T: Clone + Send + Sync + 'static, UT> ::uniffi::FfiConverterArc<UT>
    for ImmutableWrapper<T>
{
    type FfiType = *const ::std::os::raw::c_void;
    /// When lowering, we have an owned `Arc` and we transfer that ownership
    /// to the foreign-language code, "leaking" it out of Rust's ownership system
    /// as a raw pointer. This works safely because we have unique ownership of `self`.
    /// The foreign-language code is responsible for freeing this by calling the
    /// `ffi_object_free` FFI function provided by the corresponding UniFFI type.
    ///
    /// Safety: when freeing the resulting pointer, the foreign-language code must
    /// call the destructor function specific to the type `T`. Calling the destructor
    /// function for other types may lead to undefined behaviour.
    fn lower(obj: ::std::sync::Arc<Self>) -> Self::FfiType {
        ::std::sync::Arc::into_raw(obj) as Self::FfiType
    }
    /// When lifting, we receive an owned `Arc` that the foreign language code cloned.
    fn try_lift(v: Self::FfiType) -> ::uniffi::Result<::std::sync::Arc<Self>> {
        let v = v as *const ImmutableWrapper<T>;
        Ok(unsafe { ::std::sync::Arc::<Self>::from_raw(v) })
    }
    /// When writing as a field of a complex structure, make a clone and transfer ownership
    /// of it to the foreign-language code by writing its pointer into the buffer.
    /// The foreign-language code is responsible for freeing this by calling the
    /// `ffi_object_free` FFI function provided by the corresponding UniFFI type.
    ///
    /// Safety: when freeing the resulting pointer, the foreign-language code must
    /// call the destructor function specific to the type `T`. Calling the destructor
    /// function for other types may lead to undefined behaviour.
    fn write(obj: ::std::sync::Arc<Self>, buf: &mut Vec<u8>) {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = ::std::mem::size_of::<*const ::std::ffi::c_void>() <= 8;
            ASSERT
        } as usize] = [];
        ::uniffi::deps::bytes::BufMut::put_u64(
            buf,
            <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::lower(obj) as u64,
        );
    }
    /// When reading as a field of a complex structure, we receive a "borrow" of the `Arc`
    /// that is owned by the foreign-language code, and make a clone for our own use.
    ///
    /// Safety: the buffer must contain a pointer previously obtained by calling
    /// the `lower()` or `write()` method of this impl.
    fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<::std::sync::Arc<Self>> {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = ::std::mem::size_of::<*const ::std::ffi::c_void>() <= 8;
            ASSERT
        } as usize] = [];
        ::uniffi::check_remaining(buf, 8)?;
        <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::try_lift(
            ::uniffi::deps::bytes::Buf::get_u64(buf) as Self::FfiType,
        )
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer =
        ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_INTERFACE)
            .concat_str("qrypt_uniffi")
            .concat_str("ImmutableWrapper");
}

unsafe impl<T: Clone + Send + Sync + 'static + Nameable, UT> ::uniffi::LowerReturn<UT>
    for ImmutableWrapper<T>
{
    type ReturnType = <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::FfiType;
    fn lower_return(obj: Self) -> ::std::result::Result<Self::ReturnType, ::uniffi::RustBuffer> {
        Ok(
            <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::lower(::std::sync::Arc::new(
                obj,
            )),
        )
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer =
        <Self as ::uniffi::FfiConverterArc<crate::UniFfiTag>>::TYPE_ID_META;
}
unsafe impl<T: Clone + Send + Sync + 'static + Nameable, UT> ::uniffi::LiftRef<UT>
    for ImmutableWrapper<T>
{
    type LiftType = ::std::sync::Arc<Self>;
}

const UNIFFI_META_CONST_QRYPT_UNIFFI_INTERFACE_IMMUTABLEWRAPPER: ::uniffi::MetadataBuffer =
    ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::INTERFACE)
        .concat_str("qrypt_uniffi")
        .concat_str("ImmutableWrapper")
        .concat_long_str("");

#[no_mangle]
#[doc(hidden)]
pub static UNIFFI_META_QRYPT_UNIFFI_INTERFACE_IMMUTABLEWRAPPER: [u8;
    UNIFFI_META_CONST_QRYPT_UNIFFI_INTERFACE_IMMUTABLEWRAPPER.size] =
    UNIFFI_META_CONST_QRYPT_UNIFFI_INTERFACE_IMMUTABLEWRAPPER.into_array();

const UNIFFI_META_CONST_QRYPT_UNIFFI_INTERFACE_ANY: ::uniffi::MetadataBuffer =
    ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::INTERFACE)
        .concat_str("qrypt_uniffi")
        .concat_str("AnyWrapper")
        .concat_long_str("");

#[no_mangle]
#[doc(hidden)]
pub static UNIFFI_META_QRYPT_UNIFFI_INTERFACE_ANY: [u8;
    UNIFFI_META_CONST_QRYPT_UNIFFI_INTERFACE_ANY.size] =
    UNIFFI_META_CONST_QRYPT_UNIFFI_INTERFACE_ANY.into_array();

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn uniffi_qrypt_uniffi_fn_method_immutablewrapper_poll(
    uniffi_self_lowered: *const c_void,
) -> ::uniffi::RustFutureHandle {
    {
        let lvl = ::log::Level::Debug;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api::log(
                format_args!("poll"),
                lvl,
                &("qrypt_uniffi", "qrypt_uniffi", "qrypt-uniffi/src/lib.rs"),
                502u32,
                (),
            );
        }
    };
    let uniffi_lift_args =
        move || {
            Ok((
                match <::std::sync::Arc<ImmutableWrapper<String>> as ::uniffi::Lift<
                    crate::UniFfiTag,
                >>::try_lift(uniffi_self_lowered)
                {
                    Ok(v) => v,
                    Err(e) => return Err(("self", e)),
                },
            ))
        };
    match uniffi_lift_args() {
        Ok(uniffi_args) => {
            ::uniffi::rust_future_new(async move { uniffi_args.0.poll().await }, crate::UniFfiTag)
        }
        Err((arg_name, anyhow_error)) => ::uniffi::rust_future_new(
            async move {
                <Option<String> as ::uniffi::LowerReturn<crate::UniFfiTag>>::handle_failed_lift(
                    arg_name,
                    anyhow_error,
                )
            },
            crate::UniFfiTag,
        ),
    }
}
const UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_IMMUTABLEWRAPPERSTRING_POLL: ::uniffi::MetadataBuffer =
    ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::METHOD)
        .concat_str("qrypt_uniffi")
        .concat_str("ImmutableWrapper")
        .concat_str("poll")
        .concat_bool(true)
        .concat_value(0u8)
        .concat(<Option<String> as ::uniffi::LowerReturn<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_long_str("");
#[no_mangle]
#[doc(hidden)]
pub static UNIFFI_META_QRYPT_UNIFFI_METHOD_IMMUTABLEWRAPPERSTRING_POLL: [u8;
    UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_IMMUTABLEWRAPPERSTRING_POLL.size] =
    UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_IMMUTABLEWRAPPERSTRING_POLL.into_array();
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn uniffi_qrypt_uniffi_checksum_method_immutablewrapperstring_poll() -> u16 {
    UNIFFI_META_CONST_QRYPT_UNIFFI_METHOD_IMMUTABLEWRAPPERSTRING_POLL.checksum()
}

trait ObjectSafeLower {
    fn lower(self: Box<Self>) -> Box<dyn Any>;
    fn write(self: Box<Self>, buf: &mut Vec<u8>);
}

impl<T: Lower<crate::UniFfiTag>> ObjectSafeLower for T {
    fn lower(self: Box<Self>) -> Box<dyn Any> {
        Box::new(<Self as Lower<crate::UniFfiTag>>::lower(*self))
    }

    fn write(self: Box<Self>, buf: &mut Vec<u8>) {
        <Self as Lower<crate::UniFfiTag>>::write(*self, buf)
    }
}

struct Immutable<T: Clone>(Arc<ImmutableWrapper<T>>);

impl<T: Clone> From<Mutable<T>> for Immutable<T> {
    fn from(value: Mutable<T>) -> Self {
        Self(Arc::new(ImmutableWrapper::from(value)))
    }
}

//custom_type!(Immutable2<String>, Arc<ImmutableWrapper<String>>);

macro_rules! custom_newtype2 {
    () => {};
}

trait Nameable {
    const name: &'static str;
}

const fn buf_and_len_to_str(buf_len: &'static ([u8; 60], usize)) -> &'static str {
    let buf = &buf_len.0;
    let len = buf_len.1;
    assert!(len < buf.len(), "buf is too long");
    // I didn't find a way to slice an array in const fn
    let buf = unsafe { core::slice::from_raw_parts(buf.as_ptr(), len) };
    match core::str::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => panic!(),
    }
}

const fn concat_buf(left: &'static str, right: &'static str) -> ([u8; 60], usize) {
    let mut buf = [0u8; 60];
    let mut i = 0;
    while i < left.len() {
        buf[i] = left.as_bytes()[i];
        i += 1;
    }
    while i - left.len() < right.len() {
        buf[i] = right.as_bytes()[i - left.len()];
        i += 1;
    }

    (buf, i)
}

impl<T: Nameable + std::clone::Clone> Nameable for Immutable<T> {
    const name: &'static str = buf_and_len_to_str(&concat_buf("Immutable", <T as Nameable>::name));
}

#[automatically_derived]
unsafe impl<T: Clone + Nameable> ::uniffi::FfiConverter<crate::UniFfiTag> for Immutable<T>
where
    ImmutableWrapper<T>: FfiConverterArc<UniFfiTag>,
{
    type FfiType = <Arc<ImmutableWrapper<T>> as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
    fn lower(obj: Immutable<T>) -> Self::FfiType {
        <Arc<ImmutableWrapper<T>> as ::uniffi::Lower<crate::UniFfiTag>>::lower(
            <Immutable<T> as crate::UniffiCustomTypeConverter>::from_custom(obj),
        )
    }
    fn try_lift(v: Self::FfiType) -> uniffi::Result<Immutable<T>> {
        <Immutable<T> as crate::UniffiCustomTypeConverter>::into_custom(
            <Arc<ImmutableWrapper<T>> as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(v)?,
        )
    }
    fn write(obj: Immutable<T>, buf: &mut Vec<u8>) {
        <Arc<ImmutableWrapper<T>> as ::uniffi::Lower<crate::UniFfiTag>>::write(
            <Immutable<T> as crate::UniffiCustomTypeConverter>::from_custom(obj),
            buf,
        );
    }
    fn try_read(buf: &mut &[u8]) -> uniffi::Result<Immutable<T>> {
        <Immutable<T> as crate::UniffiCustomTypeConverter>::into_custom(
            <Arc<ImmutableWrapper<T>> as ::uniffi::Lift<crate::UniFfiTag>>::try_read(buf)?,
        )
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer =
        ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_CUSTOM)
            .concat_str("qrypt_uniffi")
            .concat_str(Self::name)
            .concat(
                <Arc<ImmutableWrapper<String>> as ::uniffi::Lower<crate::UniFfiTag>>::TYPE_ID_META,
            );
}

unsafe impl<T: Clone + Nameable> ::uniffi::Lower<crate::UniFfiTag> for Immutable<T>
where
    ImmutableWrapper<T>: FfiConverterArc<UniFfiTag>,
{
    type FfiType = <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::FfiType;
    fn lower(obj: Self) -> Self::FfiType {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::lower(obj)
    }
    fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::write(obj, buf)
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer =
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META;
}
unsafe impl<T: Clone + Nameable> ::uniffi::Lift<crate::UniFfiTag> for Immutable<T>
where
    ImmutableWrapper<T>: FfiConverterArc<UniFfiTag>,
{
    type FfiType = <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::FfiType;
    fn try_lift(v: Self::FfiType) -> ::uniffi::deps::anyhow::Result<Self> {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_lift(v)
    }
    fn try_read(buf: &mut &[u8]) -> ::uniffi::deps::anyhow::Result<Self> {
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_read(buf)
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer =
        <Self as ::uniffi::FfiConverter<crate::UniFfiTag>>::TYPE_ID_META;
}
unsafe impl<T: Clone + Nameable> ::uniffi::LowerReturn<crate::UniFfiTag> for Immutable<T>
where
    ImmutableWrapper<T>: FfiConverterArc<UniFfiTag>,
{
    type ReturnType = <Self as ::uniffi::Lower<crate::UniFfiTag>>::FfiType;
    fn lower_return(
        obj: Self,
    ) -> ::uniffi::deps::anyhow::Result<Self::ReturnType, ::uniffi::RustBuffer> {
        Ok(<Self as ::uniffi::Lower<crate::UniFfiTag>>::lower(obj))
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer =
        <Self as ::uniffi::Lower<crate::UniFfiTag>>::TYPE_ID_META;
}
unsafe impl<T: Clone + Nameable> ::uniffi::LiftReturn<crate::UniFfiTag> for Immutable<T>
where
    ImmutableWrapper<T>: FfiConverterArc<UniFfiTag>,
{
    fn lift_callback_return(buf: ::uniffi::RustBuffer) -> Self {
        <Self as ::uniffi::Lift<crate::UniFfiTag>>::try_lift_from_rust_buffer(buf)
            .expect("Error reading callback interface result")
    }
    const TYPE_ID_META: ::uniffi::MetadataBuffer =
        <Self as ::uniffi::Lift<crate::UniFfiTag>>::TYPE_ID_META;
}
unsafe impl<T: Clone + Nameable> ::uniffi::LiftRef<crate::UniFfiTag> for Immutable<T>
where
    ImmutableWrapper<T>: FfiConverterArc<UniFfiTag>,
{
    type LiftType = Self;
}
impl ::uniffi::ConvertError<crate::UniFfiTag> for Immutable<String> {
    fn try_convert_unexpected_callback_error(
        e: ::uniffi::UnexpectedUniFFICallbackError,
    ) -> ::uniffi::deps::anyhow::Result<Self> {
        {
            pub trait GetConverterGeneric {
                fn get_converter(&self) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric;
            }
            impl<T> GetConverterGeneric for &T {
                fn get_converter(&self) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric {
                    ::uniffi::UnexpectedUniFFICallbackErrorConverterGeneric
                }
            }
            pub trait GetConverterSpecialized {
                fn get_converter(
                    &self,
                ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized;
            }
            impl<T: Into<Immutable<String>>> GetConverterSpecialized for T {
                fn get_converter(
                    &self,
                ) -> ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized {
                    ::uniffi::UnexpectedUniFFICallbackErrorConverterSpecialized
                }
            }
            (&e).get_converter()
                .try_convert_unexpected_callback_error(e)
        }
    }
}

impl<T: Clone> crate::UniffiCustomTypeConverter for Immutable<T> {
    type Builtin = Arc<ImmutableWrapper<T>>;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(val))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0
    }
}
