use crate::conversation::Conversation;
use crate::funnel::Embedder;
use crate::funnel::Funnel;
use crate::funnel::FunnelNode;
use crate::math;
use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::Arc;

/// A cluster-based funnel builder using embeddings and k-means.
pub struct ClusterFunnelBuilder {
    pub k: usize,
    pub iterations: usize,
    /// Convergence tolerance (max centroid movement under this is considered converged)
    pub tol: f32,
    /// Optional deterministic seed for randomized steps. If None, seeding is deterministic
    /// based on the data (max-min heuristic).
    pub seed: Option<u64>,
}

impl ClusterFunnelBuilder {
    pub fn new(k: usize, iterations: usize) -> Self {
        Self { k, iterations, tol: 1e-4, seed: None }
    }

    /// Create a builder with an explicit RNG seed (useful for tests wanting deterministic
    /// but randomized seeding behavior).
    pub fn with_seed(k: usize, iterations: usize, seed: u64) -> Self {
        Self { k, iterations, tol: 1e-4, seed: Some(seed) }
    }

    /// Build clusters using an embedder, create Funnel and FunnelNode objects,
    /// and persist them into the provided storage.
    pub async fn build_and_persist<S: Storage + 'static>(
        &self,
        id: &str,
        convs: &[Conversation],
        embedder: Arc<dyn Embedder>,
        storage: Arc<S>,
    ) -> Result<Funnel, String> {
        if convs.is_empty() {
            // Persist empty funnel
            let funnel = Funnel::new(id);
            let bytes = bincode::serialize(&funnel).map_err(|e| e.to_string())?;
            let key = format!("funnel:builder:{}", id);
            storage.put_blob(&key, &bytes).await?;
            return Ok(funnel);
        }

        // compute embeddings
        let mut vecs: Vec<Vec<f32>> = Vec::with_capacity(convs.len());
        for c in convs {
            // embed the intent purpose (short summary) for now
            let emb = embedder.embed(&c.intent.purpose).await.map_err(|e| e.to_string())?;
            vecs.push(emb);
        }

        // normalize vectors for cosine similarity and run spherical k-means
        for v in vecs.iter_mut() {
            // best-effort normalization; leave zero vectors as-is
            let _ = math::normalize_inplace(v, 1e-12);
        }
        // choose k. If self.k == 0 then run automatic selection (elbow) up to min(10,n)
        let mut k = if self.k == 0 {
            let max_k = std::cmp::min(10usize, vecs.len());
            select_k_by_elbow(&vecs[..], max_k, self.iterations, self.seed)
        } else {
            std::cmp::min(self.k, vecs.len())
        };
        // If selector returned 1 but there are multiple distinct embeddings,
        // prefer a k equal to the number of unique embeddings (bounded by vecs.len()).
        // Only apply this heuristic when automatic selection was used (self.k == 0).
        if k == 1 && self.k == 0 {
            let mut uniques: Vec<&Vec<f32>> = Vec::new();
            for v in vecs.iter() {
                if !uniques
                    .iter()
                    .any(|u| u.iter().zip(v.iter()).all(|(a, b)| (a - b).abs() < 1e-6))
                {
                    uniques.push(v);
                }
            }
            let uniq = uniques.len();
            if uniq > 1 {
                k = std::cmp::min(uniq, vecs.len());
            }
        }
        let centroids = spherical_kmeans(&vecs[..], k, self.iterations, self.tol, self.seed);

        // assign points
        let mut clusters: Vec<Vec<usize>> = vec![Vec::new(); k];
        for (i, v) in vecs.iter().enumerate() {
            let mut best = 0usize;
            let mut best_d = f32::MAX;
            for (ci, c) in centroids.iter().enumerate() {
                let d = math::cosine_distance(v, c, 1e-12);
                if d < best_d {
                    best_d = d;
                    best = ci;
                }
            }
            clusters[best].push(i);
        }

        // create nodes and persist
        let mut funnel = Funnel::new(id);
        for (ci, members) in clusters.into_iter().enumerate() {
            let node_id = format!("{}:n{}", id, ci);
            let mut node = FunnelNode::new(&node_id);
            node.centroid = Some(centroids[ci].clone());
            for idx in members.iter() {
                node.example_ids.push(convs[*idx].id.clone());
            }

            // persist node
            let nbytes = bincode::serialize(&node).map_err(|e| e.to_string())?;
            let nkey = format!("funnel:node:{}", node_id);
            storage.put_blob(&nkey, &nbytes).await?;
            funnel.node_ids.push(node_id);
        }

        // persist funnel
        let bytes = bincode::serialize(&funnel).map_err(|e| e.to_string())?;
        let key = format!("funnel:builder:{}", id);
        storage.put_blob(&key, &bytes).await?;
        Ok(funnel)
    }
}

// Use `crate::math` helpers (dot, centroid, cosine_distance) for numerical
// stability and reuse across the codebase.

// k-means helpers moved to `crate::funnel::kmeans` to reduce this file's size.

/// Select k by computing silhouette scores for candidate k values from 1..=max_k
/// and returning the k with the highest average silhouette. Ties choose the smaller k.
use crate::funnel::kmeans::*;
// Re-export selected kmeans helpers for backward compatibility with tests
pub use crate::funnel::kmeans::spherical_kmeans;

pub struct SimpleFunnelBuilder {}

impl SimpleFunnelBuilder {
    pub fn new() -> Self {
        Self {}
    }
    /// Build a funnel grouping conversations by their first topic (if present)
    pub async fn build_and_persist<S: Storage + 'static>(
        &self,
        id: &str,
        convs: &[Conversation],
        storage: Arc<S>,
    ) -> Result<Funnel, String> {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        for c in convs {
            let key = c.intent.topics.first().cloned().unwrap_or_else(|| "__misc__".to_string());
            groups.entry(key).or_default().push(c.id.clone());
        }

        let funnel = Funnel::new(id);
        let bytes = bincode::serialize(&funnel).map_err(|e| e.to_string())?;
        let key = format!("funnel:builder:{}", id);
        storage.put_blob(&key, &bytes).await?;
        Ok(funnel)
    }
}

impl Default for SimpleFunnelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
