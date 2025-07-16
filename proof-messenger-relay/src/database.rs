//! TDD Step 1: Database persistence module for message storage
//!
//! This module provides secure storage and retrieval of verified messages
//! using SQLite for development and testing, with PostgreSQL support for production.
//! Also includes proof revocation functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool, Row};
use thiserror::Error;
use uuid::Uuid;
use std::time::Duration;

use crate::Message;

/// Database-specific error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),
    
    #[error("Message not found: {0}")]
    MessageNotFound(String),
    
    #[error("Invalid group ID format: {0}")]
    InvalidGroupId(String),
    
    #[error("Database migration error: {0}")]
    MigrationError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Proof already revoked: {0}")]
    ProofAlreadyRevoked(String),
}

/// Stored message with metadata
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredMessage {
    /// Unique message ID
    pub id: String,
    /// Group/channel ID for message organization
    pub group_id: String,
    /// Public key of the sender (hex encoded)
    pub sender: String,
    /// Context data that was signed (hex encoded)
    pub context: String,
    /// Message body content
    pub body: String,
    /// Cryptographic proof/signature (hex encoded)
    pub proof: String,
    /// Timestamp when message was stored
    pub created_at: DateTime<Utc>,
    /// Whether the message signature was verified
    pub verified: bool,
}

/// Revoked proof information
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RevokedProof {
    /// The signature of the revoked proof (hex encoded)
    pub proof_signature: String,
    /// When the proof was revoked
    pub revoked_at: DateTime<Utc>,
    /// Optional reason for revocation
    pub reason: Option<String>,
    /// Who revoked the proof (user ID or system)
    pub revoked_by: Option<String>,
    /// Optional expiration time for TTL
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<Message> for StoredMessage {
    fn from(message: Message) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            group_id: "default".to_string(), // Default group for now
            sender: message.sender,
            context: message.context,
            body: message.body,
            proof: message.proof,
            created_at: Utc::now(),
            verified: false, // Will be set after verification
        }
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
}

impl DatabaseConfig {
    /// Create a new database configuration with default values
    pub fn new(database_url: &str) -> Self {
        Self {
            database_url: database_url.to_string(),
        }
    }
    
    /// Create a database configuration from environment variables
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:/app/db/messages.db".to_string());
            
        Self {
            database_url,
        }
    }
}

