#!/usr/bin/env python3
"""
TrustOS Video Publisher — Upload to YouTube + post on Reddit automatically.

Usage:
    python publish_video.py VIDEO_FILE [--title TITLE] [--description DESC] [--tags TAGS]
    python publish_video.py --setup          # First-time OAuth setup
    python publish_video.py --youtube-only   # Skip Reddit
    python publish_video.py --reddit-only    # Skip YouTube (needs URL)

Examples:
    python publish_video.py demo.mp4
    python publish_video.py demo.mp4 --title "TrustOS v0.1.6 — Ed25519 + 3D Chess"
    python publish_video.py demo.mp4 --title "My Demo" --tags "rust,os,dev"
    python publish_video.py --reddit-only --url "https://youtu.be/xxx"
"""

import os
import sys
import json
import argparse
import time
from pathlib import Path

# ═══════════════════════════════════════════════════════════
# CONFIGURATION — Edit these defaults or pass as CLI args
# ═══════════════════════════════════════════════════════════

SCRIPT_DIR = Path(__file__).parent
CONFIG_DIR = SCRIPT_DIR / "config"
TOKENS_DIR = SCRIPT_DIR / "tokens"

# Default video metadata (override with --title, --description, --tags)
DEFAULT_TITLE = "TrustOS — 120,000 Lines of Pure Rust | Bare-Metal OS Demo"
DEFAULT_DESCRIPTION = """🦀 TrustOS — A fully auditable, bare-metal operating system written in 120,000 lines of pure Rust. Zero C. Zero secrets.

✨ Features shown in this video:
• COSMIC2 Desktop Environment (144 FPS, SSE2 SIMD)
• TrustLang — Built-in programming language & compiler
• TrustBrowser — Web browser with TLS 1.3 (from scratch)
• Formula3D — Real-time wireframe 3D engine
• Chess 3D — Full 3D chess game with AI opponent
• Full TCP/IP network stack, TLS 1.3, HTTPS
• Ed25519 asymmetric signatures
• 200+ shell commands
• All running on bare metal — no libc, no dependencies

🔗 Source Code: https://github.com/nathan237/TrustOS
⭐ Star the repo if you believe in transparent, auditable operating systems!

#TrustOS #Rust #OSDev #BareMetal #OperatingSystem #Programming
"""

DEFAULT_TAGS = [
    "TrustOS", "Rust", "OSDev", "operating system", "bare metal",
    "programming", "kernel", "Rust programming", "OS development",
    "desktop environment", "from scratch", "no dependencies",
    "TrustLang", "3D engine", "compiler", "TLS 1.3", "Ed25519",
]

# Reddit subreddits to post to
REDDIT_SUBREDDITS = ["osdev", "rust", "programming", "linux"]

# Reddit post template
REDDIT_TITLE_TEMPLATE = "{title} [120K lines of Rust, zero C]"
REDDIT_FLAIR = None  # Set per-subreddit if needed


# ═══════════════════════════════════════════════════════════
# YOUTUBE UPLOAD
# ═══════════════════════════════════════════════════════════

def youtube_authenticate():
    """Authenticate with YouTube Data API v3 via OAuth2."""
    try:
        from google_auth_oauthlib.flow import InstalledAppFlow
        from google.auth.transport.requests import Request
        from google.oauth2.credentials import Credentials
    except ImportError:
        print("❌ Missing packages. Run:")
        print("   pip install google-api-python-client google-auth-oauthlib")
        sys.exit(1)

    SCOPES = ["https://www.googleapis.com/auth/youtube.upload"]
    token_path = TOKENS_DIR / "youtube_token.json"
    client_secret = CONFIG_DIR / "client_secret.json"

    if not client_secret.exists():
        print(f"❌ client_secret.json not found at: {client_secret}")
        print("   Follow the setup guide in tools/SETUP_GUIDE.md")
        sys.exit(1)

    creds = None
    if token_path.exists():
        creds = Credentials.from_authorized_user_file(str(token_path), SCOPES)

    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            print("🔄 Refreshing YouTube token...")
            creds.refresh(Request())
        else:
            print("🌐 Opening browser for YouTube authentication...")
            print("   (This only happens once — token is saved locally)")
            flow = InstalledAppFlow.from_client_secrets_file(str(client_secret), SCOPES)
            creds = flow.run_local_server(port=8090, open_browser=True)

        TOKENS_DIR.mkdir(parents=True, exist_ok=True)
        with open(token_path, "w") as f:
            f.write(creds.to_json())
        print("✅ YouTube token saved.")

    return creds


