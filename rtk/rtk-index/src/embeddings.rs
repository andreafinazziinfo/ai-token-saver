#[cfg(feature = "embeddings")]
use anyhow::{anyhow, Result};
#[cfg(feature = "embeddings")]
use std::path::Path;
#[cfg(feature = "embeddings")]
use tokenizers::Tokenizer;
#[cfg(feature = "embeddings")]
use tract_onnx::prelude::*;

#[cfg(feature = "embeddings")]
pub struct OnnxEmbedder {
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    tokenizer: Tokenizer,
}

#[cfg(feature = "embeddings")]
impl OnnxEmbedder {
    pub fn load_model(model_path: &Path, tokenizer_path: &Path) -> Result<Self> {
        let model = tract_onnx::onnx()
            .model_for_path(model_path)?
            .into_runnable()?;
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow!(e))?;
        Ok(Self { model, tokenizer })
    }

    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let encoding = self.tokenizer.encode(text, true).map_err(|e| anyhow!(e))?;
        let ids = encoding
            .get_ids()
            .iter()
            .map(|&x| x as i64)
            .collect::<Vec<i64>>();
        let attention_mask = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect::<Vec<i64>>();
        let seq_len = ids.len();

        if seq_len == 0 {
            return Err(anyhow!("Empty text tokenization"));
        }

        // Convert to 2D arrays with shape [1, seq_len]
        let input_ids_tensor = tract_ndarray::Array2::from_shape_vec((1, seq_len), ids.clone())?;
        let attention_mask_tensor =
            tract_ndarray::Array2::from_shape_vec((1, seq_len), attention_mask.clone())?;

        // Run model
        let outputs = self.model.run(tvec!(
            input_ids_tensor.into_tensor(),
            attention_mask_tensor.into_tensor()
        ))?;

        // Extract last_hidden_state (usually index 0)
        let output_tensor = outputs[0].to_array_view::<f32>()?;
        let shape = output_tensor.shape();

        if shape.len() < 3 {
            return Err(anyhow!("Unexpected output tensor shape: {:?}", shape));
        }

        let hidden_size = shape[2];
        let mut mean_embedding = vec![0.0f32; hidden_size];
        let mut count = 0.0f32;

        for i in 0..seq_len {
            if attention_mask[i] > 0 {
                for j in 0..hidden_size {
                    mean_embedding[j] += output_tensor[[0, i, j]];
                }
                count += 1.0;
            }
        }

        if count > 0.0 {
            for j in 0..hidden_size {
                mean_embedding[j] /= count;
            }
        }

        // L2 Normalization
        let mut norm = 0.0f32;
        for &val in &mean_embedding {
            norm += val * val;
        }
        norm = norm.sqrt();
        if norm > 0.0 {
            for val in &mut mean_embedding {
                *val /= norm;
            }
        }

        Ok(mean_embedding)
    }
}

#[cfg(feature = "embeddings")]
pub fn dot_product(v1: &[f32], v2: &[f32]) -> f32 {
    v1.iter().zip(v2.iter()).map(|(x, y)| x * y).sum()
}

// Fallback dummy structs for compilation when feature is disabled
#[cfg(not(feature = "embeddings"))]
use anyhow::{anyhow, Result};

#[cfg(not(feature = "embeddings"))]
pub struct OnnxEmbedder;

#[cfg(not(feature = "embeddings"))]
impl OnnxEmbedder {
    pub fn load_model(
        _model_path: &std::path::Path,
        _tokenizer_path: &std::path::Path,
    ) -> Result<Self> {
        Err(anyhow!(
            "Embeddings feature is disabled. Recompile with --features embeddings"
        ))
    }
    pub fn embed_text(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow!(
            "Embeddings feature is disabled. Recompile with --features embeddings"
        ))
    }
}

#[cfg(not(feature = "embeddings"))]
pub fn dot_product(_v1: &[f32], _v2: &[f32]) -> f32 {
    0.0
}
