use std::{collections::HashMap, sync::Mutex};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum MetricsType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Clone)]
pub struct MetricsDescription {
    pub unit: Option<metrics::Unit>,
    pub description: metrics::SharedString,
}

#[derive(Default)]
pub struct Metrics {
    inner: Mutex<HashMap<(MetricsType, metrics::KeyName), MetricsDescription>>,
}

impl Metrics {
    pub fn get(&self, r#type: MetricsType, key: &metrics::KeyName) -> Option<MetricsDescription> {
        self.inner
            .lock()
            .unwrap()
            .get(&(r#type, key.clone()))
            .cloned()
    }

    pub fn set(&self, r#type: MetricsType, key: metrics::KeyName, value: MetricsDescription) {
        self.inner.lock().unwrap().insert((r#type, key), value);
    }
}
