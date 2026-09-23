//! In-memory Ollama metadata cache (`#370`).
//!
//! Keys are `(endpoint, name, digest)`. Endpoint changes bump a generation and
//! drop entries so availability does not carry across servers. Late `/api/show`
//! results from a superseded generation are rejected. Concurrent enrich uses
//! at most 2–4 workers.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Result;

use crate::ollama::{
    effective_ollama_host, merge_tags_with_show, ollama_model_needs_show, OllamaJsonField,
    OllamaShowFields, OllamaTagsModel,
};

/// Default per-request timeout for cached enrich (`#370`).
pub const OLLAMA_META_CACHE_DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

type ShowFetchFn = dyn Fn(&str) -> Result<OllamaShowFields> + Send + Sync;
type EnrichOutcome = (usize, Result<OllamaTagsModel>);

/// Clamp parallel `/api/show` workers to the Phase Y band (`#370`).
pub fn clamp_ollama_meta_parallelism(requested: usize) -> usize {
    requested.clamp(2, 4)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    name: String,
    /// Digest string when Known; empty sentinel when Unknown (still keyed by name).
    digest: String,
}

impl CacheKey {
    fn from_model(model: &OllamaTagsModel) -> Self {
        let digest = match &model.digest {
            OllamaJsonField::Known(d) => d.clone(),
            OllamaJsonField::Unknown => String::new(),
        };
        Self {
            name: model.name.clone(),
            digest,
        }
    }
}

/// Process-local cache of enriched Ollama model rows (`#370`).
#[derive(Debug)]
pub struct OllamaMetaCache {
    endpoint: String,
    generation: u64,
    entries: HashMap<CacheKey, OllamaTagsModel>,
    timeout: Duration,
    max_parallel: usize,
}

impl OllamaMetaCache {
    /// Create an empty cache for `endpoint` (normalized via [`effective_ollama_host`]).
    pub fn new(endpoint: &str, timeout: Duration, max_parallel: usize) -> Self {
        Self {
            endpoint: effective_ollama_host(Some(endpoint)),
            generation: 0,
            entries: HashMap::new(),
            timeout,
            max_parallel: clamp_ollama_meta_parallelism(max_parallel),
        }
    }

    /// Current endpoint string.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Monotonic generation; bumps on [`Self::set_endpoint`].
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn max_parallel(&self) -> usize {
        self.max_parallel
    }

    /// Switch endpoint: bump generation and clear entries (no cross-server carry).
    pub fn set_endpoint(&mut self, endpoint: &str) {
        let next = effective_ollama_host(Some(endpoint));
        if next == self.endpoint {
            return;
        }
        self.endpoint = next;
        self.generation = self.generation.saturating_add(1);
        self.entries.clear();
    }

    /// Lookup by model name + digest under the current endpoint.
    pub fn get(&self, model: &OllamaTagsModel) -> Option<&OllamaTagsModel> {
        self.entries.get(&CacheKey::from_model(model))
    }

    /// Insert only when `observed_generation` still matches (`#370` late-reject).
    ///
    /// Returns `true` when stored; `false` when the generation is stale.
    pub fn put_if_current(&mut self, observed_generation: u64, model: OllamaTagsModel) -> bool {
        if observed_generation != self.generation {
            return false;
        }
        self.entries.insert(CacheKey::from_model(&model), model);
        true
    }

    /// Enrich tags rows: cache hits skip HTTP; misses call `fetch_show` with ≤N workers.
    ///
    /// `fetch_show(name) -> show fields`. Results are merged and stored only if the
    /// cache generation is still the one observed at the start of this call.
    pub fn enrich_models_with<F>(
        &mut self,
        models: &[OllamaTagsModel],
        fetch_show: F,
    ) -> Result<Vec<OllamaTagsModel>>
    where
        F: Fn(&str) -> Result<OllamaShowFields> + Send + Sync + 'static,
    {
        let peak = AtomicUsize::new(0);
        self.enrich_models_with_peak(models, fetch_show, &peak)
    }

