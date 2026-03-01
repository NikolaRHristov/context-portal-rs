// VectorStore for the ConPort MCP server
use std::collections::HashMap;

pub struct Store {
	pub Embeddings:HashMap<String, Vec<f32>>,
}

impl Store {
	pub fn New() -> Self { Self { Embeddings:HashMap::new() } }

	pub fn Add(&mut self, Id:String, Embedding:Vec<f32>) -> Result<(), crate::Error::Kind::Kind> {
		if Embedding.len() != crate::AI::Embedding::Generate::Generate::Dimension() {
			return Err(crate::Error::Kind::Kind::VectorStore(
				"Embedding dimension mismatch".to_string(),
			));
		}

		self.Embeddings.insert(Id, Embedding);
		Ok(())
	}

	pub fn Get(&self, Id:&str) -> Result<&Vec<f32>, crate::Error::Kind::Kind> {
		self.Embeddings
			.get(Id)
			.ok_or_else(|| crate::Error::Kind::Kind::NotFound(format!("Embedding {} not found", Id)))
	}

	pub fn Search(&self, Query:&[f32], Limit:usize) -> Result<Vec<(String, f32)>, crate::Error::Kind::Kind> {
		if Query.len() != crate::AI::Embedding::Generate::Generate::Dimension() {
			return Err(crate::Error::Kind::Kind::VectorStore("Query dimension mismatch".to_string()));
		}

		let mut Results:Vec<(String, f32)> = self
			.Embeddings
			.iter()
			.map(|(Id, Embedding)| {
				let Similarity = Self::CosineSimilarity(Query, Embedding);
				(Id.clone(), Similarity)
			})
			.collect();

		Results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

		Results.truncate(Limit);
		Ok(Results)
	}

	fn CosineSimilarity(A:&[f32], B:&[f32]) -> f32 {
		let DotProduct:f32 = A.iter().zip(B.iter()).map(|(a, b)| a * b).sum();
		let MagnitudeA:f32 = A.iter().map(|a| a * a).sum::<f32>().sqrt();
		let MagnitudeB:f32 = B.iter().map(|b| b * b).sum::<f32>().sqrt();

		if MagnitudeA == 0.0 || MagnitudeB == 0.0 {
			return 0.0;
		}

		DotProduct / (MagnitudeA * MagnitudeB)
	}
}

impl Default for Store {
	fn default() -> Self { Self::New() }
}
