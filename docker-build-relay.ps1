# PowerShell Docker Build Script for Proof Messenger Relay Server Only
# Enhanced with TDD testing for Windows environments

param(
    [switch]$SkipTests = $false,
    [switch]$Verbose = $false
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Cyan"

function Write-Info($message) {
    Write-Host "[INFO] $message" -ForegroundColor $Blue
}

function Write-Success($message) {
    Write-Host "[SUCCESS] $message" -ForegroundColor $Green
}

function Write-Error($message) {
    Write-Host "[ERROR] $message" -ForegroundColor $Red
}

function Write-Warning($message) {
    Write-Host "[WARNING] $message" -ForegroundColor $Yellow
}

# Cleanup function
function Cleanup {
    Write-Info "Cleaning up test containers..."
    docker stop test-relay-server 2>$null | Out-Null
    docker rm test-relay-server 2>$null | Out-Null
}

# Pre-build validation
function Test-PreBuildValidation {
    Write-Info "Running pre-build validation..."

    # Check if Docker is running
    try {
        docker info | Out-Null
    } catch {
        Write-Error "Docker is not running. Please start Docker and try again."
        exit 1
    }

    # Check required files exist
    $requiredFiles = @(
        "proof-messenger-relay/Dockerfile",
        "proof-messenger-relay/src/main.rs"
    )

    foreach ($file in $requiredFiles) {
        if (!(Test-Path $file)) {
            Write-Error "Required file not found: $file"
            exit 1
        }
    }

    # Validate database path configuration
    $mainRsContent = Get-Content "proof-messenger-relay/src/main.rs" -Raw
    if ($mainRsContent -notmatch "sqlite:/app/db/messages.db") {
        Write-Error "Database path not configured correctly in main.rs"
        Write-Error "Expected: sqlite:/app/db/messages.db"
        exit 1
    }

    # Validate Dockerfile permissions setup
    $dockerfileContent = Get-Content "proof-messenger-relay/Dockerfile" -Raw
    if ($dockerfileContent -notmatch "RUN chown proofmessenger:proofmessenger /app/db") {
        Write-Error "Database permissions not configured correctly in Dockerfile"
        exit 1
    }

    Write-Success "Pre-build validation passed"
}

# Run unit tests
function Test-UnitTests {
    if ($SkipTests) {
        Write-Warning "Skipping unit tests"
        return
    }

    Write-Info "Running unit tests..."
    
    try {
        cargo test --workspace --lib
        if ($LASTEXITCODE -ne 0) {
            throw "Unit tests failed"
        }
        Write-Success "Unit tests passed"
    } catch {
        Write-Error "Unit tests failed"
        exit 1
    }
}

# Build Docker image
function Build-RelayImage {
    Write-Info "Building Relay Server Docker image..."

    try {
        docker build -t proof-messenger-relay:latest -f proof-messenger-relay/Dockerfile .
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to build Relay Server image"
        }
        Write-Success "Relay Server image built successfully"
    } catch {
        Write-Error "Failed to build Relay Server image"
        exit 1
    }
}

# Validate built image
function Test-Image {
    Write-Info "Validating built image..."

    # Check if image exists
    $relayImage = docker image inspect proof-messenger-relay:latest 2>$null
    if (!$relayImage) {
        Write-Error "Relay Server image not found"
        exit 1
    }

    Write-Success "Image validated successfully"
}

# Test container startup
function Test-ContainerStartup {
    Write-Info "Testing container startup..."

    try {
        docker run -d --name test-relay-server -p 8080:8080 -e DATABASE_URL=sqlite:/app/db/messages.db proof-messenger-relay:latest
        if ($LASTEXITCODE -ne 0) {
            throw "Relay Server container failed to start"
        }
        Write-Success "Relay Server container started successfully"
    } catch {
        Write-Error "Relay Server container failed to start"
        exit 1
    }
}

# Wait for service to be ready
function Wait-ForService($url, $serviceName, $maxAttempts = 5) {
    Write-Info "Waiting for $serviceName to be ready..."
    
    for ($attempt = 1; $attempt -le $maxAttempts; $attempt++) {
        try {
            $response = Invoke-WebRequest -Uri $url -TimeoutSec 5 -UseBasicParsing
            if ($response.StatusCode -eq 200) {
                Write-Success "$serviceName is ready"
                return $true
            }
        } catch {
            # Service not ready yet
        }
        
        Write-Info "Attempt $attempt/$maxAttempts`: $serviceName not ready yet, waiting..."
        Start-Sleep -Seconds 2
    }
    
    Write-Error "$serviceName failed to become ready after $maxAttempts attempts"
    return $false
}

# Test service health
function Test-ServiceHealth {
    Write-Info "Testing service health..."

    if (Wait-ForService "http://localhost:8080/health" "Relay Server") {
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:8080/health" -UseBasicParsing
            if ($response.status -eq "healthy") {
                Write-Success "Relay Server health check passed"
                Write-Info "Health response: $($response | ConvertTo-Json -Compress)"
            } else {
                Write-Error "Relay Server health check failed - unhealthy response"
                Write-Host "Response: $($response | ConvertTo-Json)"
                exit 1
            }
        } catch {
            Write-Error "Relay Server health check failed - $_"
            exit 1
        }
    } else {
        Write-Error "Relay Server health check failed - service not accessible"
        exit 1
    }
}

# Main execution
function Main {
    Write-Info "Starting Docker build process for Relay Server..."

    try {
        # Pre-build phase
        Test-PreBuildValidation
        Test-UnitTests

        # Build phase
        Build-RelayImage
        Test-Image

        # Test phase
        Test-ContainerStartup
        Test-ServiceHealth

        # Final validation
        Write-Info "Running final validation..."
        Write-Info "Image Information:"
        docker images | Where-Object { $_ -match "proof-messenger-relay" }

        Write-Success "All tests passed! Relay Server Docker build completed successfully."
        Write-Success "Ready to run: docker run -p 8080:8080 proof-messenger-relay:latest"

        Write-Info "Next steps:"
        Write-Info "1. Run 'docker run -d -p 8080:8080 -e DATABASE_URL=sqlite:/app/db/messages.db proof-messenger-relay:latest' to start the relay server"
        Write-Info "2. Access Relay Server API at: http://localhost:8080"
        Write-Info "3. Health check: http://localhost:8080/health"

    } finally {
        Cleanup
    }
}

# Run main function
Main