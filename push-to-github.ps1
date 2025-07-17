# Script to push changes to GitHub with credentials

# Get GitHub credentials
$username = Read-Host "Enter your GitHub username"
$token = Read-Host "Enter your GitHub personal access token" -AsSecureString
$tokenPlain = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($token))

# Set the remote URL with credentials
$repoUrl = "https://$username`:$tokenPlain@github.com/DeepFriedCyber/proof-messenger-workspace.git"
git remote set-url origin $repoUrl

# Push to GitHub
Write-Host "Pushing changes to GitHub..."
git push origin main

# Reset the remote URL to remove credentials
git remote set-url origin "https://github.com/DeepFriedCyber/proof-messenger-workspace.git"

Write-Host "Push completed and remote URL reset."