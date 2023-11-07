#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use self::{
    metrics_fn::{CounterFn, GaugeFn, HistogramFn},
    storage::{MetricsDescription, MetricsType},
};
use metrics::atomics::AtomicU64;
use opentelemetry::metrics::{Meter, Unit};
use std::{slice, sync::Arc};

mod metrics_fn;
mod storage;

fn labels_to_keyvalue(labels: slice::Iter<'_, metrics::Label>) -> Vec<opentelemetry::KeyValue> {
    labels
        .map(|label| {
            let key = opentelemetry::Key::new(label.key().to_string());
            let value = opentelemetry::Value::from(label.value().to_string());
            opentelemetry::KeyValue { key, value }
        })
        .collect()
}

pub struct OpenTelemetryRecorder {
    meter: Meter,
    storage: storage::Metrics,
}

impl OpenTelemetryRecorder {
    #[must_use]
    pub fn new(meter: Meter) -> Self {
        Self {
            meter,
            storage: storage::Metrics::default(),
        }
    }
}

impl metrics::Recorder for OpenTelemetryRecorder {
    fn describe_counter(
        &self,
        key: metrics::KeyName,
        unit: Option<metrics::Unit>,
        description: metrics::SharedString,
    ) {
        self.storage.set(
            MetricsType::Counter,
            key,
            MetricsDescription { unit, description },
        );
    }

    fn describe_gauge(
        &self,
        key: metrics::KeyName,
        unit: Option<metrics::Unit>,
        description: metrics::SharedString,
    ) {
        self.storage.set(
            MetricsType::Gauge,
            key,
            MetricsDescription { unit, description },
        );
    }

    fn describe_histogram(
        &self,
        key: metrics::KeyName,
        unit: Option<metrics::Unit>,
        description: metrics::SharedString,
    ) {
        self.storage.set(
            MetricsType::Histogram,
            key,
            MetricsDescription { unit, description },
        );
    }

    fn register_counter(&self, key: &metrics::Key) -> metrics::Counter {
        let key_name = metrics::KeyName::from(key.name().to_string());
        let mut counter_builder = self.meter.u64_counter(key.name().to_string());

        if let Some(description) = self.storage.get(MetricsType::Counter, &key_name) {
            counter_builder = counter_builder.with_description(description.description.to_string());

            if let Some(unit) = description.unit {
                counter_builder =
                    counter_builder.with_unit(Unit::new(unit.as_canonical_label().to_string()));
            }
        }

        metrics::Counter::from_arc(Arc::new(CounterFn {
            inner: counter_builder.init(),
            labels: labels_to_keyvalue(key.labels()),
            last_value: AtomicU64::new(0),
        }))
    }

    fn register_gauge(&self, key: &metrics::Key) -> metrics::Gauge {
        let key_name = metrics::KeyName::from(key.name().to_string());
        let mut gauge_builder = self.meter.f64_up_down_counter(key.name().to_string());

        if let Some(description) = self.storage.get(MetricsType::Gauge, &key_name) {
            gauge_builder = gauge_builder.with_description(description.description.to_string());

            if let Some(unit) = description.unit {
                gauge_builder =
                    gauge_builder.with_unit(Unit::new(unit.as_canonical_label().to_string()));
            }
        }

        metrics::Gauge::from_arc(Arc::new(GaugeFn {
            inner: gauge_builder.init(),
            labels: labels_to_keyvalue(key.labels()),
            last_value: AtomicU64::new(0),
        }))
    }

    fn register_histogram(&self, key: &metrics::Key) -> metrics::Histogram {
        let key_name = metrics::KeyName::from(key.name().to_string());
        let mut histogram_builder = self.meter.f64_histogram(key.name().to_string());

        if let Some(description) = self.storage.get(MetricsType::Histogram, &key_name) {
            histogram_builder =
                histogram_builder.with_description(description.description.to_string());

            if let Some(unit) = description.unit {
                histogram_builder =
                    histogram_builder.with_unit(Unit::new(unit.as_canonical_label().to_string()));
            }
        }

        metrics::Histogram::from_arc(Arc::new(HistogramFn {
            inner: histogram_builder.init(),
            labels: labels_to_keyvalue(key.labels()),
        }))
    }
}
