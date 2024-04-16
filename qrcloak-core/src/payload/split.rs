use crate::{
    format::{CompletePayload, PartialPayload, PartialPayloadHead, PartialPayloadTail},
    payload::utils::{IndexIter, Splits},
};

use super::one_or_more::OneOrMore;

#[derive(Default, Clone)]
pub struct PayloadSplitter {
    splits: u32,
}

impl PayloadSplitter {
    pub fn with_splits(mut self, splits: u32) -> Self {
        self.splits = splits.max(1);
        self
    }

    pub fn split(&self, payload: CompletePayload) -> OneOrMore<'static, PartialPayload> {
        let CompletePayload {
            data,
            encryption,
            compression,
        } = payload;

        let mut splits = Splits::new(data, self.splits as usize);
        let mut index = IndexIter::new(self.splits);

        let (head_bytes, head_index) = (&mut splits)
            .zip(&mut index)
            .next()
            .expect("splits should be at least 1");

        assert!(head_index.is_head());

        let head = PartialPayload::Head(PartialPayloadHead {
            complete: CompletePayload {
                data: head_bytes,
                encryption,
                compression,
            },
            index: head_index,
        });

        let tail = splits
            .zip(index)
            .map(|(split, index)| PartialPayload::Tail(PartialPayloadTail { data: split, index }));

        [head]
            .into_iter()
            .chain(tail)
            .collect::<Vec<_>>()
            .try_into()
            .expect("at least one split")
    }
}
