// Copyright 2021 TiKV Project Authors. Licensed under Apache-2.0.

//! Collector and the collected spans.

pub(crate) mod global_collector;

use crate::local::span_id::SpanId;

/// A span records been collected.
#[derive(Clone, Debug, Default)]
pub struct SpanRecord {
    pub id: u32,
    pub parent_id: u32,
    pub begin_unix_time_ns: u64,
    pub duration_ns: u64,
    pub event: &'static str,
    pub properties: Vec<(&'static str, String)>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParentSpan {
    pub(crate) span_id: SpanId,
    pub(crate) collect_id: u32,
}

/// The collector for collecting all spans of a request.
///
/// A [`Collector`] is provided when starting a root [`Span`](crate::Span) by calling [`Span::root()`](crate::Span::root).
///
/// # Examples
///
/// ```
/// use minitrace::prelude::*;
/// use futures::executor::block_on;
///
/// let (root, collector) = Span::root("root");
/// drop(root);
///
/// let records: Vec<SpanRecord> = block_on(collector.collect());
/// ```
pub struct Collector {
    collect_id: u32,
}

impl Collector {
    pub(crate) fn start_collect(args: CollectArgs) -> (Self, u32) {
        let collect_id = global_collector::start_collect(args);

        (Collector { collect_id }, collect_id)
    }

    /// Stop the trace and collect all span been recorded.
    ///
    /// To extremely eliminate the overhead of tracing, the heavy computation and thread synchronization
    /// work are moved to a background thread, and thus, we have to wait for the background thread to send
    /// the result back. It usually takes 10 milliseconds because the background thread wakes up and processes
    /// messages every 10 milliseconds.
    pub async fn collect(self) -> Vec<SpanRecord> {
        let (tx, rx) = futures::channel::oneshot::channel();
        global_collector::commit_collect(self.collect_id, tx);

        // Because the collect is committed, don't drop the collect.
        std::mem::forget(self);

        rx.await.unwrap_or_else(|_| Vec::new())
    }
}

impl Drop for Collector {
    fn drop(&mut self) {
        global_collector::drop_collect(self.collect_id);
    }
}

/// Arguments for the collector.
///
/// Customize collection behavior through [`Span::root_with_args()`](crate::Span::root_with_args).
#[must_use]
#[derive(Default, Debug)]
pub struct CollectArgs {
    pub(crate) max_span_count: Option<usize>,
}

impl CollectArgs {
    /// A soft limit for the span collection in background, usually used to avoid out-of-memory.
    ///
    /// # Notice
    ///
    /// Root span will always be collected. The eventually collected spans may exceed the limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use minitrace::prelude::*;
    ///
    /// let args = CollectArgs::default().max_span_count(Some(100));
    /// let (root, collector) = Span::root_with_args("root", args);
    /// ```
    pub fn max_span_count(self, max_span_count: Option<usize>) -> Self {
        Self { max_span_count }
    }
}
