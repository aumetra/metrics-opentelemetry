use metrics::atomics::AtomicU64;
use opentelemetry_api::metrics::{Counter, Histogram, UpDownCounter};
use std::sync::atomic::Ordering;

pub struct CounterFn {
    pub inner: Counter<u64>,
    pub labels: Vec<opentelemetry_api::KeyValue>,
    pub last_value: AtomicU64,
}

impl metrics::CounterFn for CounterFn {
    fn absolute(&self, value: u64) {
        let diff = value - self.last_value.swap(value, Ordering::AcqRel);
        self.increment(diff);
    }

    fn increment(&self, value: u64) {
        self.last_value.increment(value);
        self.inner.add(value, &self.labels);
    }
}

pub struct GaugeFn {
    pub inner: UpDownCounter<f64>,
    pub labels: Vec<opentelemetry_api::KeyValue>,
    pub last_value: AtomicU64,
}

impl metrics::GaugeFn for GaugeFn {
    fn decrement(&self, value: f64) {
        self.last_value.decrement(value);
        self.inner.add(value * -1.0, &self.labels);
    }

    fn increment(&self, value: f64) {
        self.last_value.increment(value);
        self.inner.add(value, &self.labels);
    }

    fn set(&self, new_value: f64) {
        let old_value = self.last_value.swap(new_value.to_bits(), Ordering::AcqRel);
        let old_value = f64::from_bits(old_value);
        self.inner.add(old_value - new_value, &self.labels);
    }
}

pub struct HistogramFn {
    pub inner: Histogram<f64>,
    pub labels: Vec<opentelemetry_api::KeyValue>,
}

impl metrics::HistogramFn for HistogramFn {
    fn record(&self, value: f64) {
        self.inner.record(value, &self.labels);
    }
}
