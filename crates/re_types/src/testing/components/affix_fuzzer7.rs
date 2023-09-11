// DO NOT EDIT! This file was auto-generated by crates/re_types_builder/src/codegen/rust/api.rs
// Based on "crates/re_types/definitions/rerun/testing/components/fuzzy.fbs".

#![allow(trivial_numeric_casts)]
#![allow(unused_parens)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::iter_on_single_items)]
#![allow(clippy::map_flatten)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::needless_question_mark)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_cast)]

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AffixFuzzer7(pub Option<Vec<crate::testing::datatypes::AffixFuzzer1>>);

impl<I: Into<crate::testing::datatypes::AffixFuzzer1>, T: IntoIterator<Item = I>> From<Option<T>>
    for AffixFuzzer7
{
    fn from(v: Option<T>) -> Self {
        Self(v.map(|v| v.into_iter().map(|v| v.into()).collect()))
    }
}

impl<'a> From<AffixFuzzer7> for ::std::borrow::Cow<'a, AffixFuzzer7> {
    #[inline]
    fn from(value: AffixFuzzer7) -> Self {
        std::borrow::Cow::Owned(value)
    }
}

impl<'a> From<&'a AffixFuzzer7> for ::std::borrow::Cow<'a, AffixFuzzer7> {
    #[inline]
    fn from(value: &'a AffixFuzzer7) -> Self {
        std::borrow::Cow::Borrowed(value)
    }
}

impl crate::Loggable for AffixFuzzer7 {
    type Name = crate::ComponentName;

    #[inline]
    fn name() -> Self::Name {
        "rerun.testing.components.AffixFuzzer7".into()
    }

    #[allow(unused_imports, clippy::wildcard_imports)]
    #[inline]
    fn arrow_datatype() -> arrow2::datatypes::DataType {
        use ::arrow2::datatypes::*;
        DataType::List(Box::new(Field {
            name: "item".to_owned(),
            data_type: <crate::testing::datatypes::AffixFuzzer1>::arrow_datatype(),
            is_nullable: true,
            metadata: [].into(),
        }))
    }

    #[allow(unused_imports, clippy::wildcard_imports)]
    fn try_to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<::std::borrow::Cow<'a, Self>>>>,
    ) -> crate::SerializationResult<Box<dyn ::arrow2::array::Array>>
    where
        Self: Clone + 'a,
    {
        use crate::{Loggable as _, ResultExt as _};
        use ::arrow2::{array::*, datatypes::*};
        Ok({
            let (somes, data0): (Vec<_>, Vec<_>) = data
                .into_iter()
                .map(|datum| {
                    let datum: Option<::std::borrow::Cow<'a, Self>> = datum.map(Into::into);
                    let datum = datum
                        .map(|datum| {
                            let Self(data0) = datum.into_owned();
                            data0
                        })
                        .flatten();
                    (datum.is_some(), datum)
                })
                .unzip();
            let data0_bitmap: Option<::arrow2::bitmap::Bitmap> = {
                let any_nones = somes.iter().any(|some| !*some);
                any_nones.then(|| somes.into())
            };
            {
                use arrow2::{buffer::Buffer, offset::OffsetsBuffer};
                let data0_inner_data: Vec<_> = data0
                    .iter()
                    .flatten()
                    .flatten()
                    .cloned()
                    .map(Some)
                    .collect();
                let data0_inner_bitmap: Option<::arrow2::bitmap::Bitmap> = None;
                let offsets = ::arrow2::offset::Offsets::<i32>::try_from_lengths(
                    data0
                        .iter()
                        .map(|opt| opt.as_ref().map(|datum| datum.len()).unwrap_or_default()),
                )
                .unwrap()
                .into();
                ListArray::new(
                    Self::arrow_datatype(),
                    offsets,
                    {
                        _ = data0_inner_bitmap;
                        crate::testing::datatypes::AffixFuzzer1::try_to_arrow_opt(data0_inner_data)?
                    },
                    data0_bitmap,
                )
                .boxed()
            }
        })
    }

    #[allow(unused_imports, clippy::wildcard_imports)]
    fn try_from_arrow_opt(
        arrow_data: &dyn ::arrow2::array::Array,
    ) -> crate::DeserializationResult<Vec<Option<Self>>>
    where
        Self: Sized,
    {
        use crate::{Loggable as _, ResultExt as _};
        use ::arrow2::{array::*, buffer::*, datatypes::*};
        Ok({
            let arrow_data = arrow_data
                .as_any()
                .downcast_ref::<::arrow2::array::ListArray<i32>>()
                .ok_or_else(|| {
                    crate::DeserializationError::datatype_mismatch(
                        DataType::List(Box::new(Field {
                            name: "item".to_owned(),
                            data_type: <crate::testing::datatypes::AffixFuzzer1>::arrow_datatype(),
                            is_nullable: true,
                            metadata: [].into(),
                        })),
                        arrow_data.data_type().clone(),
                    )
                })
                .with_context("rerun.testing.components.AffixFuzzer7#many_optional")?;
            if arrow_data.is_empty() {
                Vec::new()
            } else {
                let arrow_data_inner = {
                    let arrow_data_inner = &**arrow_data.values();
                    crate::testing::datatypes::AffixFuzzer1::try_from_arrow_opt(arrow_data_inner)
                        .with_context("rerun.testing.components.AffixFuzzer7#many_optional")?
                        .into_iter()
                        .collect::<Vec<_>>()
                };
                let offsets = arrow_data.offsets();
                arrow2::bitmap::utils::ZipValidity::new_with_validity(
                    offsets.iter().zip(offsets.lengths()),
                    arrow_data.validity(),
                )
                .map(|elem| {
                    elem.map(|(start, len)| {
                        let start = *start as usize;
                        let end = start + len;
                        if end as usize > arrow_data_inner.len() {
                            return Err(crate::DeserializationError::offset_slice_oob(
                                (start, end),
                                arrow_data_inner.len(),
                            ));
                        }

                        #[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
                        let data =
                            unsafe { arrow_data_inner.get_unchecked(start as usize..end as usize) };
                        let data = data
                            .iter()
                            .cloned()
                            .map(Option::unwrap_or_default)
                            .collect();
                        Ok(data)
                    })
                    .transpose()
                })
                .collect::<crate::DeserializationResult<Vec<Option<_>>>>()?
            }
            .into_iter()
        }
        .map(Ok)
        .map(|res| res.map(|v| Some(Self(v))))
        .collect::<crate::DeserializationResult<Vec<Option<_>>>>()
        .with_context("rerun.testing.components.AffixFuzzer7#many_optional")
        .with_context("rerun.testing.components.AffixFuzzer7")?)
    }
}
