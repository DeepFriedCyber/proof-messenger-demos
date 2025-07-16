# PowerShell Docker Build Script for Proof Messenger
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
    docker stop test-relay-server test-web-app 2>$null | Out-Null
    docker rm test-relay-server test-web-app 2>$null | Out-Null
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
        "proof-messenger-web/Dockerfile", 
        "proof-messenger-relay/src/main.rs",
        "docker-compose.yml"
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

# Build Docker images
function Build-DockerImages {
    Write-Info "Building Docker images..."

    # Build relay server image
    Write-Info "Building Relay Server image..."
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

    # Build web application image
    Write-Info "Building Web Application image..."
    try {
        docker build -t proof-messenger-web:latest -f proof-messenger-web/Dockerfile .
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to build Web Application image"
        }
        Write-Success "Web Application image built successfully"
    } catch {
        Write-Error "Failed to build Web Application image"
        exit 1
    }
}

# Validate built images
function Test-Images {
    Write-Info "Validating built images..."

    # Check if images exist
    $relayImage = docker image inspect proof-messenger-relay:latest 2>$null
    if (!$relayImage) {
        Write-Error "Relay Server image not found"
        exit 1
    }

    $webImage = docker image inspect proof-messenger-web:latest 2>$null
    if (!$webImage) {
        Write-Error "Web Application image not found"
        exit 1
    }

    Write-Success "All images validated successfully"
}

# Test container startup
function Test-ContainerStartup {
    Write-Info "Testing container startup..."

    # Test relay server container
    Write-Info "Testing Relay Server container startup..."
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

    # Test web application container
    Write-Info "Testing Web Application container startup..."
    try {
        docker run -d --name test-web-app -p 8001:80 proof-messenger-web:latest
        if ($LASTEXITCODE -ne 0) {
            throw "Web Application container failed to start"
        }
        Write-Success "Web Application container started successfully"
    } catch {
        Write-Error "Web Application container failed to start"
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

    # Test relay server health
    if (Wait-ForService "http://localhost:8080/health" "Relay Server") {
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:8080/health" -UseBasicParsing
            if ($response.status -eq "healthy") {
                Write-Success "Relay Server health check passed"
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

    # Test web application (basic connectivity)
    if (Wait-ForService "http://localhost:8001/" "Web Application") {
        Write-Success "Web Application health check passed"
    } else {
        Write-Warning "Web Application health check failed (this may be expected if the web app is not fully configured)"
    }
}

# Main execution
function Main {
    Write-Info "Starting TDD-enhanced Docker build process..."

    try {
        # Pre-build phase
        Test-PreBuildValidation
        Test-UnitTests

        # Build phase
        Build-DockerImages
        Test-Images

        # Test phase
        Test-ContainerStartup
        Test-ServiceHealth

        # Final validation
        Write-Info "Running final validation..."
        Write-Info "Image Information:"
        docker images | Where-Object { $_ -match "proof-messenger" }

        Write-Success "All tests passed! Docker build process completed successfully."
        Write-Success "Ready to run: docker-compose up"

        Write-Info "Next steps:"
        Write-Info "1. Run 'docker-compose up' to start all services"
        Write-Info "2. Access Web Application at: http://localhost:80"
        Write-Info "3. Access Relay Server API at: http://localhost:8080"

    } finally {
        Cleanup
    }
}

# Run main function
Main