    /// Same as [`Self::enrich_models_with`] but records peak concurrent fetches.
    pub fn enrich_models_with_peak<F>(
        &mut self,
        models: &[OllamaTagsModel],
        fetch_show: F,
        peak_out: &AtomicUsize,
    ) -> Result<Vec<OllamaTagsModel>>
    where
        F: Fn(&str) -> Result<OllamaShowFields> + Send + Sync + 'static,
    {
        let gen = self.generation;
        let max_parallel = self.max_parallel;
        let fetch_show: Arc<ShowFetchFn> = Arc::new(fetch_show);

        let mut out = Vec::with_capacity(models.len());
        let mut pending: Vec<(usize, OllamaTagsModel)> = Vec::new();

        for (i, model) in models.iter().enumerate() {
            if let Some(hit) = self.get(model) {
                out.push((i, hit.clone()));
                continue;
            }
            if !ollama_model_needs_show(model) {
                let stored = model.clone();
                let _ = self.put_if_current(gen, stored.clone());
                out.push((i, stored));
                continue;
            }
            pending.push((i, model.clone()));
        }

        if !pending.is_empty() {
            let results: Arc<Mutex<Vec<EnrichOutcome>>> = Arc::new(Mutex::new(Vec::new()));
            let inflight = Arc::new(AtomicUsize::new(0));
            let mut handles = Vec::new();

            for (idx, model) in pending {
                while inflight.load(Ordering::SeqCst) >= max_parallel {
                    thread::sleep(Duration::from_millis(1));
                }
                let now = inflight.fetch_add(1, Ordering::SeqCst) + 1;
                peak_out.fetch_max(now, Ordering::SeqCst);

                let fetch_show = Arc::clone(&fetch_show);
                let results = Arc::clone(&results);
                let inflight = Arc::clone(&inflight);
                let name = model.name.clone();
                handles.push(thread::spawn(move || {
                    let outcome =
                        (fetch_show)(&name).map(|show| merge_tags_with_show(&model, &show));
                    results.lock().expect("results mutex").push((idx, outcome));
                    inflight.fetch_sub(1, Ordering::SeqCst);
                }));
            }
            for h in handles {
                let _ = h.join();
            }
            for (idx, outcome) in results.lock().expect("results mutex").drain(..) {
                let model = outcome?;
                let _ = self.put_if_current(gen, model.clone());
                out.push((idx, model));
            }
        }

        out.sort_by_key(|(i, _)| *i);
        Ok(out.into_iter().map(|(_, m)| m).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn sample(name: &str, digest: OllamaJsonField<String>) -> OllamaTagsModel {
        OllamaTagsModel {
            name: name.into(),
            digest,
            size: OllamaJsonField::Known(1),
            remote_host: OllamaJsonField::Unknown,
            remote_model: OllamaJsonField::Unknown,
            capabilities: OllamaJsonField::Known(vec!["completion".into()]),
        }
    }

    /// `#370`: endpoint change bumps generation and drops cached rows.
    #[test]
    fn endpoint_change_bumps_generation_and_clears_entries() {
        let mut cache = OllamaMetaCache::new("http://127.0.0.1:11434", Duration::from_secs(1), 2);
        assert_eq!(cache.generation(), 0);
        let model = sample("a:latest", OllamaJsonField::Known("d1".into()));
        assert!(cache.put_if_current(0, model.clone()));
        assert!(cache.get(&model).is_some());

        cache.set_endpoint("http://127.0.0.1:11435");
        assert_eq!(cache.generation(), 1);
        assert!(
            cache.get(&model).is_none(),
            "availability must not carry across endpoints"
        );
        assert_eq!(cache.endpoint(), "http://127.0.0.1:11435");
    }

    /// `#370`: late put from an old generation is rejected.
    #[test]
    fn late_response_from_stale_generation_is_rejected() {
        let mut cache = OllamaMetaCache::new("http://127.0.0.1:11434", Duration::from_secs(1), 2);
        let gen0 = cache.generation();
        cache.set_endpoint("http://10.0.0.2:11434");
        let model = sample("late:latest", OllamaJsonField::Known("x".into()));
        assert!(
            !cache.put_if_current(gen0, model.clone()),
            "stale generation must not write"
        );
        assert!(cache.get(&model).is_none());
        assert!(cache.put_if_current(cache.generation(), model.clone()));
        assert!(cache.get(&model).is_some());
    }

    /// `#370`: parallelism stays within 2–4.
    #[test]
    fn enrich_parallelism_clamped_and_respected() {
        assert_eq!(clamp_ollama_meta_parallelism(1), 2);
        assert_eq!(clamp_ollama_meta_parallelism(9), 4);
        assert_eq!(clamp_ollama_meta_parallelism(3), 3);

        let mut cache = OllamaMetaCache::new("http://127.0.0.1:11434", Duration::from_secs(5), 9);
        assert_eq!(cache.max_parallel(), 4);

        let models: Vec<_> = (0..8)
            .map(|i| {
                sample(
                    &format!("m{i}:latest"),
                    OllamaJsonField::Known(format!("d{i}")),
                )
            })
            .collect();
        let peak = AtomicUsize::new(0);
        let started = Instant::now();
        let out = cache
            .enrich_models_with_peak(
                &models,
                move |_name| {
                    thread::sleep(Duration::from_millis(40));
                    Ok(OllamaShowFields {
                        remote_host: OllamaJsonField::Known(String::new()),
                        remote_model: OllamaJsonField::Known(String::new()),
                        capabilities: OllamaJsonField::Unknown,
                    })
                },
                &peak,
            )
            .unwrap();
        assert_eq!(out.len(), 8);
        let peak_n = peak.load(Ordering::SeqCst);
        assert!(peak_n <= 4, "peak concurrent fetches {peak_n} must be ≤ 4");
        assert!(peak_n >= 2, "expected parallel work, peak={peak_n}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    /// `#370`: cache hit skips a second fetch for the same endpoint/name/digest.
    #[test]
    fn cache_hit_skips_second_fetch() {
        let mut cache = OllamaMetaCache::new("http://127.0.0.1:11434", Duration::from_secs(1), 2);
        let model = sample("phi:latest", OllamaJsonField::Known("abc".into()));
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_f = Arc::clone(&calls);
        let once = cache
            .enrich_models_with(std::slice::from_ref(&model), move |_| {
                calls_f.fetch_add(1, Ordering::SeqCst);
                Ok(OllamaShowFields {
                    remote_host: OllamaJsonField::Known(String::new()),
                    remote_model: OllamaJsonField::Known(String::new()),
                    capabilities: OllamaJsonField::Unknown,
                })
            })
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!once[0].remote_host.is_unknown());

        let calls2 = Arc::new(AtomicUsize::new(0));
        let calls2_f = Arc::clone(&calls2);
        let _ = cache
            .enrich_models_with(std::slice::from_ref(&model), move |_| {
                calls2_f.fetch_add(1, Ordering::SeqCst);
                panic!("must not fetch on cache hit");
            })
            .unwrap();
        assert_eq!(calls2.load(Ordering::SeqCst), 0);
    }

    /// `#370`: same name, different digest → distinct cache entries.
    #[test]
    fn digest_is_part_of_cache_key() {
        let mut cache = OllamaMetaCache::new("http://127.0.0.1:11434", Duration::from_secs(1), 2);
        let a = sample("phi:latest", OllamaJsonField::Known("digest-a".into()));
        let b = sample("phi:latest", OllamaJsonField::Known("digest-b".into()));
        assert!(cache.put_if_current(0, a.clone()));
        assert!(cache.get(&a).is_some());
        assert!(cache.get(&b).is_none());
    }
}
