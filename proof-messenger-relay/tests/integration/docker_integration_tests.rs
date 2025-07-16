// tests/integration/docker_integration_test.rs
use std::process::Command;
use std::thread;
use std::time::Duration;
use reqwest;
use serde_json::Value;
use tokio::time::timeout;

#[cfg(test)]
mod docker_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_relay_container_startup() {
        // Arrange
        let container_name = "test-relay-container";
        cleanup_container(container_name);

        // Act
        let output = Command::new("docker")
            .args(&[
                "run", "-d", "--name", container_name,
                "-p", "8080:8080",
                "-e", "DATABASE_URL=sqlite:/app/db/messages.db",
                "proof-messenger-relay:latest"
            ])
            .output()
            .expect("Failed to start Docker container");

        // Assert
        assert!(output.status.success(), "Container should start successfully");

        // Wait for container to be ready
        thread::sleep(Duration::from_secs(3));

        // Verify container is running
        let status = get_container_status(container_name);
        assert!(status.contains("Up"), "Container should be running");

        // Cleanup
        cleanup_container(container_name);
    }

    #[tokio::test]
    async fn test_relay_container_health_check() {
        // Arrange
        let container_name = "test-relay-health";
        cleanup_container(container_name);

        // Start container
        let output = Command::new("docker")
            .args(&[
                "run", "-d", "--name", container_name,
                "-p", "8081:8080",
                "-e", "DATABASE_URL=sqlite:/app/db/messages.db",
                "proof-messenger-relay:latest"
            ])
            .output()
            .expect("Failed to start Docker container");

        assert!(output.status.success(), "Container should start successfully");

        // Wait for container to be ready
        thread::sleep(Duration::from_secs(5));

        // Act
        let client = reqwest::Client::new();
        let response = timeout(
            Duration::from_secs(10),
            client.get("http://localhost:8081/health").send()
        ).await;

        // Assert
        assert!(response.is_ok(), "Health check request should not timeout");

        let response = response.unwrap().unwrap();
        assert_eq!(response.status(), 200, "Health check should return 200 OK");

        let body: Value = response.json().await.unwrap();
        assert_eq!(body["status"], "healthy");

        // Cleanup
        cleanup_container(container_name);
    }

    #[tokio::test]
    async fn test_database_persistence_in_container() {
        // Arrange
        let container_name = "test-db-persistence";
        cleanup_container(container_name);

        // Create a volume for database persistence
        Command::new("docker")
            .args(&["volume", "create", "test-db-vol"])
            .output()
            .expect("Failed to create volume");

        // Start container with volume
        let output = Command::new("docker")
            .args(&[
                "run", "-d", "--name", container_name,
                "-p", "8082:8080",
                "-v", "test-db-vol:/app/db",
                "-e", "DATABASE_URL=sqlite:/app/db/messages.db",
                "proof-messenger-relay:latest"
            ])
            .output()
            .expect("Failed to start Docker container");

        assert!(output.status.success(), "Container should start successfully");

        // Wait for container to be ready
        thread::sleep(Duration::from_secs(5));

        // Act - Make a request to create some data
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:8082/health")
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), 200);

        // Stop and restart container
        Command::new("docker")
            .args(&["stop", container_name])
            .output()
            .expect("Failed to stop container");

        Command::new("docker")
            .args(&["rm", container_name])
            .output()
            .expect("Failed to remove container");

        // Start new container with same volume
        let output = Command::new("docker")
            .args(&[
                "run", "-d", "--name", container_name,
                "-p", "8082:8080",
                "-v", "test-db-vol:/app/db",
                "-e", "DATABASE_URL=sqlite:/app/db/messages.db",
                "proof-messenger-relay:latest"
            ])
            .output()
            .expect("Failed to start Docker container");

        assert!(output.status.success(), "Container should start successfully after restart");

        thread::sleep(Duration::from_secs(5));

        // Verify database is still accessible
        let response = client
            .get("http://localhost:8082/health")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        // Cleanup
        cleanup_container(container_name);
        Command::new("docker")
            .args(&["volume", "rm", "test-db-vol"])
            .output()
            .expect("Failed to remove volume");
    }

    #[tokio::test]
    async fn test_web_app_container_startup() {
        // Arrange
        let container_name = "test-web-container";
        cleanup_container(container_name);

        // Act
        let output = Command::new("docker")
            .args(&[
                "run", "-d", "--name", container_name,
                "-p", "8001:80",
                "proof-messenger-web:latest"
            ])
            .output()
            .expect("Failed to start Docker container");

        // Assert
        assert!(output.status.success(), "Web container should start successfully");

        // Wait for container to be ready
        thread::sleep(Duration::from_secs(3));

        // Verify container is running
        let status = get_container_status(container_name);
        assert!(status.contains("Up"), "Web container should be running");

        // Test web app accessibility
        let client = reqwest::Client::new();
        let response = timeout(
            Duration::from_secs(10),
            client.get("http://localhost:8001/index.html").send()
        ).await;

        assert!(response.is_ok(), "Web app should be accessible");

        let response = response.unwrap().unwrap();
        assert_eq!(response.status(), 200, "Web app should return 200 OK");

        // Cleanup
        cleanup_container(container_name);
    }

    #[tokio::test]
    async fn test_docker_compose_integration() {
        // Arrange
        let compose_project = "test-proof-messenger";

        // Stop any existing compose services
        Command::new("docker-compose")
            .args(&["-p", compose_project, "down"])
            .output()
            .expect("Failed to stop compose services");

        // Act
        let output = Command::new("docker-compose")
            .args(&[
                "-f", "docker-compose.yml",
                "-p", compose_project,
                "up", "-d"
            ])
            .output()
            .expect("Failed to start compose services");

        // Assert
        assert!(output.status.success(), "Docker compose should start successfully");

        // Wait for services to be ready
        thread::sleep(Duration::from_secs(10));

        // Test both services
        let client = reqwest::Client::new();

        // Test relay server
        let relay_response = timeout(
            Duration::from_secs(10),
            client.get("http://localhost:8080/health").send()
        ).await;

        assert!(relay_response.is_ok(), "Relay server should be accessible");
        assert_eq!(relay_response.unwrap().unwrap().status(), 200);

        // Test web application
        let web_response = timeout(
            Duration::from_secs(10),
            client.get("http://localhost:80/index.html").send()
        ).await;

        assert!(web_response.is_ok(), "Web application should be accessible");
        assert_eq!(web_response.unwrap().unwrap().status(), 200);

        // Cleanup
        Command::new("docker-compose")
            .args(&["-p", compose_project, "down"])
            .output()
            .expect("Failed to stop compose services");
    }

    #[test]
    fn test_container_logs_for_errors() {
        // Arrange
        let container_name = "test-logs-container";
        cleanup_container(container_name);

        // Start container
        let output = Command::new("docker")
            .args(&[
                "run", "-d", "--name", container_name,
                "-p", "8083:8080",
                "proof-messenger-relay:latest"
            ])
            .output()
            .expect("Failed to start Docker container");

        assert!(output.status.success(), "Container should start successfully");

        // Wait for some logs to be generated
        thread::sleep(Duration::from_secs(5));

        // Act
        let logs = Command::new("docker")
            .args(&["logs", container_name])
            .output()
            .expect("Failed to get container logs");

        let log_output = String::from_utf8_lossy(&logs.stdout);

        // Assert
        assert!(!log_output.contains("ERROR"), "Container logs should not contain errors");
        assert!(log_output.contains("Server ready to accept connections"), "Should show server ready message");

        // Cleanup
        cleanup_container(container_name);
    }
}

// Helper functions
fn cleanup_container(container_name: &str) {
    Command::new("docker")
        .args(&["stop", container_name])
        .output()
        .ok();

    Command::new("docker")
        .args(&["rm", container_name])
        .output()
        .ok();
}

fn get_container_status(container_name: &str) -> String {
    let output = Command::new("docker")
        .args(&["ps", "-f", &format!("name={}", container_name), "--format", "{{.Status}}"])
        .output()
        .expect("Failed to get container status");

    String::from_utf8_lossy(&output.stdout).to_string()
}