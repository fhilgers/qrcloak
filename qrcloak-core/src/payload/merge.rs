use std::collections::HashMap;

use bytes::BytesMut;

use crate::format::{CompletePayload, PartialPayload, Payload};

#[derive(Debug, Clone, Default)]
pub struct UnmergedPayloads {
    partials: HashMap<(u32, u32), Vec<Option<PartialPayload>>>,
    misconfigured: Vec<PartialPayload>,
}

impl UnmergedPayloads {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn partials(&self) -> &HashMap<(u32, u32), Vec<Option<PartialPayload>>> {
        &self.partials
    }

    pub fn misconfigured(&self) -> &Vec<PartialPayload> {
        &self.misconfigured
    }
}

#[derive(Debug, Clone, Default)]
pub struct MergeResult(pub Vec<CompletePayload>, pub UnmergedPayloads);

#[derive(Debug, Clone, Default)]
pub struct PayloadMerger {
    completes: Vec<CompletePayload>,
    unmerged: UnmergedPayloads,
}

impl PayloadMerger {
    pub fn with_unmerged(mut self, unmerged: UnmergedPayloads) -> Self {
        self.unmerged = unmerged;
        self
    }

    fn collect_partials<T, I>(&mut self, payloads: I)
    where
        T: Into<Payload>,
        I: IntoIterator<Item = T>,
    {
        for payload in payloads {
            let payload = match payload.into() {
                Payload::Complete(c) => {
                    self.completes.push(c);
                    continue;
                }
                Payload::Partial(p) => p,
            };

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

    fn collect_merged(&mut self) {
        self.unmerged.partials.retain(|_, val| {
            let head = if let Some(PartialPayload::Head(head)) = &mut val[0] {
                head.clone()
            } else {
                return true;
            };

            let mut capacity = head.data.len();
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
            complete.extend_from_slice(&head.data);
            complete.extend(tail_data.into_iter());

            self.completes.push(CompletePayload {
                data: complete.freeze(),
                encryption: head.encryption,
                compression: head.compression,
            });

            false
        });
    }

    pub fn merge(mut self, payloads: impl IntoIterator<Item = impl Into<Payload>>) -> MergeResult {
        self.collect_partials(payloads);
        self.collect_merged();

        MergeResult(self.completes, self.unmerged)
    }
}
