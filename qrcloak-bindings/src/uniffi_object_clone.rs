#[macro_export]
macro_rules! uniffi_object_clone {
    ($name:ident) => {
        const _: () = {
            use std::sync::Arc;
            use uniffi::{FfiConverter, FfiConverterArc, Lift, Lower, MetadataBuffer, Result};

            unsafe impl<UT> FfiConverter<UT> for $name {
                const TYPE_ID_META: MetadataBuffer = <Self as FfiConverterArc<UT>>::TYPE_ID_META;

                fn lower(obj: Self) -> Self::FfiType {
                    <Self as FfiConverterArc<UT>>::lower(Arc::new(obj))
                }
                fn write(obj: Self, buf: &mut Vec<u8>) {
                    <Self as FfiConverterArc<UT>>::write(Arc::new(obj), buf)
                }

                fn try_lift(v: Self::FfiType) -> Result<Self> {
                    <Self as FfiConverterArc<UT>>::try_lift(v).map(|v| (*v).clone())
                }

                fn try_read(buf: &mut &[u8]) -> Result<Self> {
                    <Self as FfiConverterArc<UT>>::try_read(buf).map(|v| (*v).clone())
                }

                type FfiType = <Self as FfiConverterArc<UT>>::FfiType;
            }

            unsafe impl<UT> Lower<UT> for $name {
                const TYPE_ID_META: MetadataBuffer = <Self as FfiConverter<UT>>::TYPE_ID_META;

                type FfiType = <Self as FfiConverter<UT>>::FfiType;

                fn lower(obj: Self) -> Self::FfiType {
                    <Self as FfiConverter<UT>>::lower(obj)
                }

                fn write(obj: Self, buf: &mut Vec<u8>) {
                    <Self as FfiConverter<UT>>::write(obj, buf)
                }
            }

            unsafe impl<UT> Lift<UT> for $name {
                const TYPE_ID_META: MetadataBuffer = <Self as FfiConverter<UT>>::TYPE_ID_META;

                type FfiType = <Self as FfiConverter<UT>>::FfiType;

                fn try_lift(v: Self::FfiType) -> Result<Self> {
                    <Self as FfiConverter<UT>>::try_lift(v)
                }

                fn try_read(buf: &mut &[u8]) -> Result<Self> {
                    <Self as FfiConverter<UT>>::try_read(buf)
                }
            }
        };
    };
}