def youtube_upload(video_path: str, title: str, description: str, tags: list,
                   privacy: str = "public", category: str = "28"):
    """Upload a video to YouTube. Returns the video URL."""
    from googleapiclient.discovery import build
    from googleapiclient.http import MediaFileUpload

    creds = youtube_authenticate()
    youtube = build("youtube", "v3", credentials=creds)

    body = {
        "snippet": {
            "title": title,
            "description": description,
            "tags": tags,
            "categoryId": category,  # 28 = Science & Technology
        },
        "status": {
            "privacyStatus": privacy,
            "selfDeclaredMadeForKids": False,
        },
    }

    media = MediaFileUpload(video_path, chunksize=10 * 1024 * 1024, resumable=True)
    request = youtube.videos().insert(part="snippet,status", body=body, media_body=media)

    print(f"\n📤 Uploading to YouTube: {title}")
    print(f"   File: {video_path} ({os.path.getsize(video_path) / 1024 / 1024:.1f} MB)")

    response = None
    while response is None:
        status, response = request.next_chunk()
        if status:
            pct = int(status.progress() * 100)
            print(f"   ⏳ {pct}% uploaded...", end="\r")

    video_id = response["id"]
    video_url = f"https://youtu.be/{video_id}"
    print(f"\n✅ YouTube upload complete!")
    print(f"   🔗 {video_url}")
    return video_url


# ═══════════════════════════════════════════════════════════
# REDDIT POSTING
# ═══════════════════════════════════════════════════════════

def reddit_authenticate():
    """Authenticate with Reddit API via PRAW."""
    try:
        import praw
    except ImportError:
        print("❌ Missing package. Run:")
        print("   pip install praw")
        sys.exit(1)

    reddit_config = CONFIG_DIR / "reddit_config.json"
    if not reddit_config.exists():
        print(f"❌ reddit_config.json not found at: {reddit_config}")
        print("   Follow the setup guide in tools/SETUP_GUIDE.md")
        sys.exit(1)

    with open(reddit_config) as f:
        cfg = json.load(f)

    reddit = praw.Reddit(
        client_id=cfg["client_id"],
        client_secret=cfg["client_secret"],
        username=cfg["username"],
        password=cfg["password"],
        user_agent=f"TrustOS-Publisher/1.0 by /u/{cfg['username']}",
    )

    print(f"✅ Reddit authenticated as: /u/{reddit.user.me()}")
    return reddit


def reddit_post(video_url: str, title: str, subreddits: list = None):
    """Post the video link to Reddit subreddits."""
    reddit = reddit_authenticate()
    subs = subreddits or REDDIT_SUBREDDITS

    print(f"\n📮 Posting to {len(subs)} subreddits...")
    results = []

    for sub_name in subs:
        try:
            subreddit = reddit.subreddit(sub_name)

            # Format title for this subreddit
            post_title = REDDIT_TITLE_TEMPLATE.format(title=title)

            # Customize per subreddit
            if sub_name == "rust":
                post_title = f"[Media] {title} — bare-metal OS in 120K lines of Rust"
            elif sub_name == "osdev":
                post_title = f"{title} — bare-metal Rust OS (120K lines, zero C)"
            elif sub_name == "linux":
                post_title = f"{title} — auditable Rust OS with Linux compat layer"

            submission = subreddit.submit(
                title=post_title,
                url=video_url,
                flair_id=REDDIT_FLAIR,
                resubmit=False,
            )

            print(f"   ✅ r/{sub_name}: https://reddit.com{submission.permalink}")
            results.append({"subreddit": sub_name, "url": f"https://reddit.com{submission.permalink}", "ok": True})

            # Wait between posts to avoid rate limiting
            time.sleep(8)

        except Exception as e:
            err = str(e)
            print(f"   ❌ r/{sub_name}: {err[:100]}")
            results.append({"subreddit": sub_name, "error": err, "ok": False})
            time.sleep(5)

    return results


# ═══════════════════════════════════════════════════════════
# SETUP HELPERS
# ═══════════════════════════════════════════════════════════

