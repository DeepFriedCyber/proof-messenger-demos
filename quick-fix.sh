#!/bin/bash
# quick-fix.sh - Apply all the fixes mentioned in the handoff document
set -e
echo "🔧 Applying fixes from handoff document..."
# Fix 1: Database path in main.rs
echo "📁 Fixing database path in main.rs..."
if grep -q "sqlite:messages.db" proof-messenger-relay/src/main.rs; then
    sed -i 's|sqlite:messages\.db|sqlite:/app/db/messages.db|g' proof-messenger-relay/src/main.rs
    echo "✅ Database path updated"
else
    echo "ℹ️  Database path already correct or not found"
fi
# Fix 2: Dockerfile permissions
echo "🔐 Checking Dockerfile permissions..."
if ! grep -q "chown proofmessenger:proofmessenger /app/db" proof-messenger-relay/Dockerfile; then
    echo "❌ Dockerfile permissions need manual fixing"
    echo "Please ensure your Dockerfile ends with:"
    echo "RUN mkdir -p /app/db"
    echo "RUN chown proofmessenger:proofmessenger /app/db"
    echo "USER proofmessenger"
    echo "CMD [\"./proof-messenger-relay\"]"
else
    echo "✅ Dockerfile permissions are correct"
fi
# Fix 3: Web app health check
echo "🌐 Fixing web app health check..."
if grep -q "curl --fail http://localhost:8001/health" docker-build.sh; then
    sed -i 's|curl --fail http://localhost:8001/health|curl --fail http://localhost:8001/index.html|g' docker-build.sh
    echo "✅ Web app health check updated"
else
    echo "ℹ️  Web app health check already correct or not found"
fi
echo ""
echo "🔍 Verification:"
echo "1. Database path check:"
if grep -q "sqlite:/app/db/messages.db" proof-messenger-relay/src/main.rs; then
    echo "   ✅ Database path is correct"
else
    echo "   ❌ Database path still needs fixing"
fi
echo "2. Dockerfile permissions check:"
if grep -q "chown proofmessenger:proofmessenger /app/db" proof-messenger-relay/Dockerfile; then
    echo "   ✅ Dockerfile permissions are correct"
else
    echo "   ❌ Dockerfile permissions still need fixing"
fi
echo "3. Web app health check:"
if grep -q "index.html" docker-build.sh; then
    echo "   ✅ Web app health check is correct"
else
    echo "   ❌ Web app health check still needs fixing"
fi
echo ""
echo "🚀 Ready to run: ./docker-build.sh"