# Script to push the demo showcase files to GitHub

Write-Host "Pushing demo showcase files to GitHub..."

# First, let's make sure we have the latest changes
git pull

# Now, let's check if the files exist
$files = @("DEMO_SHOWCASE.md", "demo-showcase.html", "linkedin-demo-showcase.html", "DEMO_SHOWCASE_README.md")
$allFilesExist = $true

foreach ($file in $files) {
    if (-not (Test-Path $file)) {
        Write-Host "File not found: $file" -ForegroundColor Red
        $allFilesExist = $false
    } else {
        Write-Host "File exists: $file" -ForegroundColor Green
    }
}

if (-not $allFilesExist) {
    Write-Host "Some files are missing. Please check the file paths." -ForegroundColor Red
    exit 1
}

# Add the files to git if they're not already tracked
git add $files

# Commit the files if there are changes
$status = git status --porcelain $files
if ($status) {
    git commit -m "Add demo showcase files for website and LinkedIn"
    Write-Host "Changes committed." -ForegroundColor Green
} else {
    Write-Host "No changes to commit. Files may already be committed." -ForegroundColor Yellow
}

# Instructions for pushing
Write-Host "`nTo push these changes to GitHub, run one of the following commands:" -ForegroundColor Cyan
Write-Host "1. Use the helper script: .\push-to-github.ps1" -ForegroundColor Cyan
Write-Host "2. Or push directly if you have credentials set up: git push origin main" -ForegroundColor Cyan

Write-Host "`nAfter pushing, you can find the files in your GitHub repository." -ForegroundColor Cyan