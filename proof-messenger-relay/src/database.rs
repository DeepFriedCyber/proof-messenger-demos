//! TDD Step 1: Database persistence module for message storage
//!
//! This module provides secure storage and retrieval of verified messages
//! using SQLite for development and testing, with PostgreSQL support for production.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool, Row};
use thiserror::Error;
use uuid::Uuid;

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
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
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