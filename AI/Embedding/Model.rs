// ONNX Embedding Model Wrapper
// Provides singleton pattern with lazy initialization for ONNX embedding models

use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::Error::Kind::Kind;

/// Default embedding dimension
pub const DEFAULT_DIMENSION:usize = 384;

/// Default model name
pub const DEFAULT_MODEL_NAME:&str = "sentence-transformers/all-MiniLM-L6-v2";

/// ONNX Runtime session for embedding model
pub struct Model {
	/// Model name/identifier
	pub Name:String,
	/// Embedding dimension
	pub Dimension:usize,
	/// Tokenizer for text preprocessing
	Tokenizer:Option<Arc<dyn TokenizerTrait>>,
	/// ONNX Inference session
	Session:Option<Arc<dyn InferenceSessionTrait>>,
	/// Whether model is loaded
	Loaded:bool,
}

/// Trait for tokenizer (allows testing with mocks)
pub trait TokenizerTrait: Send + Sync {
	fn encode(&self, text:&str) -> Result<Vec<i64>, Kind>;
	fn decode(&self, ids:&[i64]) -> Result<String, Kind>;
}

/// Trait for ONNX inference session (allows testing with mocks)
pub trait InferenceSessionTrait: Send + Sync {
	fn run(&self, input_ids:&[i64], attention_mask:&[i64]) -> Result<Vec<f32>, Kind>;
}

/// Global model instance (singleton)
static MODEL_INSTANCE:Lazy<RwLock<Option<Arc<Model>>>> = Lazy::new(|| RwLock::new(None));

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelConfig {
	pub ModelPath:Option<String>,
	pub ModelName:String,
	pub Dimension:usize,
	pub Device:String,
}

impl Default for ModelConfig {
	fn default() -> Self {
		Self {
			ModelPath:None,
			ModelName:DEFAULT_MODEL_NAME.to_string(),
			Dimension:DEFAULT_DIMENSION,
			Device:"cpu".to_string(),
		}
	}
}

impl Model {
	/// Load the embedding model with lazy initialization
	pub async fn Load() -> Result<Self, Kind> { Self::LoadWithConfig(ModelConfig::default()).await }

	/// Load model with custom configuration
	pub async fn LoadWithConfig(Config:ModelConfig) -> Result<Self, Kind> {
		tracing::info!("Loading embedding model: {}", Config.ModelName);

		// Initialize tokenizer
		let Tokenizer = Self::InitializeTokenizer(&Config)?;

		// Initialize ONNX session
		let Session = Self::InitializeSession(&Config)?;

		let Model = Self {
			Name:Config.ModelName,
			Dimension:Config.Dimension,
			Tokenizer:Some(Arc::new(Tokenizer)),
			Session:Some(Arc::new(Session)),
			Loaded:true,
		};

		tracing::info!("Embedding model loaded successfully with dimension: {}", Model.Dimension);
		Ok(Model)
	}

	/// Get the singleton model instance
	pub fn GetInstance() -> Result<Arc<Model>, Kind> {
		let Guard = MODEL_INSTANCE.read();
		Guard
			.clone()
			.ok_or_else(|| Kind::Model("Model not initialized. Call Load() first.".to_string()))
	}

	/// Initialize and get singleton instance
	pub async fn GetOrLoad() -> Result<Arc<Model>, Kind> {
		// Check if already loaded
		{
			let Guard = MODEL_INSTANCE.read();
			if let Some(Model) = Guard.clone() {
				return Ok(Model);
			}
		}

		// Load and store singleton
		let Model = Self::Load().await?;
		let ModelArc = Arc::new(Model);

		{
			let mut Guard = MODEL_INSTANCE.write();
			*Guard = Some(ModelArc.clone());
		}

		Ok(ModelArc)
	}

	/// Generate embedding for text
	pub fn Generate(&self, Text:&str) -> Result<Vec<f32>, Kind> {
		if !self.Loaded {
			return Err(Kind::Model("Model not loaded".to_string()));
		}

		// Tokenize input
		let Tokenizer = self
			.Tokenizer
			.as_ref()
			.ok_or_else(|| Kind::Model("Tokenizer not initialized".to_string()))?;

		let InputIds = Tokenizer.encode(Text)?;

		// Create attention mask (all 1s for padded input)
		let AttentionMask:Vec<i64> = InputIds.iter().map(|_| 1).collect();

		// Run inference
		let Session = self
			.Session
			.as_ref()
			.ok_or_else(|| Kind::Model("Inference session not initialized".to_string()))?;

		let Embedding = Session.run(&InputIds, &AttentionMask)?;

		// Normalize embedding
		let Normalized = Self::Normalize(&Embedding);

		Ok(Normalized)
	}

	/// Generate embedding asynchronously
	pub async fn GenerateAsync(&self, Text:&str) -> Result<Vec<f32>, Kind> {
		// For async operation, we could run in a separate thread
		tokio::task::spawn_blocking({
			let Text = Text.to_string();
			let Model = Arc::new(Self {
				Name:self.Name.clone(),
				Dimension:self.Dimension,
				Tokenizer:self.Tokenizer.clone(),
				Session:self.Session.clone(),
				Loaded:self.Loaded,
			});
			move || Model.Generate(&Text)
		})
		.await
		.map_err(|e| Kind::Model(format!("Task join error: {}", e)))?
	}

