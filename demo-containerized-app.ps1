# Complete Containerized Proof Messenger Protocol Demo
# This script demonstrates the fully containerized application

Write-Host "🐳 Proof Messenger Protocol - Containerized Demo" -ForegroundColor Cyan
Write-Host "=" * 60

Write-Host "`n📋 What this demo shows:" -ForegroundColor White
Write-Host "   ✅ Multi-stage Docker builds for optimal image sizes"
Write-Host "   ✅ Docker Compose orchestration with health checks"
Write-Host "   ✅ Container networking and service discovery"
Write-Host "   ✅ Production-ready configuration"
Write-Host "   ✅ Automated testing and validation"

Write-Host "`n🚀 Starting containerized application..." -ForegroundColor Yellow
docker-compose -f docker-compose.test.yml up -d

Write-Host "`n⏳ Waiting for services to be healthy..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

Write-Host "`n🧪 Running validation tests..." -ForegroundColor Yellow
.\test-containers.ps1

Write-Host "`n📊 Container Information:" -ForegroundColor White
Write-Host "   Relay Server Image Size:" -ForegroundColor Gray
docker images proof-messenger-relay:test --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

Write-Host "`n   Web Application Image Size:" -ForegroundColor Gray
docker images proof-messenger-web:test --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

Write-Host "`n🔧 Management Commands:" -ForegroundColor White
Write-Host "   View logs:     docker-compose -f docker-compose.test.yml logs -f" -ForegroundColor Gray
Write-Host "   Stop services: docker-compose -f docker-compose.test.yml down" -ForegroundColor Gray
Write-Host "   Restart:       docker-compose -f docker-compose.test.yml restart" -ForegroundColor Gray

Write-Host "`n🌐 Access the application:" -ForegroundColor Green
Write-Host "   Web App:    http://localhost" -ForegroundColor Cyan
Write-Host "   API Server: http://localhost:8080" -ForegroundColor Cyan
Write-Host "   Health:     http://localhost:8080/health" -ForegroundColor Cyan

Write-Host "`n🎉 Demo complete! The application is now running in containers." -ForegroundColor Green
Write-Host "   Press Ctrl+C to stop the demo and clean up containers." -ForegroundColor Yellow

# Keep the demo running until user interrupts
try {
    while ($true) {
        Start-Sleep -Seconds 5
        # Check if containers are still running
        $status = docker-compose -f docker-compose.test.yml ps --services --filter "status=running"
        if (-not $status) {
            Write-Host "`n❌ Containers stopped unexpectedly!" -ForegroundColor Red
            break
        }
    }
} catch {
    Write-Host "`n🛑 Demo interrupted by user." -ForegroundColor Yellow
} finally {
    Write-Host "`n🧹 Cleaning up containers..." -ForegroundColor Yellow
    docker-compose -f docker-compose.test.yml down
    Write-Host "✅ Cleanup complete!" -ForegroundColor Green
}