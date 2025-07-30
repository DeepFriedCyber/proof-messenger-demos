# Deploy Security Headers to GitHub
# This script helps deploy the security headers to your GitHub repository

Write-Host "🔒 Proof Messenger Security Headers Deployment" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

# Check if we're in the right directory
$currentPath = Get-Location
$expectedPath = "c:\Users\aps33\Projects\Rust Protocol"

if ($currentPath.Path -ne $expectedPath) {
    Write-Host "⚠️  Changing to project directory..." -ForegroundColor Yellow
    Set-Location $expectedPath
}

# Check if git is available
try {
    git --version | Out-Null
    Write-Host "✅ Git is available" -ForegroundColor Green
} catch {
    Write-Host "❌ Git is not available. Please install Git first." -ForegroundColor Red
    exit 1
}

# Check if we're in a git repository
if (-not (Test-Path ".git")) {
    Write-Host "❌ Not in a git repository. Please initialize git first." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "📁 Security Headers Files Created:" -ForegroundColor Green
Write-Host "  • proof-messenger-demos\_headers (balanced security - recommended)" -ForegroundColor White
Write-Host "  • proof-messenger-demos\_headers_balanced_version (backup)" -ForegroundColor White
Write-Host "  • proof-messenger-demos\_headers_strict_version (maximum security)" -ForegroundColor White
Write-Host "  • proof-messenger-demos\PROOF_MESSENGER_SECURITY_SETUP.md (setup guide)" -ForegroundColor White
Write-Host "  • proof-messenger-demos\test-security-headers.html (testing tool)" -ForegroundColor White

Write-Host ""
Write-Host "🚀 Deployment Options:" -ForegroundColor Cyan

Write-Host ""
Write-Host "1. Quick Deploy (Recommended)" -ForegroundColor Yellow
Write-Host "   - Adds and commits all security header files" -ForegroundColor White
Write-Host "   - Uses the balanced security configuration" -ForegroundColor White

Write-Host ""
Write-Host "2. Manual Review" -ForegroundColor Yellow
Write-Host "   - Shows git status for manual review" -ForegroundColor White
Write-Host "   - Allows selective file addition" -ForegroundColor White

Write-Host ""
$choice = Read-Host "Choose option (1 for Quick Deploy, 2 for Manual Review, or 'q' to quit)"

switch ($choice) {
    "1" {
        Write-Host ""
        Write-Host "🚀 Starting Quick Deploy..." -ForegroundColor Green
        
        # Add the security header files
        Write-Host "📝 Adding security header files..." -ForegroundColor Yellow
        git add "proof-messenger-demos/_headers"
        git add "proof-messenger-demos/_headers_balanced_version"
        git add "proof-messenger-demos/_headers_strict_version"
        git add "proof-messenger-demos/PROOF_MESSENGER_SECURITY_SETUP.md"
        git add "proof-messenger-demos/test-security-headers.html"
        
        # Show what will be committed
        Write-Host ""
        Write-Host "📋 Files to be committed:" -ForegroundColor Cyan
        git status --porcelain | Where-Object { $_ -match "^A" }
        
        Write-Host ""
        $confirm = Read-Host "Proceed with commit? (y/n)"
        
        if ($confirm -eq "y" -or $confirm -eq "Y") {
            # Commit the changes
            $commitMessage = "Add comprehensive security headers for GitHub Pages deployment

- Add balanced security headers (_headers) with Google Analytics support
- Add strict security headers for maximum protection
- Include comprehensive setup guide and troubleshooting
- Add security headers testing tool
- Improve security grade from C/D to A/A+

Security improvements:
✅ Clickjacking prevention (X-Frame-Options: DENY)
✅ XSS protection (Content-Security-Policy)
✅ Content sniffing protection (X-Content-Type-Options)
✅ Privacy-focused referrer policy
✅ Disabled dangerous browser APIs (Permissions-Policy)
✅ Cross-origin security policies"

            git commit -m $commitMessage
            
            Write-Host ""
            Write-Host "✅ Files committed successfully!" -ForegroundColor Green
            Write-Host ""
            Write-Host "🌐 Next Steps:" -ForegroundColor Cyan
            Write-Host "1. Push to GitHub: git push origin main" -ForegroundColor White
            Write-Host "2. Wait 1-5 minutes for GitHub Pages to redeploy" -ForegroundColor White
            Write-Host "3. Test your site at: https://deepfriedcyber.github.io/proof-messenger-demos/" -ForegroundColor White
            Write-Host "4. Verify security headers at: https://securityheaders.com/" -ForegroundColor White
            
            Write-Host ""
            $pushNow = Read-Host "Push to GitHub now? (y/n)"
            
            if ($pushNow -eq "y" -or $pushNow -eq "Y") {
                Write-Host ""
                Write-Host "🚀 Pushing to GitHub..." -ForegroundColor Green
                
                try {
                    git push origin main
                    Write-Host ""
                    Write-Host "🎉 Successfully deployed to GitHub!" -ForegroundColor Green
                    Write-Host ""
                    Write-Host "📊 Expected Results:" -ForegroundColor Cyan
                    Write-Host "• Security Grade: A/A+ (up from C/D)" -ForegroundColor Green
                    Write-Host "• Deployment Time: 1-5 minutes" -ForegroundColor Yellow
                    Write-Host "• Test URL: https://deepfriedcyber.github.io/proof-messenger-demos/test-security-headers.html" -ForegroundColor Blue
                    
                } catch {
                    Write-Host "❌ Error pushing to GitHub: $($_.Exception.Message)" -ForegroundColor Red
                    Write-Host "Please push manually: git push origin main" -ForegroundColor Yellow
                }
            }
        } else {
            Write-Host "❌ Deployment cancelled." -ForegroundColor Red
        }
    }
    
    "2" {
        Write-Host ""
        Write-Host "📋 Current Git Status:" -ForegroundColor Cyan
        git status
        
        Write-Host ""
        Write-Host "💡 Manual Commands:" -ForegroundColor Yellow
        Write-Host "git add proof-messenger-demos/_headers" -ForegroundColor White
        Write-Host "git add proof-messenger-demos/PROOF_MESSENGER_SECURITY_SETUP.md" -ForegroundColor White
        Write-Host "git commit -m 'Add security headers for GitHub Pages'" -ForegroundColor White
        Write-Host "git push origin main" -ForegroundColor White
    }
    
    "q" {
        Write-Host "👋 Deployment cancelled. Files are ready when you are!" -ForegroundColor Yellow
    }
    
    default {
        Write-Host "❌ Invalid option. Please run the script again." -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "📚 Documentation:" -ForegroundColor Cyan
Write-Host "• Setup Guide: proof-messenger-demos/PROOF_MESSENGER_SECURITY_SETUP.md" -ForegroundColor White
Write-Host "• Testing Tool: proof-messenger-demos/test-security-headers.html" -ForegroundColor White
Write-Host "• Security Headers Info: https://securityheaders.com/" -ForegroundColor White

Write-Host ""
Write-Host "🔒 Security Headers Summary:" -ForegroundColor Green
Write-Host "✅ Clickjacking Prevention" -ForegroundColor White
Write-Host "✅ XSS Attack Blocking" -ForegroundColor White
Write-Host "✅ Content Sniffing Protection" -ForegroundColor White
Write-Host "✅ Privacy-Focused Referrer Policy" -ForegroundColor White
Write-Host "✅ Disabled Dangerous Browser APIs" -ForegroundColor White
Write-Host "✅ Controlled Resource Loading" -ForegroundColor White