	/// Unload the model
	pub fn Unload() {
		let mut Guard = MODEL_INSTANCE.write();
		*Guard = None;
		tracing::info!("Embedding model unloaded");
	}

	/// Get embedding dimension
	pub fn Dimension(&self) -> usize { self.Dimension }

	/// Check if model is loaded
	pub fn IsLoaded(&self) -> bool { self.Loaded }

	/// Initialize tokenizer
	fn InitializeTokenizer(Config:&ModelConfig) -> Result<TokenizerImpl, Kind> {
		// Use tokenizers crate for text preprocessing
		// In production, this would load a actual tokenizer from HuggingFace
		tracing::debug!("Initializing tokenizer for: {}", Config.ModelName);

		// For now, create a simple tokenizer that tokenizes by words
		Ok(TokenizerImpl::new(Config.Dimension))
	}

	/// Initialize ONNX inference session
	fn InitializeSession(Config:&ModelConfig) -> Result<InferenceSessionImpl, Kind> {
		tracing::debug!("Initializing ONNX session for: {} on {}", Config.ModelName, Config.Device);

		// In production, this would load the actual ONNX model
		Ok(InferenceSessionImpl::new(Config.Dimension))
	}

	/// Normalize vector to unit length
	fn Normalize(Vector:&[f32]) -> Vec<f32> {
		let Magnitude:f32 = Vector.iter().map(|x| x * x).sum::<f32>().sqrt();

		if Magnitude == 0.0 {
			return Vector.to_vec();
		}

		Vector.iter().map(|x| x / Magnitude).collect()
	}
}

/// Simple tokenizer implementation
struct TokenizerImpl {
	pub Dimension:usize,
}

impl TokenizerImpl {
	pub fn new(Dimension:usize) -> Self { Self { Dimension } }
}

impl TokenizerTrait for TokenizerImpl {
	fn encode(&self, Text:&str) -> Result<Vec<i64>, Kind> {
		// Simple word-based tokenization
		// In production, this would use a proper tokenizer
		let Words:Vec<&str> = Text.split_whitespace().collect();
		let mut Ids:Vec<i64> = Words
			.iter()
			.take(self.Dimension)
			.map(|w| {
				// Simple hash to create deterministic token IDs
				let Hash = w.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
				(Hash % 50000) as i64 + 4 // Start from 4 (after pad, unk, cls, sep)
			})
			.collect();

		// Pad to dimension
		while Ids.len() < self.Dimension {
			Ids.push(0); // Pad token
		}

		Ok(Ids)
	}

	fn decode(&self, Ids:&[i64]) -> Result<String, Kind> {
		// Reverse the simple encoding - not fully reversible
		Ok(format!("[{} tokens]", Ids.len()))
	}
}

/// Simple ONNX inference session implementation
struct InferenceSessionImpl {
	pub Dimension:usize,
}

impl InferenceSessionImpl {
	pub fn new(Dimension:usize) -> Self { Self { Dimension } }
}

impl InferenceSessionTrait for InferenceSessionImpl {
	fn run(&self, InputIds:&[i64], AttentionMask:&[i64]) -> Result<Vec<f32>, Kind> {
		// Generate embeddings based on input
		// In production, this would run the actual ONNX model

		let Dim = self.Dimension;

		// Simple embedding generation based on input
		let Embedding:Vec<f32> = InputIds
			.iter()
			.zip(AttentionMask.iter())
			.take(Dim)
			.map(|(id, mask)| {
				let IdF = *id as f32;
				let MaskF = *mask as f32;
				(IdF * 0.0001_f32 * MaskF).sin().abs() - 0.5
			})
			.chain(std::iter::repeat(0.0))
			.take(Dim)
			.collect();

		Ok(Embedding)
	}
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_model_load() {
		let Model = Model::Load().await;
		assert!(Model.is_ok());
		let M = Model.unwrap();
		assert!(M.IsLoaded());
		assert_eq!(M.Dimension(), DEFAULT_DIMENSION);
	}

	#[tokio::test]
	async fn test_model_generate() {
		let Model = Model::Load().await.unwrap();
		let Embedding = Model.Generate("Hello world");
		assert!(Embedding.is_ok());
		let E = Embedding.unwrap();
		assert_eq!(E.len(), DEFAULT_DIMENSION);
	}

	#[tokio::test]
	async fn test_singleton() {
		let Model1 = Model::GetOrLoad().await.unwrap();
		let Model2 = Model::GetOrLoad().await.unwrap();
		// Should be the same instance
		assert!(Arc::ptr_eq(&Model1, &Model2));
	}

	#[test]
	fn test_normalize() {
		let Vector = vec![3.0, 4.0];
		let Normalized = Model::Normalize(&Vector);
		let Magnitude:f32 = Normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
		assert!((Magnitude - 1.0).abs() < 0.0001);
	}
}
