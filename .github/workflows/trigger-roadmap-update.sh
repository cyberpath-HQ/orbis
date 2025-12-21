#!/bin/bash
# Manual roadmap update script
# This script triggers the GitHub Action to update the roadmap table

set -e

OWNER="cyberpath-HQ"
REPO="orbis"
WORKFLOW_FILE="update-roadmap.yml"

echo "🚀 Triggering roadmap update workflow..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed."
    echo "Please install it: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Not authenticated with GitHub CLI."
    echo "Run: gh auth login"
    exit 1
fi

# Trigger the workflow
gh workflow run "$WORKFLOW_FILE" --repo "$OWNER/$REPO"

echo "✅ Workflow triggered successfully!"
echo ""
echo "Monitor progress at:"
echo "https://github.com/$OWNER/$REPO/actions/workflows/$WORKFLOW_FILE"
echo ""
echo "Or run: gh run watch --repo $OWNER/$REPO"
