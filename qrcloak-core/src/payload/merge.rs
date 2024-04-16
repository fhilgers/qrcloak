use std::collections::HashMap;

use bytes::BytesMut;

use crate::format::{CompletePayload, PartialPayload};

#[derive(Debug, Clone, Default)]
pub struct UnmergedPayloads {
    partials: HashMap<(u32, u32), Vec<Option<PartialPayload>>>,
    misconfigured: Vec<PartialPayload>,
}

#[derive(Debug, Clone, Default)]
pub struct MergeResult(pub Vec<CompletePayload>, pub UnmergedPayloads);

#[derive(Debug, Clone, Default)]
pub struct PayloadMerger {
    unmerged: UnmergedPayloads,
}

impl PayloadMerger {
    pub fn with_unmerged(mut self, unmerged: UnmergedPayloads) -> Self {
        self.unmerged = unmerged;
        self
    }

    fn collect_partials(&mut self, payloads: Vec<PartialPayload>) {
        for payload in payloads {
            if payload.is_misconfigured() {
                self.unmerged.misconfigured.push(payload);
                continue;
            }

            let index = payload.index();
            let entry = self
                .unmerged
                .partials
                .entry((index.id, index.size))
                .or_insert(vec![None; index.size as usize]);

            entry[index.index as usize] = Some(payload);
        }
    }

    fn collect_merged(&mut self) -> Vec<CompletePayload> {
        let mut merged = Vec::new();

        self.unmerged.partials.retain(|_, val| {
            let head = if let Some(PartialPayload::Head(head)) = &mut val[0] {
                head.clone()
            } else {
                return true;
            };

            let mut capacity = head.complete.data.len();
            let mut tail_data = Vec::with_capacity(head.index.size as usize - 1);

            for maybe_tail in val.into_iter().skip(1) {
                if let Some(PartialPayload::Tail(tail)) = maybe_tail {
                    capacity += tail.data.len();
                    tail_data.push(tail.data.clone());
                } else {
                    return true;
                }
            }

            let mut complete = BytesMut::with_capacity(capacity);
            complete.extend_from_slice(&head.complete.data);
            complete.extend(tail_data.into_iter());

            merged.push(CompletePayload {
                data: complete.freeze(),
                encryption: head.complete.encryption,
                compression: head.complete.compression,
            });

            false
        });

        merged
    }

    pub fn merge(mut self, payloads: Vec<PartialPayload>) -> MergeResult {
        self.collect_partials(payloads);

        MergeResult(self.collect_merged(), self.unmerged)
    }
}
