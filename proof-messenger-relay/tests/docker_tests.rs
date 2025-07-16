// Docker-specific integration tests
// These tests are separated because they require Docker to be running
// and may take longer to execute

// Import the Docker integration tests module
mod integration {
    pub mod docker_integration_tests;
}

// This file is intentionally kept minimal to allow running Docker tests separately
// Run with: cargo test --test docker_tests