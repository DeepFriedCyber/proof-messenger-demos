# Script to set up SSH authentication for GitHub

# Check if SSH key exists
$sshKeyPath = "$env:USERPROFILE\.ssh\id_rsa"
if (-not (Test-Path $sshKeyPath)) {
    Write-Host "No SSH key found. Creating a new SSH key..."
    
    # Create .ssh directory if it doesn't exist
    if (-not (Test-Path "$env:USERPROFILE\.ssh")) {
        New-Item -ItemType Directory -Path "$env:USERPROFILE\.ssh" | Out-Null
    }
    
    # Generate SSH key
    ssh-keygen -t rsa -b 4096 -C "your_email@example.com" -f $sshKeyPath
    
    Write-Host "SSH key generated at $sshKeyPath"
} else {
    Write-Host "Existing SSH key found at $sshKeyPath"
}

# Display the public key to add to GitHub
Write-Host "`nHere is your public SSH key. Add this to your GitHub account:`n"
Get-Content "$sshKeyPath.pub"

Write-Host "`n1. Go to GitHub > Settings > SSH and GPG keys"
Write-Host "2. Click 'New SSH key'"
Write-Host "3. Paste the key above and save"

# Change remote URL to use SSH
Write-Host "`nWould you like to change the remote URL to use SSH? (y/n)"
$response = Read-Host
if ($response -eq "y") {
    git remote set-url origin git@github.com:DeepFriedCyber/proof-messenger-workspace.git
    Write-Host "Remote URL changed to use SSH"
    
    # Test SSH connection
    Write-Host "`nTesting SSH connection to GitHub..."
    ssh -T git@github.com
}

Write-Host "`nNow you can push using: git push origin main"