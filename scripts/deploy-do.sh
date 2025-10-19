#!/bin/bash

# QuartzDB DigitalOcean Deployment Script
# This script helps deploy QuartzDB to DigitalOcean App Platform

set -e

echo "🚀 QuartzDB DigitalOcean Deployment Setup"
echo "=========================================="
echo ""

# Check if doctl is installed
if ! command -v doctl &> /dev/null; then
    echo "❌ doctl CLI not found"
    echo "📥 Install it with: brew install doctl"
    echo "📖 Or visit: https://docs.digitalocean.com/reference/doctl/how-to/install/"
    exit 1
fi

echo "✅ doctl is installed"

# Check if authenticated
if ! doctl auth list &> /dev/null; then
    echo "🔐 Not authenticated with DigitalOcean"
    echo "Run: doctl auth init"
    exit 1
fi

echo "✅ Authenticated with DigitalOcean"

# Prompt for GitHub username
read -p "📝 Enter your GitHub username: " github_user

# Update app.yaml with GitHub username
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/YOUR_USERNAME/$github_user/g" .do/app.yaml
else
    # Linux
    sed -i "s/YOUR_USERNAME/$github_user/g" .do/app.yaml
fi

echo "✅ Updated app.yaml with GitHub username: $github_user"

# Ask if user wants to create the app now
read -p "🚀 Deploy to DigitalOcean now? (y/n): " deploy_now

if [[ "$deploy_now" == "y" || "$deploy_now" == "Y" ]]; then
    echo ""
    echo "📤 Creating DigitalOcean App..."
    
    # Create the app
    doctl apps create --spec .do/app.yaml
    
    echo ""
    echo "✅ App created successfully!"
    echo ""
    echo "📊 View your app:"
    echo "   doctl apps list"
    echo ""
    echo "📝 View logs:"
    echo "   doctl apps logs <APP_ID>"
    echo ""
    echo "🌐 Your app will be available at:"
    echo "   https://<app-name>.ondigitalocean.app"
    echo ""
    echo "💰 Estimated cost: $5/month (basic-xxs instance)"
    
else
    echo ""
    echo "📋 Manual deployment steps:"
    echo ""
    echo "1. Commit and push your changes:"
    echo "   git add ."
    echo "   git commit -m 'Add DigitalOcean deployment config'"
    echo "   git push origin main"
    echo ""
    echo "2. Create the app:"
    echo "   doctl apps create --spec .do/app.yaml"
    echo ""
    echo "3. Or use the DigitalOcean web console:"
    echo "   https://cloud.digitalocean.com/apps"
    echo ""
fi

echo ""
echo "📖 For more information, see:"
echo "   https://docs.digitalocean.com/products/app-platform/"
echo ""
echo "✨ Happy deploying!"
