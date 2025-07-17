# GitHub Push Instructions

This document provides instructions for pushing the recent changes to GitHub.

## Option 1: Using Personal Access Token (Temporary)

1. Run the `push-to-github.ps1` script:
   ```
   .\push-to-github.ps1
   ```

2. Enter your GitHub username when prompted.

3. Enter your GitHub personal access token when prompted.
   - If you don't have a personal access token, create one at:
   - GitHub > Settings > Developer settings > Personal access tokens > Generate new token
   - Make sure to give it the `repo` scope

4. The script will push the changes and then reset the remote URL to remove your credentials.

## Option 2: Set Up SSH Authentication (Recommended)

1. Run the `setup-ssh-github.ps1` script:
   ```
   .\setup-ssh-github.ps1
   ```

2. Follow the prompts to generate an SSH key (if needed) and add it to your GitHub account.

3. Once SSH is set up, you can push with:
   ```
   git push origin main
   ```

## Option 3: Manual Push from Another Machine

If you prefer to push from another machine where you already have GitHub authentication set up:

1. Copy the following files to your other machine:
   - `robust-load-test.ps1`
   - `simple-verify.ps1`
   - `proof-messenger-dashboard.json`

2. Commit and push from that machine:
   ```
   git add robust-load-test.ps1 simple-verify.ps1 proof-messenger-dashboard.json
   git commit -m "Add load testing scripts and Grafana dashboard"
   git push
   ```

## Files Added in This Commit

- `robust-load-test.ps1`: A comprehensive load testing script for the Proof-Messenger relay server
- `simple-verify.ps1`: A simple data integrity verification script
- `proof-messenger-dashboard.json`: A Grafana dashboard configuration for monitoring the relay server

## Grafana Dashboard Import Instructions

To import the Grafana dashboard:

1. Open Grafana at http://localhost:3000
2. Click the "+" icon in the left sidebar
3. Select "Import"
4. Either upload the `proof-messenger-dashboard.json` file or copy-paste its contents
5. Click "Load"
6. Select your Prometheus data source
7. Click "Import"