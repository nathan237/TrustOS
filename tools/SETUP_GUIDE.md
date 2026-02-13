# TrustOS Video Publisher — Setup Guide

## 1. YouTube API Setup (5 minutes, one-time)

### Step 1: Create a Google Cloud Project
1. Go to https://console.cloud.google.com/
2. Click **"Select a project"** → **"New Project"**
3. Name: `TrustOS-Publisher` → **Create**

### Step 2: Enable YouTube Data API v3
1. In the sidebar: **APIs & Services** → **Library**
2. Search for **"YouTube Data API v3"**
3. Click **Enable**

### Step 3: Create OAuth2 Credentials
1. **APIs & Services** → **Credentials**
2. Click **+ CREATE CREDENTIALS** → **OAuth client ID**
3. If prompted, configure the **OAuth consent screen**:
   - User Type: **External** → Create
   - App name: `TrustOS Publisher`
   - User support email: your email
   - Developer contact: your email
   - Click **Save and Continue** through all steps
   - Under **Test users**, add your Google/YouTube email
   - **Publish** the app (or stay in testing mode — both work)
4. Back to **Credentials** → **+ CREATE CREDENTIALS** → **OAuth client ID**:
   - Application type: **Desktop app**
   - Name: `TrustOS CLI`
   - Click **Create**
5. Click **⬇ Download JSON**
6. Rename the file to `client_secret.json`
7. Move it to: `tools/config/client_secret.json`

### Step 4: Authenticate
```bash
python tools/publish_video.py --setup
```
This opens your browser to log in with your YouTube account. The token is saved locally in `tools/tokens/youtube_token.json` — you won't need to log in again.

---

## 2. Reddit API Setup (3 minutes, one-time)

### Step 1: Create a Reddit App
1. Go to https://www.reddit.com/prefs/apps
2. Scroll down → **"create another app..."**
3. Fill in:
   - **name**: `TrustOS-Publisher`
   - **type**: ✅ **script** (important!)
   - **description**: `Auto-post TrustOS videos`
   - **redirect uri**: `http://localhost:8080`
4. Click **Create app**
5. Note down:
   - **client_id**: the string under the app name (looks like `Ab1Cd2Ef3Gh4`)
   - **client_secret**: the "secret" field

### Step 2: Configure
```bash
python tools/publish_video.py --setup
```
Enter your Reddit credentials when prompted. They're saved in `tools/config/reddit_config.json`.

Or create the file manually:
```json
{
  "client_id": "YOUR_CLIENT_ID",
  "client_secret": "YOUR_CLIENT_SECRET",
  "username": "YOUR_REDDIT_USERNAME",
  "password": "YOUR_REDDIT_PASSWORD"
}
```

---

## 3. Install Dependencies

```bash
pip install google-api-python-client google-auth-oauthlib praw
```

---

## 4. Usage

### Full publish (YouTube + Reddit):
```bash
python tools/publish_video.py demo.mp4
```

### Custom title & tags:
```bash
python tools/publish_video.py demo.mp4 --title "TrustOS v0.1.6 Demo" --tags "rust,os,demo"
```

### YouTube only:
```bash
python tools/publish_video.py demo.mp4 --youtube-only
```

### Reddit only (if video already uploaded):
```bash
python tools/publish_video.py --reddit-only --url "https://youtu.be/xxxx" --title "My Video"
```

### Dry run (preview without posting):
```bash
python tools/publish_video.py demo.mp4 --dry-run
```

### Unlisted upload:
```bash
python tools/publish_video.py demo.mp4 --privacy unlisted
```

### Custom subreddits:
```bash
python tools/publish_video.py demo.mp4 --subreddits "osdev,rust,programming"
```

---

## 5. File Structure

```
tools/
├── publish_video.py          # Main script
├── SETUP_GUIDE.md            # This file
├── config/
│   ├── client_secret.json    # YouTube OAuth2 credentials (from Google Cloud)
│   └── reddit_config.json    # Reddit API credentials
└── tokens/
    └── youtube_token.json    # Auto-generated YouTube auth token
```

## 6. Security Notes

- `config/` and `tokens/` are in `.gitignore` — credentials never get committed
- YouTube token auto-refreshes, no need to re-authenticate
- Reddit uses script-type app (simplest, no browser redirect needed)
- All credentials are stored **locally only**