def run_setup():
    """Interactive first-time setup."""
    print("=" * 60)
    print("  TrustOS Video Publisher — First-Time Setup")
    print("=" * 60)

    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    TOKENS_DIR.mkdir(parents=True, exist_ok=True)

    # YouTube setup
    print("\n── YouTube Setup ──")
    client_secret = CONFIG_DIR / "client_secret.json"
    if client_secret.exists():
        print("✅ client_secret.json found.")
    else:
        print(f"📁 Place your client_secret.json in: {CONFIG_DIR}")
        print("   (Download from Google Cloud Console → APIs & Services → Credentials)")
        input("   Press Enter when ready...")
        if not client_secret.exists():
            print("⚠️  File not found, skipping YouTube setup.")
        else:
            print("✅ client_secret.json found!")

    if client_secret.exists():
        print("🔐 Authenticating with YouTube (opens browser)...")
        youtube_authenticate()

    # Reddit setup
    print("\n── Reddit Setup ──")
    reddit_config = CONFIG_DIR / "reddit_config.json"
    if reddit_config.exists():
        print("✅ reddit_config.json found.")
    else:
        print("Enter your Reddit API credentials:")
        client_id = input("  Client ID: ").strip()
        client_secret_r = input("  Client Secret: ").strip()
        username = input("  Reddit Username: ").strip()
        password = input("  Reddit Password: ").strip()

        cfg = {
            "client_id": client_id,
            "client_secret": client_secret_r,
            "username": username,
            "password": password,
        }
        with open(reddit_config, "w") as f:
            json.dump(cfg, f, indent=2)
        print(f"✅ Reddit config saved to: {reddit_config}")

    print("\n✅ Setup complete! You can now run:")
    print(f"   python {__file__} YOUR_VIDEO.mp4")


# ═══════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════

def main():
    parser = argparse.ArgumentParser(
        description="TrustOS Video Publisher — Upload to YouTube + post on Reddit",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument("video", nargs="?", help="Path to the video file")
    parser.add_argument("--title", "-t", default=DEFAULT_TITLE, help="Video title")
    parser.add_argument("--description", "-d", default=DEFAULT_DESCRIPTION, help="Video description")
    parser.add_argument("--tags", default=None, help="Comma-separated tags (overrides defaults)")
    parser.add_argument("--privacy", default="public", choices=["public", "unlisted", "private"],
                        help="YouTube privacy status")
    parser.add_argument("--setup", action="store_true", help="Run first-time setup wizard")
    parser.add_argument("--youtube-only", action="store_true", help="Only upload to YouTube")
    parser.add_argument("--reddit-only", action="store_true", help="Only post to Reddit")
    parser.add_argument("--url", default=None, help="YouTube URL (for --reddit-only)")
    parser.add_argument("--subreddits", default=None,
                        help="Comma-separated subreddits (overrides defaults)")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done without doing it")

    args = parser.parse_args()

    if args.setup:
        run_setup()
        return

    tags = args.tags.split(",") if args.tags else DEFAULT_TAGS
    subreddits = args.subreddits.split(",") if args.subreddits else REDDIT_SUBREDDITS

    if args.dry_run:
        print("═══ DRY RUN ═══")
        print(f"Title:       {args.title}")
        print(f"Tags:        {', '.join(tags[:8])}...")
        print(f"Privacy:     {args.privacy}")
        print(f"Video:       {args.video or '(none)'}")
        print(f"YouTube:     {'skip' if args.reddit_only else 'upload'}")
        print(f"Reddit:      {'skip' if args.youtube_only else ', '.join('r/' + s for s in subreddits)}")
        return

    video_url = args.url

    # YouTube upload
    if not args.reddit_only:
        if not args.video:
            print("❌ No video file specified. Usage: python publish_video.py VIDEO_FILE")
            sys.exit(1)
        if not os.path.exists(args.video):
            print(f"❌ Video file not found: {args.video}")
            sys.exit(1)

        video_url = youtube_upload(
            video_path=args.video,
            title=args.title,
            description=args.description,
            tags=tags,
            privacy=args.privacy,
        )

    # Reddit posting
    if not args.youtube_only:
        if not video_url:
            print("❌ No YouTube URL. Upload first or use --url")
            sys.exit(1)

        reddit_post(
            video_url=video_url,
            title=args.title,
            subreddits=subreddits,
        )

    print("\n🎉 All done!")


if __name__ == "__main__":
    main()
