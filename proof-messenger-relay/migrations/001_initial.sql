-- Initial migration for message storage
-- Creates the messages table for storing verified messages

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL DEFAULT 'default',
    sender TEXT NOT NULL,
    context TEXT NOT NULL,
    body TEXT NOT NULL,
    proof TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    verified BOOLEAN NOT NULL DEFAULT FALSE
);

-- Index for efficient group-based queries
CREATE INDEX IF NOT EXISTS idx_messages_group_id_created_at 
ON messages(group_id, created_at DESC);

-- Index for sender-based queries
CREATE INDEX IF NOT EXISTS idx_messages_sender 
ON messages(sender);

-- Index for timestamp-based cleanup operations
CREATE INDEX IF NOT EXISTS idx_messages_created_at 
ON messages(created_at);