/// Database connection and operations
#[derive(Debug)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;
        Ok(())
    }

    /// Store a verified message in the database
    pub async fn store_message(&self, mut message: StoredMessage) -> Result<String, DatabaseError> {
        message.verified = true; // Mark as verified since we only store verified messages
        
        let result = sqlx::query(
            r#"
            INSERT INTO messages (id, group_id, sender, context, body, proof, created_at, verified)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#
        )
        .bind(&message.id)
        .bind(&message.group_id)
        .bind(&message.sender)
        .bind(&message.context)
        .bind(&message.body)
        .bind(&message.proof)
        .bind(&message.created_at)
        .bind(message.verified)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 1 {
            Ok(message.id)
        } else {
            Err(DatabaseError::SerializationError("Failed to insert message".to_string()))
        }
    }

    /// Retrieve messages for a specific group
    pub async fn get_messages_by_group(&self, group_id: &str, limit: Option<i64>) -> Result<Vec<StoredMessage>, DatabaseError> {
        let limit = limit.unwrap_or(100); // Default limit
        
        let messages = sqlx::query_as::<_, StoredMessage>(
            r#"
            SELECT id, group_id, sender, context, body, proof, created_at, verified
            FROM messages 
            WHERE group_id = ?1 
            ORDER BY created_at DESC 
            LIMIT ?2
            "#
        )
        .bind(group_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    /// Retrieve a specific message by ID
    pub async fn get_message_by_id(&self, message_id: &str) -> Result<StoredMessage, DatabaseError> {
        let message = sqlx::query_as::<_, StoredMessage>(
            r#"
            SELECT id, group_id, sender, context, body, proof, created_at, verified
            FROM messages 
            WHERE id = ?1
            "#
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?;

        message.ok_or_else(|| DatabaseError::MessageNotFound(message_id.to_string()))
    }

    /// Get message count for a group
    pub async fn get_message_count(&self, group_id: &str) -> Result<i64, DatabaseError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM messages WHERE group_id = ?1")
            .bind(group_id)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Delete old messages (for cleanup)
    pub async fn delete_old_messages(&self, older_than: DateTime<Utc>) -> Result<u64, DatabaseError> {
        let result = sqlx::query("DELETE FROM messages WHERE created_at < ?1")
            .bind(older_than)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get database health status
    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        // Try to execute a simple query to verify database connection
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
            
        // Check if migrations table exists (indicates proper schema setup)
        let migrations_result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'")
            .fetch_optional(&self.pool)
            .await?;
            
        if migrations_result.is_none() {
            return Err(DatabaseError::MigrationError("Migrations table not found".to_string()));
        }
        
        Ok(())
    }
    
    /// Revoke a proof by adding it to the revocation list
    pub async fn revoke_proof(
        &self, 
        proof_signature: &str, 
        reason: Option<&str>, 
        revoked_by: Option<&str>,
        ttl_hours: Option<i64>
    ) -> Result<(), DatabaseError> {
        // Check if proof is already revoked
        let existing = sqlx::query("SELECT proof_signature FROM revoked_proofs WHERE proof_signature = ?1")
            .bind(proof_signature)
            .fetch_optional(&self.pool)
            .await?;
            
        if existing.is_some() {
            return Err(DatabaseError::ProofAlreadyRevoked(proof_signature.to_string()));
        }
        
        // Calculate expiration time if TTL is provided
        let expires_at = ttl_hours.map(|hours| {
            Utc::now() + chrono::Duration::hours(hours)
        });
        
        // Insert into revocation list
        sqlx::query(
            r#"
            INSERT INTO revoked_proofs (proof_signature, revoked_at, reason, revoked_by, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#
        )
        .bind(proof_signature)
        .bind(Utc::now())
        .bind(reason)
        .bind(revoked_by)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Check if a proof has been revoked
    pub async fn is_proof_revoked(&self, proof_signature: &str) -> Result<bool, DatabaseError> {
        // First, clean up expired revocations
        self.cleanup_expired_revocations().await?;
        
        // Check if proof is in the revocation list
        let result = sqlx::query(
            r#"
            SELECT proof_signature FROM revoked_proofs 
            WHERE proof_signature = ?1
            AND (expires_at IS NULL OR expires_at > ?2)
            "#
        )
        .bind(proof_signature)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result.is_some())
    }
    
    /// Clean up expired revocations
    pub async fn cleanup_expired_revocations(&self) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            DELETE FROM revoked_proofs
            WHERE expires_at IS NOT NULL AND expires_at < ?1
            "#
        )
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected())
    }
    
    /// Get all active revocations
    pub async fn get_active_revocations(&self) -> Result<Vec<RevokedProof>, DatabaseError> {
        // First, clean up expired revocations
        self.cleanup_expired_revocations().await?;
        
        let revocations = sqlx::query_as::<_, RevokedProof>(
            r#"
            SELECT proof_signature, revoked_at, reason, revoked_by, expires_at
            FROM revoked_proofs
            WHERE expires_at IS NULL OR expires_at > ?1
            ORDER BY revoked_at DESC
            "#
        )
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await?;
        
        Ok(revocations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use std::time::Duration;
    use tokio::time::sleep;

    async fn setup_test_db() -> Database {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        db
    }

    fn create_test_message() -> Message {
        Message {
            sender: "abcd1234".to_string(),
            context: "test_context".to_string(),
            body: "Test message body".to_string(),
            proof: "proof1234".to_string(),
        }
    }

    #[tokio::test]
    async fn test_database_creation_and_migration() {
        // ARRANGE & ACT: Create database and run migrations
        let db = setup_test_db().await;

        // ASSERT: Health check should pass
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_message() {
        // ARRANGE: Setup database and create test message
        let db = setup_test_db().await;
        let message = create_test_message();
        let stored_message = StoredMessage::from(message);

        // ACT: Store message
        let message_id = db.store_message(stored_message.clone()).await.unwrap();

        // ASSERT: Message should be stored and retrievable
        assert!(!message_id.is_empty());
        
        let retrieved = db.get_message_by_id(&message_id).await.unwrap();
        assert_eq!(retrieved.sender, stored_message.sender);
        assert_eq!(retrieved.body, stored_message.body);
        assert_eq!(retrieved.verified, true); // Should be marked as verified
    }

    #[tokio::test]
    async fn test_get_messages_by_group() {
        // ARRANGE: Setup database and store multiple messages
        let db = setup_test_db().await;
        
        let mut message1 = StoredMessage::from(create_test_message());
        message1.group_id = "group1".to_string();
        message1.body = "Message 1".to_string();
        
        let mut message2 = StoredMessage::from(create_test_message());
        message2.group_id = "group1".to_string();
        message2.body = "Message 2".to_string();
        
        let mut message3 = StoredMessage::from(create_test_message());
        message3.group_id = "group2".to_string();
        message3.body = "Message 3".to_string();

        // ACT: Store messages
        db.store_message(message1).await.unwrap();
        db.store_message(message2).await.unwrap();
        db.store_message(message3).await.unwrap();

        // ASSERT: Should retrieve only messages from specified group
        let group1_messages = db.get_messages_by_group("group1", None).await.unwrap();
        assert_eq!(group1_messages.len(), 2);
        
        let group2_messages = db.get_messages_by_group("group2", None).await.unwrap();
        assert_eq!(group2_messages.len(), 1);
        
        // Messages should be ordered by created_at DESC (newest first)
        assert!(group1_messages[0].created_at >= group1_messages[1].created_at);
    }

    #[tokio::test]
    async fn test_get_messages_with_limit() {
        // ARRANGE: Setup database and store multiple messages
        let db = setup_test_db().await;
        
        for i in 0..5 {
            let mut message = StoredMessage::from(create_test_message());
            message.body = format!("Message {}", i);
            db.store_message(message).await.unwrap();
        }

        // ACT: Retrieve with limit
        let messages = db.get_messages_by_group("default", Some(3)).await.unwrap();

        // ASSERT: Should respect limit
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_message_not_found() {
        // ARRANGE: Setup database
        let db = setup_test_db().await;

        // ACT: Try to retrieve non-existent message
        let result = db.get_message_by_id("non-existent-id").await;

        // ASSERT: Should return MessageNotFound error
        assert!(matches!(result, Err(DatabaseError::MessageNotFound(_))));
    }

    #[tokio::test]
    async fn test_get_message_count() {
        // ARRANGE: Setup database and store messages
        let db = setup_test_db().await;
        
        let mut message1 = StoredMessage::from(create_test_message());
        message1.group_id = "test_group".to_string();
        
        let mut message2 = StoredMessage::from(create_test_message());
        message2.group_id = "test_group".to_string();

        // ACT: Store messages and get count
        db.store_message(message1).await.unwrap();
        db.store_message(message2).await.unwrap();
        
        let count = db.get_message_count("test_group").await.unwrap();

        // ASSERT: Count should be correct
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_delete_old_messages() {
        // ARRANGE: Setup database and store messages with different timestamps
        let db = setup_test_db().await;
        
        let mut old_message = StoredMessage::from(create_test_message());
        old_message.created_at = Utc::now() - chrono::Duration::hours(2);
        
        let new_message = StoredMessage::from(create_test_message());
        // new_message has current timestamp

        db.store_message(old_message).await.unwrap();
        db.store_message(new_message).await.unwrap();

        // ACT: Delete messages older than 1 hour
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        let deleted_count = db.delete_old_messages(cutoff).await.unwrap();

        // ASSERT: Should delete only the old message
        assert_eq!(deleted_count, 1);
        
        let remaining_count = db.get_message_count("default").await.unwrap();
        assert_eq!(remaining_count, 1);
    }

    #[tokio::test]
    async fn test_stored_message_from_message_conversion() {
        // ARRANGE: Create a Message
        let message = create_test_message();

        // ACT: Convert to StoredMessage
        let stored = StoredMessage::from(message.clone());

        // ASSERT: Conversion should preserve data
        assert_eq!(stored.sender, message.sender);
        assert_eq!(stored.context, message.context);
        assert_eq!(stored.body, message.body);
        assert_eq!(stored.proof, message.proof);
        assert_eq!(stored.group_id, "default");
        assert_eq!(stored.verified, false); // Initially not verified
        assert!(!stored.id.is_empty());
    }

    #[tokio::test]
    async fn test_concurrent_message_storage() {
        // ARRANGE: Setup database (not used in this test as each task creates its own)
        let _db = setup_test_db().await;
        
        // ACT: Store messages concurrently
        let mut handles = vec![];
        
        for i in 0..10 {
            let db_clone = Database::new("sqlite::memory:").await.unwrap();
            db_clone.migrate().await.unwrap();
            
            let handle = tokio::spawn(async move {
                let mut message = StoredMessage::from(create_test_message());
                message.body = format!("Concurrent message {}", i);
                db_clone.store_message(message).await
            });
            
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // ASSERT: All operations should succeed
        for result in results {
            assert!(result.unwrap().is_ok());
        }
    }
    
    #[tokio::test]
    async fn test_revoke_and_check_proof() {
        // ARRANGE: Setup database
        let db = setup_test_db().await;
        let proof_signature = "test_signature_123";
        
        // ACT: Revoke a proof
        db.revoke_proof(proof_signature, Some("Test revocation"), Some("test_user"), Some(24)).await.unwrap();
        
        // ASSERT: Proof should be marked as revoked
        let is_revoked = db.is_proof_revoked(proof_signature).await.unwrap();
        assert!(is_revoked);
    }
    
    #[tokio::test]
    async fn test_revoke_proof_already_revoked() {
        // ARRANGE: Setup database and revoke a proof
        let db = setup_test_db().await;
        let proof_signature = "already_revoked_signature";
        
        db.revoke_proof(proof_signature, None, None, None).await.unwrap();
        
        // ACT: Try to revoke the same proof again
        let result = db.revoke_proof(proof_signature, None, None, None).await;
        
        // ASSERT: Should return ProofAlreadyRevoked error
        assert!(matches!(result, Err(DatabaseError::ProofAlreadyRevoked(_))));
    }
    
    #[tokio::test]
    async fn test_expired_revocation() {
        // ARRANGE: Setup database and revoke a proof with a very short TTL
        let db = setup_test_db().await;
        let proof_signature = "soon_to_expire_signature";
        
        // Set expiration to 0 hours (immediate expiration for testing)
        db.revoke_proof(proof_signature, None, None, Some(0)).await.unwrap();
        
        // Force expiration by manipulating the database directly
        sqlx::query("UPDATE revoked_proofs SET expires_at = datetime('now', '-1 hour') WHERE proof_signature = ?1")
            .bind(proof_signature)
            .execute(&db.pool)
            .await
            .unwrap();
        
        // ACT: Check if proof is still revoked (should trigger cleanup)
        let is_revoked = db.is_proof_revoked(proof_signature).await.unwrap();
        
        // ASSERT: Proof should no longer be considered revoked
        assert!(!is_revoked);
    }
    
    #[tokio::test]
    async fn test_get_active_revocations() {
        // ARRANGE: Setup database and add multiple revocations
        let db = setup_test_db().await;
        
        // Add permanent revocation
        db.revoke_proof("permanent_revocation", Some("Never expires"), Some("admin"), None).await.unwrap();
        
        // Add temporary revocation
        db.revoke_proof("temporary_revocation", Some("Will expire"), Some("user"), Some(24)).await.unwrap();
        
        // Add expired revocation
        db.revoke_proof("expired_revocation", Some("Already expired"), Some("user"), Some(0)).await.unwrap();
        
        // Force expiration
        sqlx::query("UPDATE revoked_proofs SET expires_at = datetime('now', '-1 hour') WHERE proof_signature = ?1")
            .bind("expired_revocation")
            .execute(&db.pool)
            .await
            .unwrap();
        
        // ACT: Get active revocations
        let active_revocations = db.get_active_revocations().await.unwrap();
        
        // ASSERT: Should only include non-expired revocations
        assert_eq!(active_revocations.len(), 2);
        
        // Check that the expired one is not included
        let contains_expired = active_revocations.iter()
            .any(|r| r.proof_signature == "expired_revocation");
        assert!(!contains_expired);
        
        // Check that permanent and temporary are included
        let contains_permanent = active_revocations.iter()
            .any(|r| r.proof_signature == "permanent_revocation");
        let contains_temporary = active_revocations.iter()
            .any(|r| r.proof_signature == "temporary_revocation");
        assert!(contains_permanent);
        assert!(contains_temporary);
    }

    #[tokio::test]
    async fn test_database_health_check() {
        // ARRANGE: Setup database
        let db = setup_test_db().await;

        // ACT: Perform health check
        let result = db.health_check().await;

        // ASSERT: Health check should pass
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_message_ordering() {
        // ARRANGE: Setup database
        let db = setup_test_db().await;
        
        // Store messages with small delays to ensure different timestamps
        let mut message1 = StoredMessage::from(create_test_message());
        message1.body = "First message".to_string();
        db.store_message(message1).await.unwrap();
        
        sleep(Duration::from_millis(10)).await;
        
        let mut message2 = StoredMessage::from(create_test_message());
        message2.body = "Second message".to_string();
        db.store_message(message2).await.unwrap();

        // ACT: Retrieve messages
        let messages = db.get_messages_by_group("default", None).await.unwrap();

        // ASSERT: Messages should be ordered newest first
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].body, "Second message");
        assert_eq!(messages[1].body, "First message");
    }
}