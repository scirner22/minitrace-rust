// Copyright 2021 TiKV Project Authors. Licensed under Apache-2.0.

use minstant::Instant;

use crate::local::span_id::SpanId;

#[derive(Clone, Debug)]
pub(crate) struct RawSpan {
    pub id: SpanId,
    pub parent_id: SpanId,
    pub begin_instant: Instant,
    pub event: &'static str,
    pub properties: Vec<(&'static str, String)>,

    // Will write this field at post processing
    pub end_instant: Instant,
}

impl RawSpan {
    #[inline]
    pub(crate) fn begin_with(
        id: SpanId,
        parent_id: SpanId,
        begin_instant: Instant,
        event: &'static str,
    ) -> Self {
        RawSpan {
            id,
            parent_id,
            begin_instant,
            event,
            properties: vec![],
            end_instant: begin_instant,
        }
    }

    #[inline]
    pub(crate) fn end_with(&mut self, end_instant: Instant) {
        self.end_instant = end_instant;
    }
}
