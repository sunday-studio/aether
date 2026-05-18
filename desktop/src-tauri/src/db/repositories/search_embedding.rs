use crate::error::{AppError, Result};
use crate::utils::embeddings::generate_embedding;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SearchEmbeddingInput {
    pub search_document_id: String,
    pub model_name: String,
    pub dimensions: i64,
    pub vector: Vec<f32>,
    pub text_hash: String,
}

#[derive(Debug, Clone)]
pub struct SearchEmbedding {
    pub id: String,
    pub search_document_id: String,
    pub model_name: String,
    pub dimensions: i64,
    pub vector: Vec<f32>,
    pub text_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchEmbeddingModelStatus {
    pub model_name: String,
    pub dimensions: i64,
    pub total_embeddings: i64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchEmbeddingStatus {
    pub total_embeddings: i64,
    pub models: Vec<SearchEmbeddingModelStatus>,
}

pub struct SearchEmbeddingRepository {
    database: Arc<Database>,
}

impl SearchEmbeddingRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub async fn upsert_embedding(&self, input: SearchEmbeddingInput) -> Result<()> {
        if input.dimensions <= 0 {
            return Err(AppError::BadRequest(
                "Embedding dimensions must be greater than zero".to_string(),
            ));
        }
        if input.vector.len() != input.dimensions as usize {
            return Err(AppError::BadRequest(format!(
                "Embedding vector length {} does not match dimensions {}",
                input.vector.len(),
                input.dimensions
            )));
        }

        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let now = Utc::now().to_rfc3339();
        let id = format!("{}:{}", input.search_document_id, input.model_name);
        let vector = encode_vector(&input.vector);

        conn.execute(
            "INSERT INTO search_embeddings (
                id, search_document_id, model_name, dimensions, vector, text_hash, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(search_document_id, model_name) DO UPDATE SET
                dimensions = excluded.dimensions,
                vector = excluded.vector,
                text_hash = excluded.text_hash,
                updated_at = excluded.updated_at",
            libsql::params![
                id,
                input.search_document_id,
                input.model_name,
                input.dimensions,
                vector,
                input.text_hash,
                now.clone(),
                now,
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        Ok(())
    }

    pub async fn index_all_embeddings(&self, model_name: &str) -> Result<SearchEmbeddingStatus> {
        let documents = self.load_documents(None, None).await?;
        for document in documents {
            self.index_document_embedding(document, model_name).await?;
        }
        self.status().await
    }

    pub async fn index_resource_embeddings(
        &self,
        resource_type: &str,
        resource_id: &str,
        model_name: &str,
    ) -> Result<SearchEmbeddingStatus> {
        let documents = self
            .load_documents(Some(resource_type), Some(resource_id))
            .await?;
        if documents.is_empty() {
            self.delete_for_resource(resource_type, resource_id).await?;
            return self.status().await;
        }

        for document in documents {
            self.index_document_embedding(document, model_name).await?;
        }
        self.status().await
    }

    pub async fn refresh_existing_resource_embeddings(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<()> {
        let documents = self
            .load_documents(Some(resource_type), Some(resource_id))
            .await?;
        if documents.is_empty() {
            self.delete_for_resource(resource_type, resource_id).await?;
            return Ok(());
        }

        let model_names = self.load_indexed_model_names().await?;
        for model_name in model_names {
            for document in documents.clone() {
                self.index_document_embedding(document, &model_name).await?;
            }
        }

        Ok(())
    }

    pub async fn find_by_document_and_model(
        &self,
        search_document_id: &str,
        model_name: &str,
    ) -> Result<Option<SearchEmbedding>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, search_document_id, model_name, dimensions, vector, text_hash, created_at, updated_at
                 FROM search_embeddings
                 WHERE search_document_id = ?1 AND model_name = ?2",
                libsql::params![search_document_id, model_name],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let Some(row) = rows.next().await.map_err(AppError::LibSQL)? else {
            return Ok(None);
        };

        let dimensions: i64 = row.get(3).map_err(AppError::LibSQL)?;
        let vector_bytes: Vec<u8> = row.get(4).map_err(AppError::LibSQL)?;

        Ok(Some(SearchEmbedding {
            id: row.get(0).map_err(AppError::LibSQL)?,
            search_document_id: row.get(1).map_err(AppError::LibSQL)?,
            model_name: row.get(2).map_err(AppError::LibSQL)?,
            dimensions,
            vector: decode_vector(&vector_bytes, dimensions)?,
            text_hash: row.get(5).map_err(AppError::LibSQL)?,
            created_at: row.get(6).map_err(AppError::LibSQL)?,
            updated_at: row.get(7).map_err(AppError::LibSQL)?,
        }))
    }

    pub async fn delete_for_document(&self, search_document_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        conn.execute(
            "DELETE FROM search_embeddings WHERE search_document_id = ?1",
            libsql::params![search_document_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
        Ok(())
    }

    pub async fn delete_for_resource(&self, resource_type: &str, resource_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        conn.execute(
            "DELETE FROM search_embeddings
             WHERE search_document_id IN (
                SELECT id FROM search_documents WHERE resource_type = ?1 AND resource_id = ?2
             )",
            libsql::params![resource_type, resource_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
        Ok(())
    }

    pub async fn status(&self) -> Result<SearchEmbeddingStatus> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let total_embeddings = count_embeddings(&conn).await?;
        let mut rows = conn
            .query(
                "SELECT model_name, dimensions, COUNT(*)
                 FROM search_embeddings
                 GROUP BY model_name, dimensions
                 ORDER BY model_name, dimensions",
                libsql::params![],
            )
            .await
            .map_err(AppError::LibSQL)?;
        let mut models = Vec::new();

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            models.push(SearchEmbeddingModelStatus {
                model_name: row.get(0).map_err(AppError::LibSQL)?,
                dimensions: row.get(1).map_err(AppError::LibSQL)?,
                total_embeddings: row.get(2).map_err(AppError::LibSQL)?,
            });
        }

        Ok(SearchEmbeddingStatus {
            total_embeddings,
            models,
        })
    }

    async fn load_documents(
        &self,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
    ) -> Result<Vec<SearchEmbeddingDocument>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows =
            if let (Some(resource_type), Some(resource_id)) = (resource_type, resource_id) {
                conn.query(
                    "SELECT id, text, text_hash
                 FROM search_documents
                 WHERE resource_type = ?1 AND resource_id = ?2
                 ORDER BY id",
                    libsql::params![resource_type, resource_id],
                )
                .await
                .map_err(AppError::LibSQL)?
            } else {
                conn.query(
                    "SELECT id, text, text_hash
                 FROM search_documents
                 ORDER BY id",
                    libsql::params![],
                )
                .await
                .map_err(AppError::LibSQL)?
            };

        let mut documents = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            documents.push(SearchEmbeddingDocument {
                id: row.get(0).map_err(AppError::LibSQL)?,
                text: row.get(1).map_err(AppError::LibSQL)?,
                text_hash: row.get(2).map_err(AppError::LibSQL)?,
            });
        }

        Ok(documents)
    }

    async fn load_indexed_model_names(&self) -> Result<Vec<String>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT DISTINCT model_name
                 FROM search_embeddings
                 ORDER BY model_name",
                libsql::params![],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let mut model_names = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            model_names.push(row.get(0).map_err(AppError::LibSQL)?);
        }

        Ok(model_names)
    }

    async fn index_document_embedding(
        &self,
        document: SearchEmbeddingDocument,
        model_name: &str,
    ) -> Result<()> {
        if document.text.trim().is_empty() {
            self.delete_for_document(&document.id).await?;
            return Ok(());
        }

        let vector = generate_embedding(&document.text).await?;
        self.upsert_embedding(SearchEmbeddingInput {
            search_document_id: document.id,
            model_name: model_name.to_string(),
            dimensions: vector.len() as i64,
            vector,
            text_hash: document.text_hash,
        })
        .await
    }
}

#[derive(Debug, Clone)]
struct SearchEmbeddingDocument {
    id: String,
    text: String,
    text_hash: String,
}

async fn count_embeddings(conn: &libsql::Connection) -> Result<i64> {
    let mut rows = conn
        .query("SELECT COUNT(*) FROM search_embeddings", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;

    if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        Ok(row.get(0).map_err(AppError::LibSQL)?)
    } else {
        Ok(0)
    }
}

fn encode_vector(vector: &[f32]) -> Vec<u8> {
    vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn decode_vector(bytes: &[u8], dimensions: i64) -> Result<Vec<f32>> {
    let expected_len = dimensions as usize * std::mem::size_of::<f32>();
    if bytes.len() != expected_len {
        return Err(AppError::Internal(format!(
            "Stored embedding byte length {} does not match dimensions {}",
            bytes.len(),
            dimensions
        )));
    }

    Ok(bytes
        .chunks_exact(std::mem::size_of::<f32>())
        .map(|chunk| {
            let mut value = [0u8; 4];
            value.copy_from_slice(chunk);
            f32::from_le_bytes(value)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::db::repositories::search_document::{SearchDocumentInput, SearchDocumentRepository};
    use libsql::Builder;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    async fn test_repos() -> (
        Arc<Database>,
        SearchDocumentRepository,
        SearchEmbeddingRepository,
        PathBuf,
    ) {
        let id = DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let db_path = std::env::temp_dir().join(format!(
            "aether-search-embedding-test-{}-{}.db",
            std::process::id(),
            id
        ));
        let database = Builder::new_local(&db_path)
            .build()
            .await
            .expect("create test database");
        migrations::run_migrations(&database)
            .await
            .expect("run migrations");
        let database = Arc::new(database);
        let document_repo = SearchDocumentRepository::new(database.clone());
        let embedding_repo = SearchEmbeddingRepository::new(database.clone());
        (database, document_repo, embedding_repo, db_path)
    }

    async fn seed_search_document(repo: &SearchDocumentRepository) {
        repo.upsert_document(SearchDocumentInput {
            resource_type: "entry".to_string(),
            resource_id: "entry-1".to_string(),
            chunk_index: 0,
            title: "Entry one".to_string(),
            text: "Search document text".to_string(),
            source_updated_at: "2026-05-18T09:00:00Z".to_string(),
        })
        .await
        .expect("seed search document");
    }

    fn cleanup_db(path: PathBuf) {
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }

    #[tokio::test]
    async fn upsert_embedding_stores_model_scoped_vector() {
        let (_database, document_repo, embedding_repo, db_path) = test_repos().await;
        seed_search_document(&document_repo).await;

        embedding_repo
            .upsert_embedding(SearchEmbeddingInput {
                search_document_id: "entry:entry-1:0".to_string(),
                model_name: "local-test-model".to_string(),
                dimensions: 3,
                vector: vec![0.1, 0.2, 0.3],
                text_hash: "hash-one".to_string(),
            })
            .await
            .expect("upsert embedding");

        let embedding = embedding_repo
            .find_by_document_and_model("entry:entry-1:0", "local-test-model")
            .await
            .expect("find embedding")
            .expect("embedding exists");
        let status = embedding_repo.status().await.expect("embedding status");

        assert_eq!(embedding.search_document_id, "entry:entry-1:0");
        assert_eq!(embedding.model_name, "local-test-model");
        assert_eq!(embedding.dimensions, 3);
        assert_eq!(embedding.vector, vec![0.1, 0.2, 0.3]);
        assert_eq!(embedding.text_hash, "hash-one");
        assert_eq!(status.total_embeddings, 1);
        assert_eq!(status.models[0].model_name, "local-test-model");

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn upsert_embedding_rejects_dimension_mismatch() {
        let (_database, document_repo, embedding_repo, db_path) = test_repos().await;
        seed_search_document(&document_repo).await;

        let error = embedding_repo
            .upsert_embedding(SearchEmbeddingInput {
                search_document_id: "entry:entry-1:0".to_string(),
                model_name: "local-test-model".to_string(),
                dimensions: 3,
                vector: vec![0.1, 0.2],
                text_hash: "hash-one".to_string(),
            })
            .await
            .expect_err("dimension mismatch should fail");

        assert!(matches!(error, AppError::BadRequest(_)));

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn deleting_search_document_removes_embedding_rows() {
        let (_database, document_repo, embedding_repo, db_path) = test_repos().await;
        seed_search_document(&document_repo).await;
        embedding_repo
            .upsert_embedding(SearchEmbeddingInput {
                search_document_id: "entry:entry-1:0".to_string(),
                model_name: "local-test-model".to_string(),
                dimensions: 3,
                vector: vec![0.1, 0.2, 0.3],
                text_hash: "hash-one".to_string(),
            })
            .await
            .expect("upsert embedding");

        document_repo
            .delete_resource("entry", "entry-1")
            .await
            .expect("delete search document");
        let status = embedding_repo.status().await.expect("embedding status");

        assert_eq!(status.total_embeddings, 0);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn index_all_embeddings_generates_local_vectors_for_search_documents() {
        let (_database, document_repo, embedding_repo, db_path) = test_repos().await;
        seed_search_document(&document_repo).await;

        let status = embedding_repo
            .index_all_embeddings("local-hash-384")
            .await
            .expect("index all embeddings");
        let embedding = embedding_repo
            .find_by_document_and_model("entry:entry-1:0", "local-hash-384")
            .await
            .expect("find embedding")
            .expect("embedding exists");

        assert_eq!(status.total_embeddings, 1);
        assert_eq!(embedding.dimensions, 384);
        assert_eq!(embedding.vector.len(), 384);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn index_resource_embeddings_deletes_missing_resource_embeddings() {
        let (_database, document_repo, embedding_repo, db_path) = test_repos().await;
        seed_search_document(&document_repo).await;
        embedding_repo
            .index_all_embeddings("local-hash-384")
            .await
            .expect("index all embeddings");
        document_repo
            .delete_resource("entry", "entry-1")
            .await
            .expect("delete search document");

        let status = embedding_repo
            .index_resource_embeddings("entry", "entry-1", "local-hash-384")
            .await
            .expect("index missing resource");

        assert_eq!(status.total_embeddings, 0);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn refresh_existing_resource_embeddings_updates_stale_vectors() {
        let (_database, document_repo, embedding_repo, db_path) = test_repos().await;
        seed_search_document(&document_repo).await;
        embedding_repo
            .index_all_embeddings("local-hash-384")
            .await
            .expect("index all embeddings");
        let original = embedding_repo
            .find_by_document_and_model("entry:entry-1:0", "local-hash-384")
            .await
            .expect("find original embedding")
            .expect("original embedding exists");

        document_repo
            .upsert_document(SearchDocumentInput {
                resource_type: "entry".to_string(),
                resource_id: "entry-1".to_string(),
                chunk_index: 0,
                title: "Entry one updated".to_string(),
                text: "Changed search document text".to_string(),
                source_updated_at: "2026-05-18T10:00:00Z".to_string(),
            })
            .await
            .expect("update search document");
        embedding_repo
            .refresh_existing_resource_embeddings("entry", "entry-1")
            .await
            .expect("refresh resource embeddings");
        let refreshed = embedding_repo
            .find_by_document_and_model("entry:entry-1:0", "local-hash-384")
            .await
            .expect("find refreshed embedding")
            .expect("refreshed embedding exists");

        assert_ne!(refreshed.text_hash, original.text_hash);

        cleanup_db(db_path);
    }
}
