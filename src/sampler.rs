use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

/// Rolling averages over the standard 1 / 5 / 15-minute windows. Each field is
/// `f64::NAN` when the sampler has not yet collected any samples.
#[derive(Debug, Clone, Copy)]
pub struct Averages {
    pub one_min: f64,
    pub five_min: f64,
    pub fifteen_min: f64,
}

impl Averages {
    pub fn is_nan(self) -> bool {
        self.one_min.is_nan()
    }
}

/// Fixed-cadence ring buffer of samples. With a 1 Hz sampler this holds the
/// last 15 minutes of readings; older samples roll off the front.
struct Window {
    samples: VecDeque<f64>,
}

impl Window {
    const MAX_SAMPLES: usize = 15 * 60;

    fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(Self::MAX_SAMPLES),
        }
    }

    fn push(&mut self, value: f64) {
        if self.samples.len() >= Self::MAX_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(value);
    }

    /// Mean of the most recent `n` samples (or all available if fewer have
    /// been collected). Returns `NAN` for an empty window.
    fn mean_last(&self, n: usize) -> f64 {
        let take = n.min(self.samples.len());
        if take == 0 {
            return f64::NAN;
        }
        let skip = self.samples.len() - take;
        let sum: f64 = self.samples.iter().skip(skip).sum();
        sum / take as f64
    }

    fn averages(&self) -> Averages {
        Averages {
            one_min: self.mean_last(60),
            five_min: self.mean_last(60 * 5),
            fifteen_min: self.mean_last(60 * 15),
        }
    }
}

struct Sampler {
    tps: Window,
    mspt: Window,
}

static SAMPLER: OnceLock<Mutex<Sampler>> = OnceLock::new();

fn sampler() -> &'static Mutex<Sampler> {
    SAMPLER.get_or_init(|| {
        Mutex::new(Sampler {
            tps: Window::new(),
            mspt: Window::new(),
        })
    })
}

/// Append one TPS / MSPT reading to the rolling windows. Called from the
/// background sampler task scheduled in [`Plugin::on_load`].
///
/// [`Plugin::on_load`]: pumpkin_plugin_api::Plugin::on_load
pub fn record(tps: f64, mspt: f64) {
    let mut s = sampler().lock().unwrap();
    s.tps.push(tps);
    s.mspt.push(mspt);
}

pub fn tps_averages() -> Averages {
    sampler().lock().unwrap().tps.averages()
}

pub fn mspt_averages() -> Averages {
    sampler().lock().unwrap().mspt.averages()
}
