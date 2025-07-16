-- Migration for proof revocation list
-- Creates the revoked_proofs table for storing revoked proof signatures

CREATE TABLE IF NOT EXISTS revoked_proofs (
    proof_signature TEXT PRIMARY KEY NOT NULL,
    revoked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason TEXT,
    revoked_by TEXT,
    expires_at DATETIME -- Optional expiration time for TTL
);

-- Index for efficient expiration-based cleanup
CREATE INDEX IF NOT EXISTS idx_revoked_proofs_expires_at 
ON revoked_proofs(expires_at);