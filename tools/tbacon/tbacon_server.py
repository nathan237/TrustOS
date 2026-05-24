#!/usr/bin/env python3
"""
T-Bacon Private Hub.

Local MVP for user/session management, private rooms, HMAC image beacons, and
an admin dashboard. It is intentionally small and auditable.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import hmac
import json
import os
import secrets
import sqlite3
import struct
import threading
import time
from io import BytesIO
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
from shutil import copyfile, rmtree

import re

import qrcode


REPO_ROOT = Path(__file__).resolve().parents[2]

# ---------------------------------------------------------------------------
# Security: recon/scan pattern detection
# ---------------------------------------------------------------------------
_RECON_RE = re.compile(
    r'(?:'
    r'\.env|\.git/|wp-admin|wp-login\.php|phpmyadmin|xmlrpc\.php|'
    r'\.php$|/cgi-bin/|/etc/passwd|proc/self|\.htaccess|web\.config|'
    r'actuator|/swagger|/graphql|/manager/|/console|setup\.php|'
    r'config\.php|install\.php|test\.php|shell\.php|cmd\.php|'
    r'backup|/mysql|/mssql|\.bak$|\.sql$|\.zip$|\.tar\.gz$|'
    r'\.\./|%2e%2e|%252e|%00|null\.php|eval\(|'
    r'union.*select|<script|javascript:|vbscript:|'
    r'\.well-known/acme|/telescope|/horizon|/_profiler|'
    r'/server-status|/server-info|/nginx_status'
    r')',
    re.IGNORECASE,
)
_AUTO_BAN_RECON_THRESHOLD = 3   # recon hits before auto-ban
_AUTO_BAN_404_THRESHOLD   = 20  # 404s in window before auto-ban
_AUTO_BAN_WINDOW_SECS     = 300 # 5 minutes


# ---------------------------------------------------------------------------
# WebSocket pub/sub manager (no external deps)
# ---------------------------------------------------------------------------

class _WsManager:
    """Minimal in-process WebSocket broadcaster."""

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._conns: dict[str, list] = {}  # account_id → [(send_fn, close_fn)]

    def add(self, account_id: str, send_fn, close_fn) -> None:
        with self._lock:
            self._conns.setdefault(account_id, []).append((send_fn, close_fn))

    def remove(self, account_id: str, send_fn) -> None:
        with self._lock:
            if account_id in self._conns:
                self._conns[account_id] = [p for p in self._conns[account_id] if p[0] is not send_fn]

    def broadcast(self, account_id: str, event: dict) -> None:
        msg = json.dumps(event, separators=(",", ":"))
        with self._lock:
            pairs = list(self._conns.get(account_id, []))
        for send_fn, close_fn in pairs:
            try:
                send_fn(msg)
            except Exception:
                try:
                    close_fn()
                except Exception:
                    pass


_ws_mgr = _WsManager()
PNG_1X1 = base64.b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII="
)
ALLOWED_OPS = {"alive", "ack", "nack", "help", "retry", "cap_l0", "cap_l1", "cap_l2", "cap_l3"}
MAX_SKEW_SECONDS = 300
MAX_JSON_BYTES = 2_500_000
MAX_TEXT_MESSAGE_BYTES = 4_000
MAX_PIXEL_CHUNKS = 80
MAX_PIXEL_CHUNK_PNG_BYTES = 1_200_000
DEFAULT_RETENTION_SECONDS = 7 * 24 * 60 * 60
OOB_SESSION_SECONDS = 15 * 60


SCHEMA = """
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  secret TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  last_seen INTEGER,
  level TEXT DEFAULT 'L0',
  revoked INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS rooms (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS room_members (
  room_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  PRIMARY KEY (room_id, user_id)
);

CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at INTEGER NOT NULL,
  user_id TEXT,
  sid TEXT,
  op TEXT NOT NULL,
  seq INTEGER,
  nonce TEXT,
  level TEXT,
  ok INTEGER NOT NULL,
  error TEXT
);

CREATE TABLE IF NOT EXISTS replay (
  replay_key TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at INTEGER NOT NULL,
  room_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  body TEXT NOT NULL,
  transport TEXT DEFAULT 'text'
);

CREATE TABLE IF NOT EXISTS room_carriers (
  room_id TEXT PRIMARY KEY,
  mask_id TEXT NOT NULL,
  mask_json TEXT NOT NULL,
  carrier_frame_path TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS twookie_accounts (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  public_key TEXT NOT NULL,
  seed_hash TEXT NOT NULL,
  hardware_log_hash TEXT NOT NULL,
  hardware_log_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  last_seen INTEGER,
  revoked INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS twookie_contacts (
  owner_id TEXT NOT NULL,
  peer_id TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (owner_id, peer_id)
);

CREATE TABLE IF NOT EXISTS twookie_calls (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at INTEGER NOT NULL,
  caller_id TEXT NOT NULL,
  peer_id TEXT NOT NULL,
  mode TEXT NOT NULL,
  status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS image_nodes (
  id TEXT PRIMARY KEY,
  prev_id TEXT,
  payload_hash TEXT NOT NULL,
  image_path TEXT,
  created_at INTEGER NOT NULL,
  meta_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS twookie_bridge_messages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at INTEGER NOT NULL,
  direction TEXT NOT NULL,
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  body TEXT NOT NULL,
  status TEXT NOT NULL,
  source TEXT NOT NULL,
  meta_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS twookie_voice_memos (
  id TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  audio_base64 TEXT NOT NULL,
  revoked INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS twookie_pixel_payloads (
  id TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  algorithm TEXT NOT NULL,
  chunk_count INTEGER NOT NULL,
  payload_hash TEXT NOT NULL,
  meta_json TEXT NOT NULL,
  revoked INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS twookie_pixel_chunks (
  payload_id TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  png_base64 TEXT NOT NULL,
  file_path TEXT DEFAULT '',
  chunk_hash TEXT NOT NULL,
  PRIMARY KEY (payload_id, chunk_index)
);

CREATE TABLE IF NOT EXISTS twookie_pixel_uploads (
  id TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  algorithm TEXT NOT NULL,
  chunk_count INTEGER NOT NULL,
  payload_hash TEXT NOT NULL,
  meta_json TEXT NOT NULL,
  status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS twookie_pixel_upload_chunks (
  upload_id TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  file_path TEXT NOT NULL,
  chunk_hash TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (upload_id, chunk_index)
);

CREATE TABLE IF NOT EXISTS twookie_invites (
  token TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  inviter_id TEXT NOT NULL,
  label TEXT NOT NULL,
  accepted_by TEXT,
  accepted_at INTEGER,
  revoked INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS twookie_idempotency (
  scope TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  request_hash TEXT NOT NULL,
  result_json TEXT NOT NULL,
  PRIMARY KEY (scope, idempotency_key)
);

CREATE TABLE IF NOT EXISTS twookie_oob_sessions (
  id TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  client_pubkey TEXT NOT NULL,
  peer_pubkey TEXT DEFAULT '',
  transcript_hash TEXT NOT NULL,
  verify_phrase TEXT NOT NULL,
  status TEXT NOT NULL,
  confirmed_by TEXT DEFAULT '',
  meta_json TEXT NOT NULL
);
"""


def now() -> int:
    return int(time.time())


def sign(secret: str, sid: str, user: str, op: str, seq: str, nonce: str, ts: str) -> str:
    material = "|".join([sid, user, op, seq, nonce, ts]).encode("utf-8")
    return hmac.new(secret.encode("utf-8"), material, hashlib.sha256).hexdigest()


def safe_id(value: str) -> str:
    cleaned = "".join(ch.lower() for ch in str(value).strip() if ch.isalnum() or ch in ("-", "_"))
    return cleaned[:48]


def clean_text(value: str, max_len: int) -> str:
    return str(value).replace("\x00", "").strip()[:max_len]


def retention_seconds() -> int:
    return int(os.getenv("TWOOKIE_RETENTION_SECONDS", str(DEFAULT_RETENTION_SECONDS)))


def canonical_json(value: object) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True)


def sha256_hex_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_hex_text(value: object) -> str:
    return sha256_hex_bytes(canonical_json(value).encode("utf-8"))


def require_hex_hash(value: str, field: str, lengths: tuple[int, ...] = (64, 128)) -> str:
    value = str(value or "").replace("\x00", "").strip()
    if len(value) not in lengths or any(ch not in "0123456789abcdefABCDEF" for ch in value):
        raise ValueError(f"{field} must be a hex hash")
    return value.lower()


def require_safe_identifier(value: str, field: str, max_len: int = 48) -> str:
    clean = safe_id(value)
    if not clean:
        raise ValueError(f"{field} required")
    if len(clean) > max_len:
        raise ValueError(f"{field} too long")
    return clean


def require_text_field(value: str, field: str, max_len: int) -> str:
    clean = str(value or "").replace("\x00", "").strip()
    if not clean:
        raise ValueError(f"{field} required")
    if len(clean.encode("utf-8")) > max_len:
        raise ValueError(f"{field} too large")
    return clean


def idempotency_request_shape(value: object) -> object:
    if isinstance(value, dict):
        out = {}
        for key, item in value.items():
            if key in {"audio_base64", "png_base64"}:
                text = str(item or "")
                out[key] = {"sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(), "length": len(text)}
            elif key == "chunks" and isinstance(item, list):
                out[key] = [
                    {
                        "index": chunk.get("index"),
                        "chunk_hash": chunk.get("chunk_hash"),
                        "png_base64_sha256": hashlib.sha256(str(chunk.get("png_base64") or "").encode("utf-8")).hexdigest(),
                        "png_base64_length": len(str(chunk.get("png_base64") or "")),
                    }
                    for chunk in item
                    if isinstance(chunk, dict)
                ]
            else:
                out[key] = idempotency_request_shape(item)
        return out
    if isinstance(value, list):
        return [idempotency_request_shape(item) for item in value]
    return value


def parse_meta(row: sqlite3.Row | dict) -> dict:
    try:
        return json.loads(row["meta_json"] or "{}")
    except Exception:
        return {}


def codex_local_reply(prompt: str) -> str:
    text = clean_text(prompt, 500).lower()
    if not text:
        return "Je te recois dans T-Wookie. Envoie une petite question et je reponds ici."
    if any(word in text for word in ("salut", "bonjour", "allo", "hello")):
        return "Salut Nate. Codex Local te recoit bien via T-Wookie."
    if "ping" in text:
        return "pong. Transport web_local confirme."
    if "heure" in text or "time" in text:
        return f"Heure locale serveur: {time.strftime('%H:%M:%S')}."
    if "transport" in text or "data" in text or "donnee" in text:
        return "Transport: POST /api/twookie/messages -> SQLite twookie_bridge_messages -> polling /bridge/recent -> rendu UI."
    if "qui" in text and ("tu" in text or "es" in text):
        return "Je suis Codex Local, un responder de test branche sur le canal T-Wookie."
    if "refresh" in text or "actualis" in text:
        return "Oui, la page poll les nouveaux messages toutes les 2.5 secondes."
    if "notification" in text or "non-lu" in text:
        return "Les notifications ne s'allument que pour les messages entrants sur un canal inactif."
    return f"Recu dans T-Wookie: {clean_text(prompt, 180)}"


def twookie_context_pack(origin: str = "") -> dict:
    origin = clean_text(origin or "http://127.0.0.1:8894", 200).rstrip("/")
    return {
        "project": "T-Wookie",
        "purpose": "Private talkie-walkie style web hub for testing local-first messaging, invite links, QR onboarding, Discord relay, and future image/video carrier transports.",
        "public_test_url": origin,
        "entrypoints": {
            "app": f"{origin}/",
            "human_context": f"{origin}/ai-context",
            "machine_context": f"{origin}/api/twookie/context",
            "accounts": f"{origin}/api/twookie/accounts",
            "messages": f"{origin}/api/twookie/bridge/recent?owner=<account_id>&after=0",
            "invite_scoped_write_get": f"{origin}/api/twookie/invites/<invite_token>/ai-message?from=chatgpt&body=Test%20depuis%20IA",
            "invite_scoped_write_post": f"{origin}/api/twookie/invites/<invite_token>/message",
        },
        "current_features": [
            "Create/connect a browser account with public key, seed hash, and optional hashed device log metadata.",
            "Create QR/link invitations so another browser can join the inviter as a contact.",
            "Send direct web-local messages in a Messenger/talkie UI.",
            "Show message source labels for web, local Codex responder, Discord bridge, and future platforms.",
            "Persist transport records in SQLite through twookie_bridge_messages.",
            "Expose a Codex Local test contact for deterministic replies.",
            "Allow invite-scoped public AI test writes without exposing global write access.",
        ],
        "ai_role": "Act as a remote test assistant. Read this context, open the app URL if your environment allows browsing, then help validate onboarding, sending a message, receiving a reply, and reporting what transport/data path was observed.",
        "test_script": [
            "Open the app URL.",
            "Create or connect a simple test account.",
            "Use an invite link/QR if provided by the operator.",
            "If UI automation is unavailable, open the invite-scoped GET write endpoint with body=<short public test message>.",
            "Otherwise send a short message to the active contact or Codex Local.",
            "Report: account id used, visible source labels, whether the message persisted after refresh, and any UI friction.",
        ],
        "write_test_contract": {
            "scope": "invite token only",
            "get_example": f"{origin}/api/twookie/invites/<invite_token>/ai-message?from=chatgpt&body=Test%20depuis%20ChatGPT",
            "post_example": {
                "url": f"{origin}/api/twookie/invites/<invite_token>/message",
                "method": "POST",
                "json": {"from": "chatgpt", "body": "Test depuis ChatGPT via endpoint invite T-Wookie."},
            },
            "result": "Creates/updates an AI guest contact and posts a web_local message to the invitation issuer.",
        },
        "security_bounds": [
            "Do not request or expose ngrok authtokens, Discord bot tokens, API keys, private seeds, or raw device fingerprints.",
            "Treat this as a prototype transport/UX test, not a production secure messenger.",
            "Only use public test messages unless the operator explicitly enables encrypted payload testing.",
        ],
    }


def twookie_public_session() -> dict:
    session_path = REPO_ROOT / "reports/tbacon/latest/twookie_ngrok_session.json"
    if not session_path.exists():
        return {"public_url": "", "ai_context_url": "", "machine_context_url": "", "active": False}
    try:
        session = json.loads(session_path.read_text(encoding="utf-8-sig"))
    except (OSError, json.JSONDecodeError):
        return {"public_url": "", "ai_context_url": "", "machine_context_url": "", "active": False}
    public_url = clean_text(session.get("public_url") or "", 220).rstrip("/")
    ai_context_url = clean_text(session.get("ai_context_url") or "", 260)
    machine_context_url = clean_text(session.get("machine_context_url") or "", 260)
    return {
        "public_url": public_url,
        "ai_context_url": ai_context_url,
        "machine_context_url": machine_context_url,
        "active": bool(public_url.startswith("https://")),
    }


def twookie_ai_context_page(origin: str) -> str:
    context = twookie_context_pack(origin)
    context_json = json.dumps(context, indent=2, ensure_ascii=False)
    escaped = (
        context_json.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )
    return f"""<!doctype html>
<html lang="fr">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>T-Wookie AI Context</title>
<style>
body {{ margin: 0; background: #10131a; color: #f6f7fb; font-family: Inter, "Segoe UI", system-ui, sans-serif; }}
main {{ max-width: 920px; margin: 0 auto; padding: 28px 18px 42px; }}
h1 {{ margin: 0 0 8px; font-size: 30px; }}
p {{ color: #c7ced9; line-height: 1.55; }}
.card {{ border: 1px solid #2c3340; background: rgba(255,255,255,.045); border-radius: 18px; padding: 16px; margin: 16px 0; }}
a {{ color: #ffd2a8; font-weight: 800; }}
button {{ border: 0; border-radius: 16px; min-height: 42px; padding: 10px 14px; font: inherit; font-weight: 900; cursor: pointer; background: linear-gradient(135deg, #ff7a1a, #c94e16); color: #170904; }}
textarea {{ width: 100%; min-height: 460px; margin-top: 12px; border-radius: 16px; border: 1px solid #333b49; background: #0b0e14; color: #f7f2ff; padding: 14px; font: 13px/1.45 ui-monospace, SFMono-Regular, Consolas, monospace; }}
.tiny {{ color: #98a4b4; font-size: 13px; }}
</style>
<main>
  <h1>T-Wookie AI Context</h1>
  <p>Cette page donne a une IA externe le contexte minimum pour tester T-Wookie via un lien public ngrok, sans exposer de secret.</p>
  <div class="card">
    <p><strong>App:</strong> <a href="{context['entrypoints']['app']}">{context['entrypoints']['app']}</a></p>
    <p><strong>JSON machine:</strong> <a href="{context['entrypoints']['machine_context']}">{context['entrypoints']['machine_context']}</a></p>
    <p><strong>Ecriture test invite:</strong> <code>/api/twookie/invites/&lt;invite_token&gt;/ai-message?from=chatgpt&amp;body=Test</code></p>
    <p class="tiny">Si l'IA ne peut pas remplir l'interface, elle peut ouvrir ce lien GET avec le token d'invitation. Le message arrive dans le canal de l'emetteur de l'invitation.</p>
    <p class="tiny">A coller dans GPT/Claude/autre IA si elle ne peut pas lire le lien directement.</p>
    <button id="copy">Copier le contexte</button>
  </div>
  <textarea id="ctx" spellcheck="false">{escaped}</textarea>
</main>
<script>
document.querySelector("#copy").addEventListener("click", async () => {{
  await navigator.clipboard.writeText(document.querySelector("#ctx").value);
  document.querySelector("#copy").textContent = "Copie";
}});
</script>
</html>
"""


class Hub:
    def __init__(self, db_path: Path) -> None:
        self.db_path = db_path
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self.init_db()

    def connect(self) -> sqlite3.Connection:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        conn.execute("PRAGMA busy_timeout = 5000")
        conn.execute("PRAGMA journal_mode = WAL")
        conn.execute("PRAGMA synchronous = NORMAL")
        conn.execute("PRAGMA foreign_keys = ON")
        return conn

    def init_db(self) -> None:
        with self.connect() as conn:
            conn.executescript(SCHEMA)
            self.migrate_db(conn)
            existing = conn.execute("SELECT COUNT(*) AS count FROM users").fetchone()["count"]
            if existing == 0:
                self.create_user("alice", "Alice", conn)
                self.create_user("bob", "Bob", conn)
            rooms = conn.execute("SELECT COUNT(*) AS count FROM rooms").fetchone()["count"]
            if rooms == 0:
                room_id = "demo"
                conn.execute(
                    "INSERT INTO rooms (id, name, created_at) VALUES (?, ?, ?)",
                    (room_id, "Demo Room", now()),
                )
                conn.execute("INSERT OR IGNORE INTO room_members (room_id, user_id) VALUES (?, ?)", (room_id, "alice"))
                conn.execute("INSERT OR IGNORE INTO room_members (room_id, user_id) VALUES (?, ?)", (room_id, "bob"))

    def migrate_db(self, conn: sqlite3.Connection) -> None:
        message_columns = {row["name"] for row in conn.execute("PRAGMA table_info(messages)").fetchall()}
        if "transport" not in message_columns:
            conn.execute("ALTER TABLE messages ADD COLUMN transport TEXT DEFAULT 'text'")
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_voice_memos (
              id TEXT PRIMARY KEY,
              created_at INTEGER NOT NULL,
              from_id TEXT NOT NULL,
              to_id TEXT NOT NULL,
              mime_type TEXT NOT NULL,
              duration_ms INTEGER NOT NULL,
              audio_base64 TEXT NOT NULL,
              revoked INTEGER DEFAULT 0
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_pixel_payloads (
              id TEXT PRIMARY KEY,
              created_at INTEGER NOT NULL,
              from_id TEXT NOT NULL,
              to_id TEXT NOT NULL,
              mime_type TEXT NOT NULL,
              duration_ms INTEGER NOT NULL,
              algorithm TEXT NOT NULL,
              chunk_count INTEGER NOT NULL,
              payload_hash TEXT NOT NULL,
              meta_json TEXT NOT NULL,
              revoked INTEGER DEFAULT 0
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_pixel_chunks (
              payload_id TEXT NOT NULL,
              chunk_index INTEGER NOT NULL,
              png_base64 TEXT NOT NULL,
              file_path TEXT DEFAULT '',
              chunk_hash TEXT NOT NULL,
              PRIMARY KEY (payload_id, chunk_index)
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_pixel_uploads (
              id TEXT PRIMARY KEY,
              created_at INTEGER NOT NULL,
              expires_at INTEGER NOT NULL,
              from_id TEXT NOT NULL,
              to_id TEXT NOT NULL,
              mime_type TEXT NOT NULL,
              duration_ms INTEGER NOT NULL,
              algorithm TEXT NOT NULL,
              chunk_count INTEGER NOT NULL,
              payload_hash TEXT NOT NULL,
              meta_json TEXT NOT NULL,
              status TEXT NOT NULL
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_pixel_upload_chunks (
              upload_id TEXT NOT NULL,
              chunk_index INTEGER NOT NULL,
              file_path TEXT NOT NULL,
              chunk_hash TEXT NOT NULL,
              created_at INTEGER NOT NULL,
              PRIMARY KEY (upload_id, chunk_index)
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_idempotency (
              scope TEXT NOT NULL,
              idempotency_key TEXT NOT NULL,
              created_at INTEGER NOT NULL,
              expires_at INTEGER NOT NULL,
              request_hash TEXT NOT NULL,
              result_json TEXT NOT NULL,
              PRIMARY KEY (scope, idempotency_key)
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_oob_sessions (
              id TEXT PRIMARY KEY,
              created_at INTEGER NOT NULL,
              expires_at INTEGER NOT NULL,
              from_id TEXT NOT NULL,
              to_id TEXT NOT NULL,
              client_pubkey TEXT NOT NULL,
              peer_pubkey TEXT DEFAULT '',
              transcript_hash TEXT NOT NULL,
              verify_phrase TEXT NOT NULL,
              status TEXT NOT NULL,
              confirmed_by TEXT DEFAULT '',
              meta_json TEXT NOT NULL
            )
            """
        )
        pixel_chunk_columns = {row["name"] for row in conn.execute("PRAGMA table_info(twookie_pixel_chunks)").fetchall()}
        if "file_path" not in pixel_chunk_columns:
            conn.execute("ALTER TABLE twookie_pixel_chunks ADD COLUMN file_path TEXT DEFAULT ''")
        conn.execute(
            """
            UPDATE twookie_accounts
            SET hardware_log_json = '{"stored":"hash_only","legacy_redacted":true}'
            WHERE hardware_log_json IS NOT NULL
              AND hardware_log_json != ''
              AND hardware_log_json NOT LIKE '%"stored":"hash_only"%'
            """
        )
        current = now()
        retention = retention_seconds()
        cutoff = current - retention
        conn.execute("DELETE FROM twookie_bridge_messages WHERE created_at < ?", (cutoff,))
        conn.execute("DELETE FROM twookie_voice_memos WHERE created_at < ?", (cutoff,))
        conn.execute("DELETE FROM twookie_idempotency WHERE expires_at < ?", (current,))
        conn.execute("DELETE FROM twookie_oob_sessions WHERE expires_at < ?", (current,))
        expired_uploads = [row["id"] for row in conn.execute("SELECT id FROM twookie_pixel_uploads WHERE expires_at < ?", (current,)).fetchall()]
        if expired_uploads:
            placeholders = ",".join("?" for _ in expired_uploads)
            conn.execute(f"DELETE FROM twookie_pixel_upload_chunks WHERE upload_id IN ({placeholders})", expired_uploads)
            conn.execute(f"DELETE FROM twookie_pixel_uploads WHERE id IN ({placeholders})", expired_uploads)
            self.delete_pixel_upload_files(expired_uploads)
        old_payloads = [row["id"] for row in conn.execute("SELECT id FROM twookie_pixel_payloads WHERE created_at < ?", (cutoff,)).fetchall()]
        if old_payloads:
            placeholders = ",".join("?" for _ in old_payloads)
            conn.execute(f"DELETE FROM twookie_pixel_chunks WHERE payload_id IN ({placeholders})", old_payloads)
            conn.execute(f"DELETE FROM twookie_pixel_payloads WHERE id IN ({placeholders})", old_payloads)
            self.delete_pixel_payload_files(old_payloads)
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_auth_tokens (
              token TEXT PRIMARY KEY,
              account_id TEXT NOT NULL,
              created_at INTEGER NOT NULL
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_device_tokens (
              account_id TEXT PRIMARY KEY,
              device_token TEXT NOT NULL,
              bundle_id TEXT NOT NULL,
              updated_at INTEGER NOT NULL
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_seen (
              account_id TEXT NOT NULL,
              peer_id TEXT NOT NULL,
              seen_at INTEGER NOT NULL,
              PRIMARY KEY (account_id, peer_id)
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS twookie_flat_chunks (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              created_at INTEGER NOT NULL,
              from_id TEXT NOT NULL,
              to_id TEXT NOT NULL,
              chunk_index INTEGER NOT NULL,
              total_chunks INTEGER NOT NULL,
              iv TEXT NOT NULL,
              data TEXT NOT NULL,
              tag TEXT NOT NULL
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS request_log (
              id         INTEGER PRIMARY KEY AUTOINCREMENT,
              ts         INTEGER NOT NULL,
              ip         TEXT    NOT NULL,
              method     TEXT    NOT NULL,
              path       TEXT    NOT NULL,
              status     INTEGER,
              ua         TEXT,
              xff        TEXT,
              flag       TEXT
            )
            """
        )
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS ip_bans (
              ip         TEXT    PRIMARY KEY,
              reason     TEXT    NOT NULL,
              banned_at  INTEGER NOT NULL,
              auto       INTEGER NOT NULL DEFAULT 1,
              expires_at INTEGER
            )
            """
        )
        conn.execute("CREATE INDEX IF NOT EXISTS idx_reqlog_ip_ts ON request_log(ip, ts)")
        self.ensure_indexes(conn)

    def payloads_dir(self) -> Path:
        return self.db_path.parent / "payloads"

    def pixel_payload_dir(self, payload_id: str) -> Path:
        payload_id = clean_text(payload_id, 128)
        base = self.payloads_dir().resolve()
        target = (base / payload_id).resolve()
        if base not in target.parents and target != base:
            raise ValueError("invalid payload path")
        return target

    def pixel_upload_dir(self, upload_id: str) -> Path:
        upload_id = clean_text(upload_id, 128)
        base = (self.payloads_dir() / "uploads").resolve()
        target = (base / upload_id).resolve()
        if base not in target.parents and target != base:
            raise ValueError("invalid upload path")
        return target

    def delete_pixel_payload_files(self, payload_ids: list[str]) -> int:
        deleted = 0
        for payload_id in payload_ids:
            target = self.pixel_payload_dir(payload_id)
            if target.exists() and target.is_dir():
                rmtree(target)
                deleted += 1
        return deleted

    def delete_pixel_upload_files(self, upload_ids: list[str]) -> int:
        deleted = 0
        for upload_id in upload_ids:
            target = self.pixel_upload_dir(upload_id)
            if target.exists() and target.is_dir():
                rmtree(target)
                deleted += 1
        uploads_root = (self.payloads_dir() / "uploads").resolve()
        if uploads_root.exists() and not any(uploads_root.iterdir()):
            uploads_root.rmdir()
        return deleted

    def ensure_indexes(self, conn: sqlite3.Connection) -> None:
        indexes = [
            "CREATE INDEX IF NOT EXISTS idx_twookie_bridge_pair_id ON twookie_bridge_messages(from_id, to_id, id)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_bridge_created ON twookie_bridge_messages(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_bridge_status ON twookie_bridge_messages(status)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_bridge_direction_status ON twookie_bridge_messages(direction, status)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_voice_created ON twookie_voice_memos(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_voice_pair ON twookie_voice_memos(from_id, to_id)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_pixel_payload_created ON twookie_pixel_payloads(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_pixel_payload_pair ON twookie_pixel_payloads(from_id, to_id)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_pixel_chunks_payload_index ON twookie_pixel_chunks(payload_id, chunk_index)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_pixel_upload_expires ON twookie_pixel_uploads(expires_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_pixel_upload_status ON twookie_pixel_uploads(status)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_pixel_upload_chunks_upload_index ON twookie_pixel_upload_chunks(upload_id, chunk_index)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_invites_expires ON twookie_invites(expires_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_invites_inviter ON twookie_invites(inviter_id)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_accounts_seen ON twookie_accounts(last_seen)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_idempotency_expires ON twookie_idempotency(expires_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_oob_expires ON twookie_oob_sessions(expires_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_oob_pair ON twookie_oob_sessions(from_id, to_id)",
            "CREATE INDEX IF NOT EXISTS idx_image_nodes_created ON image_nodes(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_flat_chunks_to_id ON twookie_flat_chunks(to_id, created_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_flat_chunks_from_id ON twookie_flat_chunks(from_id, created_at)",
            "CREATE INDEX IF NOT EXISTS idx_twookie_auth_tokens_account ON twookie_auth_tokens(account_id)",
        ]
        for statement in indexes:
            conn.execute(statement)

    # ------------------------------------------------------------------
    # Security: request logging + IP banning
    # ------------------------------------------------------------------

    def log_request(self, ip: str, method: str, path: str, status: int,
                    ua: str, xff: str, flag: str | None) -> None:
        ts = int(time.time())
        with self.connect() as conn:
            conn.execute(
                "INSERT INTO request_log(ts,ip,method,path,status,ua,xff,flag) VALUES(?,?,?,?,?,?,?,?)",
                (ts, ip, method, path, status, ua[:512] if ua else None, xff[:256] if xff else None, flag),
            )

    def is_banned(self, ip: str) -> str | None:
        """Return ban reason if IP is banned, else None."""
        now = int(time.time())
        with self.connect() as conn:
            row = conn.execute(
                "SELECT reason, expires_at FROM ip_bans WHERE ip=?", (ip,)
            ).fetchone()
        if not row:
            return None
        if row["expires_at"] and row["expires_at"] < now:
            with self.connect() as conn:
                conn.execute("DELETE FROM ip_bans WHERE ip=?", (ip,))
            return None
        return row["reason"]

    def ban_ip(self, ip: str, reason: str, auto: bool = True,
               expires_at: int | None = None) -> None:
        now = int(time.time())
        with self.connect() as conn:
            conn.execute(
                "INSERT OR REPLACE INTO ip_bans(ip,reason,banned_at,auto,expires_at) VALUES(?,?,?,?,?)",
                (ip, reason, now, 1 if auto else 0, expires_at),
            )

    def unban_ip(self, ip: str) -> None:
        with self.connect() as conn:
            conn.execute("DELETE FROM ip_bans WHERE ip=?", (ip,))

    def check_auto_ban(self, ip: str) -> str | None:
        """Return auto-ban reason if thresholds exceeded, else None."""
        now = int(time.time())
        window = now - _AUTO_BAN_WINDOW_SECS
        with self.connect() as conn:
            recon_count = conn.execute(
                "SELECT COUNT(*) FROM request_log WHERE ip=? AND ts>=? AND flag='recon'",
                (ip, window),
            ).fetchone()[0]
            not_found_count = conn.execute(
                "SELECT COUNT(*) FROM request_log WHERE ip=? AND ts>=? AND status=404",
                (ip, window),
            ).fetchone()[0]
        if recon_count >= _AUTO_BAN_RECON_THRESHOLD:
            return f"auto: {recon_count} recon probes in {_AUTO_BAN_WINDOW_SECS}s"
        if not_found_count >= _AUTO_BAN_404_THRESHOLD:
            return f"auto: {not_found_count} 404s in {_AUTO_BAN_WINDOW_SECS}s"
        return None

    def security_report(self) -> dict:
        now = int(time.time())
        window_1h = now - 3600
        window_24h = now - 86400
        with self.connect() as conn:
            total_reqs = conn.execute("SELECT COUNT(*) FROM request_log").fetchone()[0]
            reqs_1h = conn.execute("SELECT COUNT(*) FROM request_log WHERE ts>=?", (window_1h,)).fetchone()[0]
            recon_24h = conn.execute(
                "SELECT COUNT(*) FROM request_log WHERE ts>=? AND flag='recon'", (window_24h,)
            ).fetchone()[0]
            top_ips = conn.execute(
                """SELECT ip, COUNT(*) as hits,
                          SUM(CASE WHEN flag='recon' THEN 1 ELSE 0 END) as recon_hits,
                          SUM(CASE WHEN status=404 THEN 1 ELSE 0 END) as not_found,
                          MAX(ts) as last_seen
                   FROM request_log WHERE ts>=?
                   GROUP BY ip ORDER BY hits DESC LIMIT 25""",
                (window_24h,),
            ).fetchall()
            recent_recon = conn.execute(
                """SELECT ts, ip, method, path, ua FROM request_log
                   WHERE flag='recon' ORDER BY ts DESC LIMIT 50""",
            ).fetchall()
            bans = conn.execute(
                "SELECT ip, reason, banned_at, auto, expires_at FROM ip_bans ORDER BY banned_at DESC"
            ).fetchall()
        return {
            "total_requests": total_reqs,
            "requests_1h": reqs_1h,
            "recon_24h": recon_24h,
            "top_ips": [dict(r) for r in top_ips],
            "recent_recon": [dict(r) for r in recent_recon],
            "bans": [dict(r) for r in bans],
        }

    def server_health(self) -> dict:
        db_size = self.db_path.stat().st_size if self.db_path.exists() else 0
        current = now()
        payload_dir = self.db_path.parent / "payloads"
        payload_dir_size = 0
        payload_file_count = 0
        if payload_dir.exists():
            for item in payload_dir.rglob("*"):
                if item.is_file():
                    payload_file_count += 1
                    payload_dir_size += item.stat().st_size
        with self.connect() as conn:
            counts = {
                "accounts": conn.execute("SELECT COUNT(*) FROM twookie_accounts").fetchone()[0],
                "invites": conn.execute("SELECT COUNT(*) FROM twookie_invites").fetchone()[0],
                "active_invites": conn.execute("SELECT COUNT(*) FROM twookie_invites WHERE revoked = 0 AND expires_at >= ?", (current,)).fetchone()[0],
                "expired_invites": conn.execute("SELECT COUNT(*) FROM twookie_invites WHERE revoked = 0 AND expires_at < ?", (current,)).fetchone()[0],
                "bridge_messages": conn.execute("SELECT COUNT(*) FROM twookie_bridge_messages").fetchone()[0],
                "pending_bridge_messages": conn.execute("SELECT COUNT(*) FROM twookie_bridge_messages WHERE status = 'pending'").fetchone()[0],
                "voice_memos": conn.execute("SELECT COUNT(*) FROM twookie_voice_memos WHERE revoked = 0").fetchone()[0],
                "pixel_payloads": conn.execute("SELECT COUNT(*) FROM twookie_pixel_payloads WHERE revoked = 0").fetchone()[0],
                "pixel_chunks": conn.execute("SELECT COUNT(*) FROM twookie_pixel_chunks").fetchone()[0],
                "legacy_pixel_chunks": conn.execute("SELECT COUNT(*) FROM twookie_pixel_chunks WHERE COALESCE(png_base64, '') != ''").fetchone()[0],
                "file_pixel_chunks": conn.execute("SELECT COUNT(*) FROM twookie_pixel_chunks WHERE COALESCE(file_path, '') != ''").fetchone()[0],
                "pixel_uploads_open": conn.execute("SELECT COUNT(*) FROM twookie_pixel_uploads WHERE status = 'open'").fetchone()[0],
                "pixel_upload_chunks": conn.execute("SELECT COUNT(*) FROM twookie_pixel_upload_chunks").fetchone()[0],
                "idempotency_keys": conn.execute("SELECT COUNT(*) FROM twookie_idempotency WHERE expires_at >= ?", (current,)).fetchone()[0],
                "oob_sessions_open": conn.execute("SELECT COUNT(*) FROM twookie_oob_sessions WHERE status != 'expired' AND expires_at >= ?", (current,)).fetchone()[0],
                "image_nodes": conn.execute("SELECT COUNT(*) FROM image_nodes").fetchone()[0],
            }
            orphan_chunks = conn.execute(
                """
                SELECT COUNT(*)
                FROM twookie_pixel_chunks c
                LEFT JOIN twookie_pixel_payloads p ON p.id = c.payload_id
                WHERE p.id IS NULL
                """
            ).fetchone()[0]
            largest_voice = conn.execute(
                "SELECT id, length(audio_base64) AS bytes_b64, duration_ms, mime_type FROM twookie_voice_memos ORDER BY length(audio_base64) DESC LIMIT 1"
            ).fetchone()
            largest_pixel = conn.execute(
                """
                SELECT p.id, p.chunk_count, p.duration_ms, p.mime_type, COALESCE(SUM(length(c.png_base64)), 0) AS bytes_b64
                FROM twookie_pixel_payloads p
                LEFT JOIN twookie_pixel_chunks c ON c.payload_id = p.id
                GROUP BY p.id
                ORDER BY bytes_b64 DESC LIMIT 1
                """
            ).fetchone()
            recent = conn.execute(
                """
                SELECT id, created_at, direction, from_id, to_id, status, source, body
                FROM twookie_bridge_messages
                ORDER BY id DESC LIMIT 5
                """
            ).fetchall()
            index_rows = conn.execute(
                """
                SELECT name, tbl_name
                FROM sqlite_master
                WHERE type = 'index' AND name LIKE 'idx_twookie_%'
                ORDER BY name
                """
            ).fetchall()
        return {
            "ok": True,
            "generated_at": current,
            "db_path": str(self.db_path),
            "storage": {
                "sqlite_bytes": db_size,
                "payload_dir": str(payload_dir),
                "payload_dir_exists": payload_dir.exists(),
                "payload_dir_bytes": payload_dir_size,
                "payload_dir_files": payload_file_count,
            },
            "counts": counts,
            "orphans": {"pixel_chunks": orphan_chunks},
            "largest": {
                "voice_memo": dict(largest_voice) if largest_voice else None,
                "pixel_payload": dict(largest_pixel) if largest_pixel else None,
            },
            "recent_bridge_messages": [
                {
                    "id": row["id"],
                    "created_at": row["created_at"],
                    "direction": row["direction"],
                    "from_id": row["from_id"],
                    "to_id": row["to_id"],
                    "status": row["status"],
                    "source": row["source"],
                    "body_preview": clean_text(row["body"], 80),
                }
                for row in recent
            ],
            "indexes": [dict(row) for row in index_rows],
            "retention_seconds": retention_seconds(),
        }

    def route_contract(self) -> dict:
        return {
            "ok": True,
            "stable": {
                "accounts": ["POST /api/twookie/accounts", "POST /api/twookie/touch"],
                "contacts": ["GET /api/twookie/contacts", "POST /api/twookie/contacts"],
                "invites": ["POST /api/twookie/invites", "GET /api/twookie/invites/{token}", "POST /api/twookie/invites/{token}/accept"],
                "messages": ["POST /api/twookie/messages", "GET /api/twookie/bridge/messages", "GET /api/twookie/bridge/recent"],
                "pixel_uploads": [
                    "POST /api/twookie/pixel-uploads",
                    "POST /api/twookie/pixel-uploads/{id}/chunks",
                    "POST /api/twookie/pixel-uploads/{id}/complete",
                ],
                "pixel_payloads": ["GET /api/twookie/pixel-payloads/{id}/manifest", "GET /api/twookie/pixel-payloads/{id}/chunks/{index}.png"],
                "oob": ["POST /api/twookie/oob/start", "POST /api/twookie/oob/confirm", "GET /api/twookie/oob/{id}"],
                "ops": ["GET /api/v1/server/health", "POST /api/v1/server/purge", "POST /api/v1/server/vacuum"],
            },
            "legacy_kept_for_compatibility": {
                "tbacon_demo": ["/tbacon", "/rooms/{room}", "/api/users", "/api/rooms"],
                "note": "These routes remain available so current demos do not break, but new iOS/web work should use the stable T-Wookie routes.",
            },
            "limits": {
                "json_bytes": MAX_JSON_BYTES,
                "text_message_bytes": MAX_TEXT_MESSAGE_BYTES,
                "pixel_chunks": MAX_PIXEL_CHUNKS,
                "pixel_chunk_png_bytes": MAX_PIXEL_CHUNK_PNG_BYTES,
                "voice_ms": 10000,
            },
        }

    def with_idempotency(self, scope: str, idempotency_key: str, request_data: dict, producer) -> dict:
        idempotency_key = safe_id(idempotency_key or "")
        if not idempotency_key:
            return producer()
        scope = clean_text(scope or "default", 96)
        request_hash = sha256_hex_text(idempotency_request_shape(request_data))
        current = now()
        with self.connect() as conn:
            row = conn.execute(
                """
                SELECT request_hash, result_json
                FROM twookie_idempotency
                WHERE scope = ? AND idempotency_key = ? AND expires_at >= ?
                """,
                (scope, idempotency_key, current),
            ).fetchone()
        if row:
            if row["request_hash"] != request_hash:
                raise ValueError("idempotency key reused with different request")
            result = json.loads(row["result_json"])
            result["idempotent_replay"] = True
            return result
        result = producer()
        stored = dict(result)
        stored.pop("idempotent_replay", None)
        with self.connect() as conn:
            try:
                conn.execute(
                    """
                    INSERT INTO twookie_idempotency
                      (scope, idempotency_key, created_at, expires_at, request_hash, result_json)
                    VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    (scope, idempotency_key, current, current + retention_seconds(), request_hash, canonical_json(stored)),
                )
            except sqlite3.IntegrityError:
                row = conn.execute(
                    "SELECT request_hash, result_json FROM twookie_idempotency WHERE scope = ? AND idempotency_key = ?",
                    (scope, idempotency_key),
                ).fetchone()
                if row and row["request_hash"] == request_hash:
                    replay = json.loads(row["result_json"])
                    replay["idempotent_replay"] = True
                    return replay
                raise ValueError("idempotency key reused with different request")
        return result

    def validate_png_base64(self, png_base64: str) -> bytes:
        text = str(png_base64 or "").strip()
        if not text:
            raise ValueError("png_base64 required")
        if len(text.encode("utf-8")) > MAX_PIXEL_CHUNK_PNG_BYTES:
            raise ValueError("pixel chunk png too large")
        try:
            png_bytes = base64.b64decode(text, validate=True)
        except Exception as exc:
            raise ValueError("invalid pixel chunk png") from exc
        if not png_bytes.startswith(b"\x89PNG\r\n\x1a\n"):
            raise ValueError("pixel chunk must be a PNG")
        if len(png_bytes) > MAX_PIXEL_CHUNK_PNG_BYTES:
            raise ValueError("pixel chunk png too large")
        return png_bytes

    def validate_pixel_chunks(self, chunks: list[dict], expected_count: int | None = None) -> list[tuple[int, bytes, str]]:
        if not isinstance(chunks, list) or not chunks:
            raise ValueError("pixel chunks required")
        if len(chunks) > MAX_PIXEL_CHUNKS:
            raise ValueError("too many pixel chunks")
        expected_count = expected_count if expected_count is not None else len(chunks)
        if expected_count < 1 or expected_count > MAX_PIXEL_CHUNKS:
            raise ValueError("invalid pixel chunk count")
        seen = set()
        out = []
        for item in chunks:
            if not isinstance(item, dict):
                raise ValueError("invalid pixel chunk")
            index = int(item.get("index") if item.get("index") is not None else -1)
            if index < 0 or index >= expected_count or index in seen:
                raise ValueError("invalid pixel chunk index")
            seen.add(index)
            chunk_hash = require_hex_hash(item.get("chunk_hash") or "", "chunk_hash", (64,))
            png_bytes = self.validate_png_base64(item.get("png_base64") or "")
            out.append((index, png_bytes, chunk_hash))
        if len(seen) != expected_count or seen != set(range(expected_count)):
            raise ValueError("pixel chunks must be contiguous")
        return sorted(out, key=lambda item: item[0])

    def validate_twookie_message_request(self, body: dict) -> dict:
        if not isinstance(body, dict):
            raise ValueError("json object required")
        message_kind = clean_text(body.get("kind") or "text", 32)
        if message_kind not in {"text", "voice", "pixel_voice"}:
            raise ValueError("invalid message kind")
        from_id = require_safe_identifier(body.get("from") or "", "from")
        to_id = require_safe_identifier(body.get("to") or "", "to")
        if message_kind == "text":
            message_body = require_text_field(body.get("body") or "", "body", MAX_TEXT_MESSAGE_BYTES)
        else:
            message_body = clean_text(body.get("body") or "", 120)
        return {"kind": message_kind, "from": from_id, "to": to_id, "body": message_body}

    def create_twookie_account(self, data: dict) -> dict:
        account_id = safe_id(data.get("id") or "")
        if not account_id:
            raise ValueError("account id required")
        display_name = clean_text(data.get("display_name") or account_id, 80)
        public_key = clean_text(data.get("public_key") or "", 2048)
        seed_hash = clean_text(data.get("seed_hash") or "", 128)
        hardware_log_hash = clean_text(data.get("hardware_log_hash") or "", 128)
        hardware_log = data.get("hardware_log") or {}
        if not public_key or not seed_hash or not hardware_log_hash:
            raise ValueError("public_key, seed_hash, and hardware_log_hash required")
        hardware_log_json = json.dumps(
            {
                "stored": "hash_only",
                "consent": bool(hardware_log.get("consent")),
                "source": clean_text(str(hardware_log.get("source") or "browser"), 40),
            },
            separators=(",", ":"),
            sort_keys=True,
        )
        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO twookie_accounts
                  (id, display_name, public_key, seed_hash, hardware_log_hash, hardware_log_json, created_at, last_seen)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                  display_name=excluded.display_name,
                  public_key=excluded.public_key,
                  seed_hash=excluded.seed_hash,
                  hardware_log_hash=excluded.hardware_log_hash,
                  hardware_log_json=excluded.hardware_log_json,
                  last_seen=excluded.last_seen,
                  revoked=0
                """,
                (account_id, display_name, public_key, seed_hash, hardware_log_hash, hardware_log_json, now(), now()),
            )
        auth_token = self._issue_auth_token(account_id)
        return {
            "account": {
                "id": account_id,
                "display_name": display_name,
                "public_key": public_key,
            },
            "auth_token": auth_token,
        }

    def _issue_auth_token(self, account_id: str) -> str:
        token = secrets.token_hex(32)
        with self.connect() as conn:
            conn.execute(
                "INSERT OR REPLACE INTO twookie_auth_tokens (token, account_id, created_at) VALUES (?, ?, ?)",
                (token, account_id, now()),
            )
        return token

    def claim_auth_token(self, account_id: str, seed_hash: str) -> str:
        account_id = safe_id(account_id)
        seed_hash = clean_text(seed_hash, 128)
        with self.connect() as conn:
            row = conn.execute(
                "SELECT id FROM twookie_accounts WHERE id = ? AND seed_hash = ?",
                (account_id, seed_hash),
            ).fetchone()
        if row is None:
            raise ValueError("invalid account or seed_hash mismatch")
        return self._issue_auth_token(account_id)

    def get_twookie_account_by_id(self, account_id: str) -> dict:
        account_id = safe_id(account_id)
        with self.connect() as conn:
            row = conn.execute(
                "SELECT id, display_name, public_key FROM twookie_accounts WHERE id = ? AND revoked = 0",
                (account_id,),
            ).fetchone()
        if row is None:
            raise ValueError("unknown account")
        return dict(row)

    def list_account_contacts(self, account_id: str) -> list[dict]:
        account_id = safe_id(account_id)
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT a.id, a.display_name, a.public_key
                FROM twookie_contacts c
                JOIN twookie_accounts a ON a.id = c.peer_id
                WHERE c.owner_id = ? AND a.revoked = 0
                ORDER BY c.created_at DESC
                """,
                (account_id,),
            ).fetchall()
        return [dict(row) for row in rows]

    def add_account_contact(self, account_id: str, contact_id: str) -> None:
        account_id = safe_id(account_id)
        contact_id = safe_id(contact_id)
        if not account_id or not contact_id or account_id == contact_id:
            raise ValueError("valid account_id and contact_id required")
        with self.connect() as conn:
            for uid in (account_id, contact_id):
                if not conn.execute("SELECT id FROM twookie_accounts WHERE id = ?", (uid,)).fetchone():
                    raise ValueError(f"unknown account: {uid}")
            conn.execute(
                "INSERT OR IGNORE INTO twookie_contacts (owner_id, peer_id, created_at) VALUES (?, ?, ?)",
                (account_id, contact_id, now()),
            )

    def update_account_display_name(self, account_id: str, display_name: str) -> dict:
        account_id = safe_id(account_id)
        display_name = clean_text(display_name, 80)
        if not display_name:
            raise ValueError("display_name required")
        with self.connect() as conn:
            conn.execute(
                "UPDATE twookie_accounts SET display_name = ? WHERE id = ?",
                (display_name, account_id),
            )
            row = conn.execute(
                "SELECT id, display_name, public_key FROM twookie_accounts WHERE id = ?",
                (account_id,),
            ).fetchone()
        if row is None:
            raise ValueError("unknown account")
        return dict(row)

    def list_account_messages(self, account_id: str, since: int = 0) -> list[dict]:
        account_id = safe_id(account_id)
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT id, from_id, to_id, body, created_at, status
                FROM twookie_bridge_messages
                WHERE (from_id = ? OR to_id = ?) AND created_at > ?
                ORDER BY created_at ASC
                """,
                (account_id, account_id, int(since)),
            ).fetchall()
        return [dict(row) for row in rows]

    def mark_seen(self, account_id: str, peer_id: str) -> None:
        account_id = safe_id(account_id)
        peer_id = safe_id(peer_id)
        with self.connect() as conn:
            conn.execute(
                "INSERT OR REPLACE INTO twookie_seen (account_id, peer_id, seen_at) VALUES (?, ?, ?)",
                (account_id, peer_id, now()),
            )
            # Also update message status to 'seen' for messages from peer to account
            conn.execute(
                "UPDATE twookie_bridge_messages SET status = 'seen' WHERE from_id = ? AND to_id = ? AND status != 'seen'",
                (peer_id, account_id),
            )

    def register_device_token(self, account_id: str, device_token: str, bundle_id: str) -> None:
        account_id = safe_id(account_id)
        device_token = clean_text(device_token, 256)
        bundle_id = clean_text(bundle_id, 128)
        with self.connect() as conn:
            conn.execute(
                "INSERT OR REPLACE INTO twookie_device_tokens (account_id, device_token, bundle_id, updated_at) VALUES (?, ?, ?, ?)",
                (account_id, device_token, bundle_id, now()),
            )

    def add_flat_chunk(self, payload: dict) -> None:
        from_id = safe_id(payload.get("fromId") or "")
        to_id = safe_id(payload.get("toId") or "")
        chunk_index = int(payload.get("chunkIndex") or 0)
        total_chunks = int(payload.get("totalChunks") or 1)
        iv = clean_text(payload.get("iv") or "", 128)
        data = clean_text(payload.get("data") or "", 4096)
        tag = clean_text(payload.get("tag") or "", 128)
        if not from_id or not to_id or not iv:
            raise ValueError("fromId, toId, iv required")
        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO twookie_flat_chunks
                  (created_at, from_id, to_id, chunk_index, total_chunks, iv, data, tag)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (now(), from_id, to_id, chunk_index, total_chunks, iv, data, tag),
            )
        # Notify recipient via WebSocket
        _ws_mgr.broadcast(to_id, {"type": "pixel_chunk", "from_id": from_id})

    def list_flat_chunks(self, to_id: str, since: int = 0) -> list[dict]:
        to_id = safe_id(to_id)
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT from_id, to_id, chunk_index, total_chunks, iv, data, tag
                FROM twookie_flat_chunks
                WHERE to_id = ? AND created_at > ?
                ORDER BY id ASC
                """,
                (to_id, int(since)),
            ).fetchall()
        return [
            {
                "fromId": row["from_id"],
                "toId": row["to_id"],
                "chunkIndex": row["chunk_index"],
                "totalChunks": row["total_chunks"],
                "iv": row["iv"],
                "data": row["data"],
                "tag": row["tag"],
            }
            for row in rows
        ]

    def push_apns(self, to_account_id: str, title: str = "T-Wookie", body: str = "New message") -> None:
        key_path = os.getenv("APNS_KEY_PATH", "")
        key_id = os.getenv("APNS_KEY_ID", "")
        team_id = os.getenv("APNS_TEAM_ID", "")
        if not all([key_path, key_id, team_id]):
            return
        with self.connect() as conn:
            row = conn.execute(
                "SELECT device_token, bundle_id FROM twookie_device_tokens WHERE account_id = ?",
                (to_account_id,),
            ).fetchone()
        if not row:
            return
        device_token = row["device_token"]
        bundle_id = row["bundle_id"]
        jwt_token = self._apns_jwt(key_path, key_id, team_id)
        if not jwt_token:
            return
        payload_json = json.dumps({
            "aps": {
                "alert": {"title": title, "body": body},
                "badge": 1,
                "sound": "default",
                "content-available": 1,
            }
        }, separators=(",", ":"))
        host = "api.push.apple.com" if os.getenv("APNS_PRODUCTION") else "api.sandbox.push.apple.com"
        url = f"https://{host}/3/device/{device_token}"
        headers = [
            f"authorization: bearer {jwt_token}",
            f"apns-topic: {bundle_id}",
            "apns-push-type: alert",
            "content-type: application/json",
        ]
        try:
            import httpx
            with httpx.Client(http2=True, timeout=5) as client:
                client.post(url, content=payload_json.encode(), headers={
                    "authorization": f"bearer {jwt_token}",
                    "apns-topic": bundle_id,
                    "apns-push-type": "alert",
                    "content-type": "application/json",
                })
        except ImportError:
            import subprocess
            args = ["curl", "--http2", "-s", "-o", os.devnull]
            for h in headers:
                args += ["-H", h]
            args += ["-d", payload_json, url]
            try:
                subprocess.run(args, timeout=5, capture_output=True)
            except Exception:
                pass
        except Exception:
            pass

    def _apns_jwt(self, key_path: str, key_id: str, team_id: str) -> str:
        try:
            import jwt as pyjwt  # PyJWT
            with open(key_path) as f:
                key = f.read()
            token = pyjwt.encode(
                {"iss": team_id, "iat": int(time.time())},
                key,
                algorithm="ES256",
                headers={"kid": key_id},
            )
            return token if isinstance(token, str) else token.decode()
        except Exception:
            return ""

    def list_twookie_accounts(self) -> list[dict]:
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT id, display_name, public_key, seed_hash, hardware_log_hash, created_at, last_seen, revoked
                FROM twookie_accounts ORDER BY created_at DESC
                """
            ).fetchall()
        return [dict(row) for row in rows]

    def touch_twookie_account(self, account_id: str) -> dict:
        account_id = safe_id(account_id)
        if not account_id:
            raise ValueError("account id required")
        with self.connect() as conn:
            conn.execute("UPDATE twookie_accounts SET last_seen = ? WHERE id = ?", (now(), account_id))
            row = conn.execute(
                "SELECT id, display_name, public_key, seed_hash, hardware_log_hash, created_at, last_seen, revoked FROM twookie_accounts WHERE id = ?",
                (account_id,),
            ).fetchone()
        if row is None:
            raise ValueError("unknown account")
        return dict(row)

    def add_twookie_contact(self, owner_id: str, peer_id: str) -> dict:
        owner_id = safe_id(owner_id)
        peer_id = safe_id(peer_id)
        if not owner_id or not peer_id or owner_id == peer_id:
            raise ValueError("valid owner and peer required")
        with self.connect() as conn:
            for user_id in (owner_id, peer_id):
                row = conn.execute("SELECT id FROM twookie_accounts WHERE id = ?", (user_id,)).fetchone()
                if row is None:
                    raise ValueError(f"unknown account: {user_id}")
            conn.execute(
                "INSERT OR IGNORE INTO twookie_contacts (owner_id, peer_id, created_at) VALUES (?, ?, ?)",
                (owner_id, peer_id, now()),
            )
        return {"owner_id": owner_id, "peer_id": peer_id}

    def list_twookie_contacts(self, owner_id: str | None = None) -> list[dict]:
        with self.connect() as conn:
            if owner_id:
                rows = conn.execute(
                    "SELECT owner_id, peer_id, created_at FROM twookie_contacts WHERE owner_id = ? ORDER BY created_at DESC",
                    (safe_id(owner_id),),
                ).fetchall()
            else:
                rows = conn.execute(
                    "SELECT owner_id, peer_id, created_at FROM twookie_contacts ORDER BY created_at DESC LIMIT 100"
                ).fetchall()
        return [dict(row) for row in rows]

    def create_twookie_invite(self, inviter_id: str, label: str = "", ttl_seconds: int = 86400) -> dict:
        inviter_id = safe_id(inviter_id)
        if not inviter_id:
            raise ValueError("inviter_id required")
        label = clean_text(label or "T-Wookie invite", 80)
        ttl_seconds = max(300, min(int(ttl_seconds or 86400), 7 * 86400))
        token = secrets.token_urlsafe(18).replace("-", "").replace("_", "")[:24]
        created_at = now()
        expires_at = created_at + ttl_seconds
        with self.connect() as conn:
            row = conn.execute("SELECT id FROM twookie_accounts WHERE id = ?", (inviter_id,)).fetchone()
            if row is None:
                raise ValueError(f"unknown inviter: {inviter_id}")
            conn.execute(
                "INSERT INTO twookie_invites (token, created_at, expires_at, inviter_id, label) VALUES (?, ?, ?, ?, ?)",
                (token, created_at, expires_at, inviter_id, label),
            )
        return {"token": token, "created_at": created_at, "expires_at": expires_at, "inviter_id": inviter_id, "label": label}

    def get_twookie_invite(self, token: str) -> dict:
        token = clean_text(token, 80)
        with self.connect() as conn:
            row = conn.execute(
                "SELECT token, created_at, expires_at, inviter_id, label, accepted_by, accepted_at, revoked FROM twookie_invites WHERE token = ?",
                (token,),
            ).fetchone()
        if row is None:
            raise ValueError("unknown invite")
        item = dict(row)
        item["expired"] = item["expires_at"] < now()
        return item

    def accept_twookie_invite(self, token: str, account_data: dict) -> dict:
        invite = self.get_twookie_invite(token)
        if invite["revoked"] or invite["expired"]:
            raise ValueError("invite expired or revoked")
        result = self.create_twookie_account(account_data)
        account = result["account"]
        auth_token = result["auth_token"]
        if account["id"] == invite["inviter_id"]:
            raise ValueError("invitee must be different from inviter")
        self.add_twookie_contact(invite["inviter_id"], account["id"])
        self.add_twookie_contact(account["id"], invite["inviter_id"])
        with self.connect() as conn:
            conn.execute(
                "UPDATE twookie_invites SET accepted_by = COALESCE(accepted_by, ?), accepted_at = COALESCE(accepted_at, ?) WHERE token = ?",
                (account["id"], now(), invite["token"]),
            )
        return {"invite": self.get_twookie_invite(token), "account": account, "peer_id": invite["inviter_id"], "auth_token": auth_token}

    def post_invite_scoped_message(self, token: str, sender_label: str, body: str) -> dict:
        invite = self.get_twookie_invite(token)
        if invite["revoked"] or invite["expired"]:
            raise ValueError("invite expired or revoked")
        body = clean_text(body or "", 600)
        if not body:
            raise ValueError("body required")
        sender_label = clean_text(sender_label or "ai", 40)
        sender_id = safe_id(f"ai-{sender_label}-{token[:6]}")
        if not sender_id:
            sender_id = safe_id(f"ai-guest-{token[:6]}")
        key_material = f"{token}|{sender_id}|ai-public-test"
        _acct_result = self.create_twookie_account(
            {
                "id": sender_id,
                "display_name": sender_label[:1].upper() + sender_label[1:],
                "public_key": hashlib.sha256(f"public|{key_material}".encode("utf-8")).hexdigest(),
                "seed_hash": hashlib.sha256(f"seed|{key_material}".encode("utf-8")).hexdigest(),
                "hardware_log_hash": hashlib.sha256(f"ai|{sender_label}|{token[:8]}".encode("utf-8")).hexdigest(),
                "hardware_log": {"consent": False, "source": "ai_invite_write"},
            }
        )
        account = _acct_result["account"]
        self.add_twookie_contact(invite["inviter_id"], account["id"])
        self.add_twookie_contact(account["id"], invite["inviter_id"])
        relay_from = account["id"]
        relay_to = invite["inviter_id"]
        if invite.get("accepted_by"):
            relay_from = invite["inviter_id"]
            relay_to = invite["accepted_by"]
        bridge = self.add_bridge_message(
            "web_local",
            relay_from,
            relay_to,
            body,
            "ai_public",
            {"kind": "ai_invite_test", "invite": token[:8], "ai_sender": account["id"]},
        )
        bridge = self.mark_bridge_delivered(bridge["id"], "delivered")
        return {"invite": invite, "account": account, "message": bridge, "routed": {"from": relay_from, "to": relay_to}}

    def add_twookie_call(self, caller_id: str, peer_id: str, mode: str) -> dict:
        caller_id = safe_id(caller_id)
        peer_id = safe_id(peer_id)
        mode = clean_text(mode or "signal", 32)
        with self.connect() as conn:
            cur = conn.execute(
                "INSERT INTO twookie_calls (created_at, caller_id, peer_id, mode, status) VALUES (?, ?, ?, ?, ?)",
                (now(), caller_id, peer_id, mode, "requested"),
            )
        return {"id": cur.lastrowid, "caller_id": caller_id, "peer_id": peer_id, "mode": mode, "status": "requested"}

    def add_image_node(self, payload_hash: str, image_path: str | None, meta: dict) -> dict:
        payload_hash = clean_text(payload_hash, 128)
        with self.connect() as conn:
            prev = conn.execute("SELECT id FROM image_nodes ORDER BY created_at DESC LIMIT 1").fetchone()
            prev_id = prev["id"] if prev else ""
            node_id = hashlib.sha256(f"{prev_id}|{payload_hash}|{now()}|{secrets.token_hex(8)}".encode("utf-8")).hexdigest()
            conn.execute(
                "INSERT INTO image_nodes (id, prev_id, payload_hash, image_path, created_at, meta_json) VALUES (?, ?, ?, ?, ?, ?)",
                (node_id, prev_id, payload_hash, image_path or "", now(), json.dumps(meta, separators=(",", ":"), sort_keys=True)),
            )
        return {"id": node_id, "prev_id": prev_id, "payload_hash": payload_hash, "image_path": image_path or "", "created_at": now()}

    def list_image_nodes(self) -> list[dict]:
        with self.connect() as conn:
            rows = conn.execute(
                "SELECT id, prev_id, payload_hash, image_path, created_at, meta_json FROM image_nodes ORDER BY created_at DESC LIMIT 50"
            ).fetchall()
        return [dict(row) for row in rows]

    def add_bridge_message(self, direction: str, from_id: str, to_id: str, body: str, source: str, meta: dict | None = None) -> dict:
        direction = clean_text(direction, 32)
        if direction not in {"web_to_discord", "discord_to_web", "web_local"}:
            raise ValueError("invalid bridge direction")
        from_id = safe_id(from_id)
        to_id = safe_id(to_id)
        body = clean_text(body, 1000)
        source = clean_text(source or "twookie", 32)
        if not from_id or not to_id or not body:
            raise ValueError("from_id, to_id, and body required")
        with self.connect() as conn:
            cur = conn.execute(
                """
                INSERT INTO twookie_bridge_messages
                  (created_at, direction, from_id, to_id, body, status, source, meta_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (now(), direction, from_id, to_id, body, "pending", source, json.dumps(meta or {}, separators=(",", ":"), sort_keys=True)),
            )
            message_id = cur.lastrowid
        return {"id": message_id, "direction": direction, "from_id": from_id, "to_id": to_id, "body": body, "status": "pending", "source": source, "meta_json": json.dumps(meta or {}, separators=(",", ":"), sort_keys=True)}

    def add_voice_memo(self, from_id: str, to_id: str, audio_base64: str, mime_type: str, duration_ms: int) -> dict:
        from_id = safe_id(from_id)
        to_id = safe_id(to_id)
        mime_type = clean_text(mime_type or "audio/webm", 80)
        duration_ms = max(1, min(int(duration_ms or 0), 10000))
        audio_base64 = clean_text(audio_base64 or "", 900_000)
        if not from_id or not to_id or not audio_base64:
            raise ValueError("from_id, to_id, and audio required")
        if len(audio_base64) > 900_000:
            raise ValueError("voice memo too large")
        if not (mime_type.startswith("audio/") or mime_type == "video/webm"):
            raise ValueError("unsupported voice mime type")
        try:
            base64.b64decode(audio_base64, validate=True)
        except Exception as exc:
            raise ValueError("invalid voice payload") from exc
        voice_id = hashlib.sha256(f"{from_id}|{to_id}|{now()}|{secrets.token_hex(16)}".encode("utf-8")).hexdigest()
        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO twookie_voice_memos
                  (id, created_at, from_id, to_id, mime_type, duration_ms, audio_base64)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (voice_id, now(), from_id, to_id, mime_type, duration_ms, audio_base64),
            )
        return {"id": voice_id, "from_id": from_id, "to_id": to_id, "mime_type": mime_type, "duration_ms": duration_ms}

    def get_voice_memo(self, voice_id: str) -> dict:
        voice_id = clean_text(voice_id or "", 128)
        with self.connect() as conn:
            row = conn.execute(
                """
                SELECT id, created_at, from_id, to_id, mime_type, duration_ms, audio_base64, revoked
                FROM twookie_voice_memos WHERE id = ?
                """,
                (voice_id,),
            ).fetchone()
        if row is None or row["revoked"]:
            raise ValueError("unknown voice memo")
        return dict(row)

    def add_pixel_payload(self, from_id: str, to_id: str, mime_type: str, duration_ms: int, chunks: list[dict], payload_hash: str, meta: dict | None = None) -> dict:
        from_id = require_safe_identifier(from_id, "from_id")
        to_id = require_safe_identifier(to_id, "to_id")
        mime_type = clean_text(mime_type or "audio/webm", 80)
        duration_ms = max(1, min(int(duration_ms or 0), 10000))
        payload_hash = require_hex_hash(payload_hash or "", "payload_hash", (64,))
        validated_chunks = self.validate_pixel_chunks(chunks)
        payload_id = hashlib.sha256(f"pixel|{from_id}|{to_id}|{payload_hash}|{now()}|{secrets.token_hex(16)}".encode("utf-8")).hexdigest()
        payload_dir = self.pixel_payload_dir(payload_id)
        payload_dir.mkdir(parents=True, exist_ok=True)
        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO twookie_pixel_payloads
                  (id, created_at, from_id, to_id, mime_type, duration_ms, algorithm, chunk_count, payload_hash, meta_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (payload_id, now(), from_id, to_id, mime_type, duration_ms, "tpxv-png-rgba-v1", len(validated_chunks), payload_hash, canonical_json(meta or {})),
            )
            for index, png_bytes, chunk_hash in validated_chunks:
                chunk_file = payload_dir / f"chunk_{index:04d}.png"
                chunk_file.write_bytes(png_bytes)
                rel_path = chunk_file.relative_to(self.db_path.parent).as_posix()
                conn.execute(
                    "INSERT INTO twookie_pixel_chunks (payload_id, chunk_index, png_base64, file_path, chunk_hash) VALUES (?, ?, ?, ?, ?)",
                    (payload_id, index, "", rel_path, chunk_hash),
                )
        return {"id": payload_id, "from_id": from_id, "to_id": to_id, "mime_type": mime_type, "duration_ms": duration_ms, "chunk_count": len(validated_chunks), "payload_hash": payload_hash, "algorithm": "tpxv-png-rgba-v1"}

    def create_pixel_upload(self, data: dict) -> dict:
        from_id = require_safe_identifier(data.get("from") or data.get("from_id") or "", "from")
        to_id = require_safe_identifier(data.get("to") or data.get("to_id") or "", "to")
        chunk_count = int(data.get("chunk_count") or 0)
        if chunk_count < 1 or chunk_count > MAX_PIXEL_CHUNKS:
            raise ValueError("invalid pixel chunk count")
        duration_ms = max(1, min(int(data.get("duration_ms") or 0), 10000))
        mime_type = clean_text(data.get("mime_type") or "audio/webm", 80)
        if not (mime_type.startswith("audio/") or mime_type == "video/webm"):
            raise ValueError("unsupported voice mime type")
        payload_hash = require_hex_hash(data.get("payload_hash") or "", "payload_hash", (64,))
        current = now()
        upload_id = hashlib.sha256(f"upload|{from_id}|{to_id}|{payload_hash}|{current}|{secrets.token_hex(16)}".encode("utf-8")).hexdigest()
        meta = {
            "cipher": clean_text(data.get("cipher") or "aes-256-gcm", 40),
            "nonce_b64": clean_text(data.get("nonce_b64") or "", 128),
            "width": int(data.get("width") or 96),
            "height": int(data.get("height") or 96),
            "client": clean_text(data.get("client") or "ios-web", 40),
        }
        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO twookie_pixel_uploads
                  (id, created_at, expires_at, from_id, to_id, mime_type, duration_ms, algorithm, chunk_count, payload_hash, meta_json, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (upload_id, current, current + 3600, from_id, to_id, mime_type, duration_ms, "tpxv-png-rgba-v1", chunk_count, payload_hash, canonical_json(meta), "open"),
            )
        self.pixel_upload_dir(upload_id).mkdir(parents=True, exist_ok=True)
        return {"id": upload_id, "from_id": from_id, "to_id": to_id, "chunk_count": chunk_count, "received_chunks": 0, "missing": list(range(chunk_count)), "expires_at": current + 3600, "status": "open"}

    def add_pixel_upload_chunk(self, upload_id: str, data: dict) -> dict:
        upload_id = clean_text(upload_id or "", 128)
        if not upload_id:
            raise ValueError("upload id required")
        index = int(data.get("index") if data.get("index") is not None else -1)
        chunk_hash = require_hex_hash(data.get("chunk_hash") or "", "chunk_hash", (64,))
        png_bytes = self.validate_png_base64(data.get("png_base64") or "")
        current = now()
        with self.connect() as conn:
            upload = conn.execute(
                "SELECT id, chunk_count, expires_at, status FROM twookie_pixel_uploads WHERE id = ?",
                (upload_id,),
            ).fetchone()
            if upload is None:
                raise ValueError("unknown pixel upload")
            if upload["status"] != "open" or upload["expires_at"] < current:
                raise ValueError("pixel upload is not open")
            if index < 0 or index >= int(upload["chunk_count"]):
                raise ValueError("invalid pixel chunk index")
            upload_dir = self.pixel_upload_dir(upload_id)
            upload_dir.mkdir(parents=True, exist_ok=True)
            chunk_file = upload_dir / f"chunk_{index:04d}.png"
            chunk_file.write_bytes(png_bytes)
            rel_path = chunk_file.relative_to(self.db_path.parent).as_posix()
            conn.execute(
                """
                INSERT INTO twookie_pixel_upload_chunks (upload_id, chunk_index, file_path, chunk_hash, created_at)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(upload_id, chunk_index) DO UPDATE SET
                  file_path=excluded.file_path,
                  chunk_hash=excluded.chunk_hash,
                  created_at=excluded.created_at
                """,
                (upload_id, index, rel_path, chunk_hash, current),
            )
            rows = conn.execute(
                "SELECT chunk_index FROM twookie_pixel_upload_chunks WHERE upload_id = ? ORDER BY chunk_index",
                (upload_id,),
            ).fetchall()
            received = {int(row["chunk_index"]) for row in rows}
        missing = [idx for idx in range(int(upload["chunk_count"])) if idx not in received]
        return {"id": upload_id, "received_chunks": len(received), "missing": missing, "complete": not missing}

    def get_pixel_upload(self, upload_id: str) -> dict:
        upload_id = clean_text(upload_id or "", 128)
        current = now()
        with self.connect() as conn:
            upload = conn.execute(
                "SELECT id, created_at, expires_at, from_id, to_id, mime_type, duration_ms, algorithm, chunk_count, payload_hash, status FROM twookie_pixel_uploads WHERE id = ?",
                (upload_id,),
            ).fetchone()
            if upload is None:
                raise ValueError("unknown pixel upload")
            rows = conn.execute("SELECT chunk_index FROM twookie_pixel_upload_chunks WHERE upload_id = ? ORDER BY chunk_index", (upload_id,)).fetchall()
        item = dict(upload)
        if item["expires_at"] < current and item["status"] == "open":
            item["status"] = "expired"
        received = {int(row["chunk_index"]) for row in rows}
        item["received_chunks"] = len(received)
        item["missing"] = [idx for idx in range(int(item["chunk_count"])) if idx not in received]
        return item

    def complete_pixel_upload(self, upload_id: str, data: dict) -> dict:
        upload_id = clean_text(upload_id or "", 128)
        if not upload_id:
            raise ValueError("upload id required")
        current = now()
        with self.connect() as conn:
            upload = conn.execute(
                """
                SELECT id, created_at, expires_at, from_id, to_id, mime_type, duration_ms, algorithm, chunk_count, payload_hash, meta_json, status
                FROM twookie_pixel_uploads WHERE id = ?
                """,
                (upload_id,),
            ).fetchone()
            if upload is None:
                raise ValueError("unknown pixel upload")
            existing_payload = conn.execute("SELECT id FROM twookie_pixel_payloads WHERE id = ?", (upload_id,)).fetchone()
            if existing_payload:
                payload = self.get_pixel_payload(upload_id)
                return {"payload": {key: payload[key] for key in ("id", "from_id", "to_id", "mime_type", "duration_ms", "chunk_count", "payload_hash", "algorithm")}, "bridge": None, "already_complete": True}
            if upload["status"] != "open" or upload["expires_at"] < current:
                raise ValueError("pixel upload is not open")
            rows = conn.execute(
                "SELECT chunk_index, file_path, chunk_hash FROM twookie_pixel_upload_chunks WHERE upload_id = ? ORDER BY chunk_index",
                (upload_id,),
            ).fetchall()
            received = {int(row["chunk_index"]) for row in rows}
            expected = set(range(int(upload["chunk_count"])))
            if received != expected:
                raise ValueError(f"pixel upload incomplete: missing {sorted(expected - received)}")
            final_dir = self.pixel_payload_dir(upload_id)
            final_dir.mkdir(parents=True, exist_ok=True)
            conn.execute(
                """
                INSERT INTO twookie_pixel_payloads
                  (id, created_at, from_id, to_id, mime_type, duration_ms, algorithm, chunk_count, payload_hash, meta_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (upload_id, current, upload["from_id"], upload["to_id"], upload["mime_type"], upload["duration_ms"], upload["algorithm"], upload["chunk_count"], upload["payload_hash"], upload["meta_json"]),
            )
            for row in rows:
                source = (self.db_path.parent / row["file_path"]).resolve()
                payload_root = self.payloads_dir().resolve()
                if payload_root not in source.parents:
                    raise ValueError("invalid upload chunk path")
                target = final_dir / f"chunk_{int(row['chunk_index']):04d}.png"
                if source != target:
                    copyfile(source, target)
                rel_path = target.relative_to(self.db_path.parent).as_posix()
                conn.execute(
                    "INSERT INTO twookie_pixel_chunks (payload_id, chunk_index, png_base64, file_path, chunk_hash) VALUES (?, ?, ?, ?, ?)",
                    (upload_id, int(row["chunk_index"]), "", rel_path, row["chunk_hash"]),
                )
            conn.execute("DELETE FROM twookie_pixel_upload_chunks WHERE upload_id = ?", (upload_id,))
            conn.execute("UPDATE twookie_pixel_uploads SET status = 'complete', meta_json = ? WHERE id = ?", (upload["meta_json"], upload_id))
        self.delete_pixel_upload_files([upload_id])
        meta = json.loads(upload["meta_json"] or "{}")
        payload = {"id": upload_id, "from_id": upload["from_id"], "to_id": upload["to_id"], "mime_type": upload["mime_type"], "duration_ms": upload["duration_ms"], "chunk_count": upload["chunk_count"], "payload_hash": upload["payload_hash"], "algorithm": upload["algorithm"]}
        node = self.add_image_node(upload["payload_hash"], None, {"kind": "pixel_upload_complete", "from": upload["from_id"], "to": upload["to_id"], "upload_id": upload_id})
        bridge_direction = "web_to_discord" if str(upload["to_id"]).startswith("discord-") else "web_local"
        bridge_source = "twookie_web" if bridge_direction == "web_to_discord" else "local_web"
        bridge = self.add_bridge_message(
            bridge_direction,
            upload["from_id"],
            upload["to_id"],
            clean_text(data.get("body") or "[pixel voice memo]", 120),
            bridge_source,
            {
                "node_id": node["id"],
                "kind": "pixel_voice_memo",
                "pixel_id": upload_id,
                "mime_type": upload["mime_type"],
                "duration_ms": upload["duration_ms"],
                "chunk_count": upload["chunk_count"],
                "payload_hash": upload["payload_hash"],
                "algorithm": upload["algorithm"],
                "cipher": meta.get("cipher") or "aes-256-gcm",
                "nonce_b64": meta.get("nonce_b64") or "",
            },
        )
        if bridge_direction == "web_local":
            bridge = self.mark_bridge_delivered(bridge["id"], "delivered")
        return {"payload": payload, "bridge": bridge, "node": node, "already_complete": False}

    def oob_phrase(self, digest_hex: str) -> str:
        words = [
            "amber", "violet", "cedar", "orbit", "signal", "north", "ember", "mirror",
            "copper", "atlas", "river", "quiet", "solar", "harbor", "pixel", "anchor",
        ]
        indexes = [int(digest_hex[i : i + 2], 16) % len(words) for i in range(0, 8, 2)]
        return "-".join(words[index] for index in indexes)

    def start_oob_session(self, data: dict) -> dict:
        from_id = require_safe_identifier(data.get("from") or data.get("from_id") or "", "from")
        to_id = require_safe_identifier(data.get("to") or data.get("to_id") or "", "to")
        client_pubkey = require_text_field(data.get("client_pubkey") or "", "client_pubkey", 4096)
        current = now()
        session_id = hashlib.sha256(f"oob|{from_id}|{to_id}|{client_pubkey}|{current}|{secrets.token_hex(16)}".encode("utf-8")).hexdigest()
        transcript = {"session_id": session_id, "from": from_id, "to": to_id, "client_pubkey_hash": hashlib.sha256(client_pubkey.encode("utf-8")).hexdigest(), "created_at": current}
        transcript_hash = sha256_hex_text(transcript)
        verify_phrase = self.oob_phrase(transcript_hash)
        meta = {
            "invite_token_hash": hashlib.sha256(clean_text(data.get("invite_token") or "", 120).encode("utf-8")).hexdigest() if data.get("invite_token") else "",
            "client": clean_text(data.get("client") or "unknown", 40),
            "purpose": clean_text(data.get("purpose") or "message_keys", 40),
        }
        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO twookie_oob_sessions
                  (id, created_at, expires_at, from_id, to_id, client_pubkey, transcript_hash, verify_phrase, status, meta_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (session_id, current, current + OOB_SESSION_SECONDS, from_id, to_id, client_pubkey, transcript_hash, verify_phrase, "pending", canonical_json(meta)),
            )
        return {"id": session_id, "from_id": from_id, "to_id": to_id, "expires_at": current + OOB_SESSION_SECONDS, "transcript_hash": transcript_hash, "verify_phrase": verify_phrase, "status": "pending"}

    def confirm_oob_session(self, data: dict) -> dict:
        session_id = clean_text(data.get("session_id") or "", 128)
        if not session_id:
            raise ValueError("session_id required")
        user_id = require_safe_identifier(data.get("user_id") or data.get("confirmed_by") or "", "user_id")
        peer_pubkey = require_text_field(data.get("peer_pubkey") or "", "peer_pubkey", 4096)
        transcript_hash = clean_text(data.get("transcript_hash") or "", 128)
        current = now()
        with self.connect() as conn:
            row = conn.execute(
                "SELECT id, expires_at, from_id, to_id, transcript_hash, verify_phrase, status, meta_json FROM twookie_oob_sessions WHERE id = ?",
                (session_id,),
            ).fetchone()
            if row is None:
                raise ValueError("unknown oob session")
            if row["expires_at"] < current:
                conn.execute("UPDATE twookie_oob_sessions SET status = 'expired' WHERE id = ?", (session_id,))
                raise ValueError("oob session expired")
            if transcript_hash and transcript_hash != row["transcript_hash"]:
                raise ValueError("oob transcript mismatch")
            if user_id not in {row["from_id"], row["to_id"]}:
                raise ValueError("user is not in oob session")
            conn.execute(
                "UPDATE twookie_oob_sessions SET peer_pubkey = ?, status = 'confirmed', confirmed_by = ? WHERE id = ?",
                (peer_pubkey, user_id, session_id),
            )
        return {"id": session_id, "from_id": row["from_id"], "to_id": row["to_id"], "transcript_hash": row["transcript_hash"], "verify_phrase": row["verify_phrase"], "status": "confirmed", "confirmed_by": user_id}

    def get_oob_session(self, session_id: str) -> dict:
        session_id = clean_text(session_id or "", 128)
        current = now()
        with self.connect() as conn:
            row = conn.execute(
                """
                SELECT id, created_at, expires_at, from_id, to_id, transcript_hash, verify_phrase, status, confirmed_by, meta_json
                FROM twookie_oob_sessions WHERE id = ?
                """,
                (session_id,),
            ).fetchone()
            if row is None:
                raise ValueError("unknown oob session")
            item = dict(row)
            if item["expires_at"] < current and item["status"] != "expired":
                conn.execute("UPDATE twookie_oob_sessions SET status = 'expired' WHERE id = ?", (session_id,))
                item["status"] = "expired"
        return item

    def get_pixel_payload(self, payload_id: str) -> dict:
        payload_id = clean_text(payload_id or "", 128)
        with self.connect() as conn:
            payload = conn.execute(
                """
                SELECT id, created_at, from_id, to_id, mime_type, duration_ms, algorithm, chunk_count, payload_hash, meta_json, revoked
                FROM twookie_pixel_payloads WHERE id = ?
                """,
                (payload_id,),
            ).fetchone()
            if payload is None or payload["revoked"]:
                raise ValueError("unknown pixel payload")
            chunks = conn.execute(
                "SELECT chunk_index, png_base64, file_path, chunk_hash FROM twookie_pixel_chunks WHERE payload_id = ? ORDER BY chunk_index",
                (payload_id,),
            ).fetchall()
        item = dict(payload)
        item["chunks"] = [dict(row) for row in chunks]
        return item

    def purge_bridge_conversation(self, owner_id: str, peer_id: str) -> dict:
        owner_id = safe_id(owner_id)
        peer_id = safe_id(peer_id)
        if not owner_id or not peer_id:
            raise ValueError("owner and peer required")
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT meta_json FROM twookie_bridge_messages
                WHERE (from_id = ? AND to_id = ?) OR (from_id = ? AND to_id = ?)
                """,
                (owner_id, peer_id, peer_id, owner_id),
            ).fetchall()
            voice_ids = [parse_meta(row).get("voice_id") for row in rows]
            voice_ids = [voice_id for voice_id in voice_ids if voice_id]
            pixel_ids = [parse_meta(row).get("pixel_id") for row in rows]
            pixel_ids = [pixel_id for pixel_id in pixel_ids if pixel_id]
            node_ids = [parse_meta(row).get("node_id") for row in rows]
            node_ids = [node_id for node_id in node_ids if node_id]
            deleted_messages = conn.execute(
                """
                DELETE FROM twookie_bridge_messages
                WHERE (from_id = ? AND to_id = ?) OR (from_id = ? AND to_id = ?)
                """,
                (owner_id, peer_id, peer_id, owner_id),
            ).rowcount
            deleted_nodes = 0
            if node_ids:
                placeholders = ",".join("?" for _ in node_ids)
                deleted_nodes = conn.execute(f"DELETE FROM image_nodes WHERE id IN ({placeholders})", node_ids).rowcount
            deleted_voice = 0
            if voice_ids:
                placeholders = ",".join("?" for _ in voice_ids)
                deleted_voice = conn.execute(f"DELETE FROM twookie_voice_memos WHERE id IN ({placeholders})", voice_ids).rowcount
            deleted_pixel = 0
            deleted_pixel_files = 0
            if pixel_ids:
                placeholders = ",".join("?" for _ in pixel_ids)
                conn.execute(f"DELETE FROM twookie_pixel_chunks WHERE payload_id IN ({placeholders})", pixel_ids)
                deleted_pixel = conn.execute(f"DELETE FROM twookie_pixel_payloads WHERE id IN ({placeholders})", pixel_ids).rowcount
                deleted_pixel_files = self.delete_pixel_payload_files(pixel_ids)
        return {"deleted_messages": deleted_messages, "deleted_nodes": deleted_nodes, "deleted_voice": deleted_voice, "deleted_pixel": deleted_pixel, "deleted_pixel_files": deleted_pixel_files}

    def purge_server(self, dry_run: bool = True) -> dict:
        current = now()
        retention = retention_seconds()
        cutoff = current - retention
        with self.connect() as conn:
            expired_invites = [row["token"] for row in conn.execute("SELECT token FROM twookie_invites WHERE revoked = 0 AND expires_at < ?", (current,)).fetchall()]
            old_bridge_ids = [row["id"] for row in conn.execute("SELECT id FROM twookie_bridge_messages WHERE created_at < ?", (cutoff,)).fetchall()]
            old_voice_ids = [row["id"] for row in conn.execute("SELECT id FROM twookie_voice_memos WHERE created_at < ?", (cutoff,)).fetchall()]
            old_pixel_ids = [row["id"] for row in conn.execute("SELECT id FROM twookie_pixel_payloads WHERE created_at < ?", (cutoff,)).fetchall()]
            expired_upload_ids = [row["id"] for row in conn.execute("SELECT id FROM twookie_pixel_uploads WHERE expires_at < ?", (current,)).fetchall()]
            expired_idempotency = [row["idempotency_key"] for row in conn.execute("SELECT idempotency_key FROM twookie_idempotency WHERE expires_at < ?", (current,)).fetchall()]
            expired_oob = [row["id"] for row in conn.execute("SELECT id FROM twookie_oob_sessions WHERE expires_at < ?", (current,)).fetchall()]
            orphan_chunks = conn.execute(
                """
                SELECT c.payload_id, c.chunk_index
                FROM twookie_pixel_chunks c
                LEFT JOIN twookie_pixel_payloads p ON p.id = c.payload_id
                WHERE p.id IS NULL
                """
            ).fetchall()
            result = {
                "dry_run": dry_run,
                "cutoff": cutoff,
                "retention_seconds": retention,
                "expired_invites": len(expired_invites),
                "old_bridge_messages": len(old_bridge_ids),
                "old_voice_memos": len(old_voice_ids),
                "old_pixel_payloads": len(old_pixel_ids),
                "expired_pixel_uploads": len(expired_upload_ids),
                "expired_idempotency_keys": len(expired_idempotency),
                "expired_oob_sessions": len(expired_oob),
                "orphan_pixel_chunks": len(orphan_chunks),
                "deleted": {
                    "invites": 0,
                    "bridge_messages": 0,
                    "voice_memos": 0,
                    "pixel_payloads": 0,
                    "pixel_chunks": 0,
                    "pixel_payload_dirs": 0,
                    "pixel_uploads": 0,
                    "pixel_upload_chunks": 0,
                    "pixel_upload_dirs": 0,
                    "image_nodes": 0,
                    "idempotency_keys": 0,
                    "oob_sessions": 0,
                    "orphan_pixel_chunks": 0,
                },
            }
            if dry_run:
                return result
            if expired_invites:
                placeholders = ",".join("?" for _ in expired_invites)
                result["deleted"]["invites"] = conn.execute(f"DELETE FROM twookie_invites WHERE token IN ({placeholders})", expired_invites).rowcount
            if old_bridge_ids:
                placeholders = ",".join("?" for _ in old_bridge_ids)
                bridge_rows = conn.execute(f"SELECT meta_json FROM twookie_bridge_messages WHERE id IN ({placeholders})", old_bridge_ids).fetchall()
                old_node_ids = [parse_meta(row).get("node_id") for row in bridge_rows]
                old_node_ids = [node_id for node_id in old_node_ids if node_id]
                if old_node_ids:
                    node_placeholders = ",".join("?" for _ in old_node_ids)
                    result["deleted"]["image_nodes"] += conn.execute(f"DELETE FROM image_nodes WHERE id IN ({node_placeholders})", old_node_ids).rowcount
                result["deleted"]["bridge_messages"] = conn.execute(f"DELETE FROM twookie_bridge_messages WHERE id IN ({placeholders})", old_bridge_ids).rowcount
            if old_voice_ids:
                placeholders = ",".join("?" for _ in old_voice_ids)
                result["deleted"]["voice_memos"] = conn.execute(f"DELETE FROM twookie_voice_memos WHERE id IN ({placeholders})", old_voice_ids).rowcount
            if old_pixel_ids:
                placeholders = ",".join("?" for _ in old_pixel_ids)
                result["deleted"]["pixel_chunks"] = conn.execute(f"DELETE FROM twookie_pixel_chunks WHERE payload_id IN ({placeholders})", old_pixel_ids).rowcount
                result["deleted"]["pixel_payloads"] = conn.execute(f"DELETE FROM twookie_pixel_payloads WHERE id IN ({placeholders})", old_pixel_ids).rowcount
                result["deleted"]["pixel_payload_dirs"] = self.delete_pixel_payload_files(old_pixel_ids)
            if expired_upload_ids:
                placeholders = ",".join("?" for _ in expired_upload_ids)
                result["deleted"]["pixel_upload_chunks"] = conn.execute(f"DELETE FROM twookie_pixel_upload_chunks WHERE upload_id IN ({placeholders})", expired_upload_ids).rowcount
                result["deleted"]["pixel_uploads"] = conn.execute(f"DELETE FROM twookie_pixel_uploads WHERE id IN ({placeholders})", expired_upload_ids).rowcount
                result["deleted"]["pixel_upload_dirs"] = self.delete_pixel_upload_files(expired_upload_ids)
            result["deleted"]["idempotency_keys"] = conn.execute("DELETE FROM twookie_idempotency WHERE expires_at < ?", (current,)).rowcount
            result["deleted"]["oob_sessions"] = conn.execute("DELETE FROM twookie_oob_sessions WHERE expires_at < ?", (current,)).rowcount
            if orphan_chunks:
                result["deleted"]["orphan_pixel_chunks"] = conn.execute(
                    """
                    DELETE FROM twookie_pixel_chunks
                    WHERE NOT EXISTS (
                      SELECT 1 FROM twookie_pixel_payloads p WHERE p.id = twookie_pixel_chunks.payload_id
                    )
                    """
                ).rowcount
        return result

    def migrate_legacy_pixel_chunks(self, dry_run: bool = True, limit: int = 500) -> dict:
        limit = max(1, min(int(limit or 500), 5000))
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT payload_id, chunk_index, png_base64
                FROM twookie_pixel_chunks
                WHERE COALESCE(png_base64, '') != ''
                ORDER BY payload_id, chunk_index
                LIMIT ?
                """,
                (limit,),
            ).fetchall()
            total_remaining = conn.execute(
                "SELECT COUNT(*) FROM twookie_pixel_chunks WHERE COALESCE(png_base64, '') != ''"
            ).fetchone()[0]
            result = {
                "dry_run": dry_run,
                "selected": len(rows),
                "legacy_remaining_before": total_remaining,
                "migrated": 0,
                "bytes_written": 0,
                "errors": [],
            }
            if dry_run:
                result["bytes_to_write"] = sum(len(row["png_base64"] or "") for row in rows)
                return result
            for row in rows:
                payload_id = row["payload_id"]
                chunk_index = int(row["chunk_index"])
                try:
                    png_bytes = base64.b64decode(row["png_base64"], validate=True)
                    chunk_dir = self.pixel_payload_dir(payload_id)
                    chunk_dir.mkdir(parents=True, exist_ok=True)
                    chunk_file = chunk_dir / f"chunk_{chunk_index:04d}.png"
                    chunk_file.write_bytes(png_bytes)
                    rel_path = chunk_file.relative_to(self.db_path.parent).as_posix()
                    conn.execute(
                        """
                        UPDATE twookie_pixel_chunks
                        SET png_base64 = '', file_path = ?
                        WHERE payload_id = ? AND chunk_index = ?
                        """,
                        (rel_path, payload_id, chunk_index),
                    )
                    result["migrated"] += 1
                    result["bytes_written"] += len(png_bytes)
                except Exception as exc:
                    result["errors"].append({"payload_id": payload_id, "chunk_index": chunk_index, "error": str(exc)})
            result["legacy_remaining_after"] = conn.execute(
                "SELECT COUNT(*) FROM twookie_pixel_chunks WHERE COALESCE(png_base64, '') != ''"
            ).fetchone()[0]
        return result

    def vacuum_database(self) -> dict:
        before = self.db_path.stat().st_size if self.db_path.exists() else 0
        with self.connect() as conn:
            conn.execute("VACUUM")
        after = self.db_path.stat().st_size if self.db_path.exists() else 0
        return {"before_bytes": before, "after_bytes": after, "reclaimed_bytes": max(0, before - after)}

    def list_bridge_messages(self, owner_id: str, peer_id: str, after: int = 0) -> list[dict]:
        owner_id = safe_id(owner_id)
        peer_id = safe_id(peer_id)
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT id, created_at, direction, from_id, to_id, body, status, source, meta_json
                FROM twookie_bridge_messages
                WHERE id > ?
                  AND ((from_id = ? AND to_id = ?) OR (from_id = ? AND to_id = ?))
                ORDER BY id LIMIT 100
                """,
                (after, owner_id, peer_id, peer_id, owner_id),
            ).fetchall()
        return [dict(row) for row in rows]

    def list_bridge_recent(self, owner_id: str, after: int = 0) -> list[dict]:
        owner_id = safe_id(owner_id)
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT id, created_at, direction, from_id, to_id, body, status, source, meta_json
                FROM twookie_bridge_messages
                WHERE id > ? AND (from_id = ? OR to_id = ?)
                ORDER BY id LIMIT 200
                """,
                (after, owner_id, owner_id),
            ).fetchall()
        return [dict(row) for row in rows]

    def claim_bridge_outbox(self, limit: int = 10) -> list[dict]:
        limit = max(1, min(int(limit or 10), 25))
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT id, created_at, direction, from_id, to_id, body, status, source, meta_json
                FROM twookie_bridge_messages
                WHERE direction = 'web_to_discord' AND status = 'pending'
                ORDER BY id LIMIT ?
                """,
                (limit,),
            ).fetchall()
            ids = [row["id"] for row in rows]
            if ids:
                placeholders = ",".join("?" for _ in ids)
                conn.execute(f"UPDATE twookie_bridge_messages SET status = 'claimed' WHERE id IN ({placeholders})", ids)
        return [dict(row) for row in rows]

    def mark_bridge_delivered(self, message_id: int, status: str = "delivered") -> dict:
        status = clean_text(status, 32)
        if status not in {"delivered", "failed"}:
            raise ValueError("invalid bridge status")
        with self.connect() as conn:
            conn.execute("UPDATE twookie_bridge_messages SET status = ? WHERE id = ?", (status, int(message_id)))
            row = conn.execute(
                "SELECT id, created_at, direction, from_id, to_id, body, status, source, meta_json FROM twookie_bridge_messages WHERE id = ?",
                (int(message_id),),
            ).fetchone()
        if row is None:
            raise ValueError("unknown bridge message")
        return dict(row)

    def ensure_discord_account(self, discord_user_id: str, display_name: str) -> dict:
        account_id = f"discord-{safe_id(discord_user_id)}"
        display_name = clean_text(display_name or account_id, 80)
        public_key = hashlib.sha256(f"discord-public|{account_id}".encode("utf-8")).hexdigest()
        seed_hash = hashlib.sha256(f"discord-seed-placeholder|{account_id}".encode("utf-8")).hexdigest()
        hardware_log_hash = hashlib.sha256(f"discord-bridge|{account_id}".encode("utf-8")).hexdigest()
        result = self.create_twookie_account(
            {
                "id": account_id,
                "display_name": display_name,
                "public_key": public_key,
                "seed_hash": seed_hash,
                "hardware_log_hash": hardware_log_hash,
                "hardware_log": {"source": "discord_bridge", "discord_user_id": discord_user_id},
            }
        )
        return result["account"]

    def create_user(self, user_id: str, name: str, conn: sqlite3.Connection | None = None) -> dict:
        own_conn = conn is None
        if conn is None:
            conn = self.connect()
        try:
            secret = secrets.token_hex(32)
            conn.execute(
                "INSERT INTO users (id, name, secret, created_at) VALUES (?, ?, ?, ?)",
                (user_id, name, secret, now()),
            )
            if own_conn:
                conn.commit()
            return {"id": user_id, "name": name, "secret": secret}
        finally:
            if own_conn:
                conn.close()

    def rotate_user(self, user_id: str) -> dict:
        secret = secrets.token_hex(32)
        with self.connect() as conn:
            conn.execute("UPDATE users SET secret = ?, revoked = 0 WHERE id = ?", (secret, user_id))
            user = conn.execute("SELECT id, name, secret FROM users WHERE id = ?", (user_id,)).fetchone()
            if user is None:
                raise ValueError("unknown user")
            return dict(user)

    def list_users(self) -> list[dict]:
        cutoff = now() - 45
        with self.connect() as conn:
            rows = conn.execute(
                "SELECT id, name, created_at, last_seen, level, revoked FROM users ORDER BY id"
            ).fetchall()
        out = []
        for row in rows:
            item = dict(row)
            item["online"] = bool(item["last_seen"] and item["last_seen"] >= cutoff)
            out.append(item)
        return out

    def list_rooms(self) -> list[dict]:
        with self.connect() as conn:
            rooms = conn.execute("SELECT id, name, created_at FROM rooms ORDER BY id").fetchall()
            out = []
            for room in rooms:
                members = conn.execute(
                    "SELECT user_id FROM room_members WHERE room_id = ? ORDER BY user_id",
                    (room["id"],),
                ).fetchall()
                item = dict(room)
                item["members"] = [member["user_id"] for member in members]
                out.append(item)
            return out

    def create_room(self, room_id: str, name: str) -> dict:
        with self.connect() as conn:
            conn.execute("INSERT INTO rooms (id, name, created_at) VALUES (?, ?, ?)", (room_id, name, now()))
        return {"id": room_id, "name": name}

    def add_member(self, room_id: str, user_id: str) -> None:
        with self.connect() as conn:
            conn.execute("INSERT OR IGNORE INTO room_members (room_id, user_id) VALUES (?, ?)", (room_id, user_id))

    def add_message(self, room_id: str, user_id: str, body: str) -> dict:
        with self.connect() as conn:
            cur = conn.execute(
                "INSERT INTO messages (created_at, room_id, user_id, body, transport) VALUES (?, ?, ?, ?, ?)",
                (now(), room_id, user_id, body, "text"),
            )
            return {"id": cur.lastrowid, "room_id": room_id, "user_id": user_id, "body": body}

    def add_pixel_message(self, room_id: str, user_id: str, encoded_png: str) -> dict:
        if not encoded_png.startswith("data:image/png;base64,"):
            raise ValueError("encodedPng must be a PNG data URL")
        with self.connect() as conn:
            cur = conn.execute(
                "INSERT INTO messages (created_at, room_id, user_id, body, transport) VALUES (?, ?, ?, ?, ?)",
                (now(), room_id, user_id, encoded_png, "tpxv_png"),
            )
            return {"id": cur.lastrowid, "room_id": room_id, "user_id": user_id, "transport": "tpxv_png"}

    def list_messages(self, room_id: str, after: int) -> list[dict]:
        with self.connect() as conn:
            rows = conn.execute(
                "SELECT id, created_at, room_id, user_id, body, transport FROM messages WHERE room_id = ? AND id > ? ORDER BY id LIMIT 100",
                (room_id, after),
            ).fetchall()
            messages = []
            for row in rows:
                item = dict(row)
                if item["transport"] == "tpxv_png":
                    body = item.pop("body")
                    item["payload_url"] = f"/api/messages/{item['id']}/payload"
                    item["payload_size"] = len(body)
                    item["body_preview"] = body[:48] + "..." if len(body) > 48 else body
                messages.append(item)
            return messages

    def get_message_payload(self, message_id: int) -> dict | None:
        with self.connect() as conn:
            row = conn.execute(
                "SELECT id, body, transport FROM messages WHERE id = ?",
                (message_id,),
            ).fetchone()
            return dict(row) if row else None

    def set_room_carrier(self, room_id: str, mask_json: dict, carrier_frame_path: str) -> dict:
        with self.connect() as conn:
            conn.execute(
                "INSERT OR REPLACE INTO room_carriers (room_id, mask_id, mask_json, carrier_frame_path, updated_at) VALUES (?, ?, ?, ?, ?)",
                (room_id, mask_json["mask_id"], json.dumps(mask_json), carrier_frame_path, now()),
            )
        return {"room_id": room_id, "mask_id": mask_json["mask_id"], "carrier_frame_path": carrier_frame_path}

    def get_room_carrier(self, room_id: str) -> dict | None:
        with self.connect() as conn:
            row = conn.execute("SELECT room_id, mask_id, mask_json, carrier_frame_path, updated_at FROM room_carriers WHERE room_id = ?", (room_id,)).fetchone()
            if row is None:
                return None
            out = dict(row)
            out["mask"] = json.loads(out.pop("mask_json"))
            return out

    def list_events(self, limit: int = 100) -> list[dict]:
        with self.connect() as conn:
            rows = conn.execute(
                "SELECT id, created_at, user_id, sid, op, seq, nonce, level, ok, error FROM events ORDER BY id DESC LIMIT ?",
                (limit,),
            ).fetchall()
            return [dict(row) for row in rows]

    def validate_beacon(self, op: str, params: dict[str, str]) -> dict:
        event = {
            "created_at": now(),
            "user_id": params.get("user", ""),
            "sid": params.get("sid", ""),
            "op": op,
            "seq": None,
            "nonce": params.get("nonce", ""),
            "level": None,
            "ok": False,
            "error": None,
        }
        required = ["sid", "user", "seq", "nonce", "ts", "mac"]
        if op not in ALLOWED_OPS:
            event["error"] = "op not allowed"
            return self.store_event(event)
        if any(not params.get(key) for key in required):
            event["error"] = "missing parameter"
            return self.store_event(event)
        try:
            seq = int(params["seq"])
            ts = int(params["ts"])
        except ValueError:
            event["error"] = "bad seq or ts"
            return self.store_event(event)
        event["seq"] = seq
        if abs(now() - ts) > MAX_SKEW_SECONDS:
            event["error"] = "timestamp outside window"
            return self.store_event(event)

        with self.connect() as conn:
            user = conn.execute("SELECT id, secret, revoked FROM users WHERE id = ?", (params["user"],)).fetchone()
            if user is None or user["revoked"]:
                event["error"] = "unknown or revoked user"
                return self.store_event(event, conn)
            replay_key = "|".join([params["sid"], params["user"], params["seq"], params["nonce"]])
            exists = conn.execute("SELECT replay_key FROM replay WHERE replay_key = ?", (replay_key,)).fetchone()
            if exists is not None:
                event["error"] = "replay"
                return self.store_event(event, conn)
            expected = sign(user["secret"], params["sid"], params["user"], op, params["seq"], params["nonce"], params["ts"])
            if not hmac.compare_digest(expected, params["mac"]):
                event["error"] = "bad mac"
                return self.store_event(event, conn)

            level = op.upper() if op.startswith("cap_") else None
            event["level"] = level
            event["ok"] = True
            conn.execute("INSERT INTO replay (replay_key, created_at) VALUES (?, ?)", (replay_key, now()))
            conn.execute("UPDATE users SET last_seen = ?, level = COALESCE(?, level) WHERE id = ?", (now(), level, params["user"]))
            return self.store_event(event, conn)

    def store_event(self, event: dict, conn: sqlite3.Connection | None = None) -> dict:
        own_conn = conn is None
        if conn is None:
            conn = self.connect()
        try:
            cur = conn.execute(
                "INSERT INTO events (created_at, user_id, sid, op, seq, nonce, level, ok, error) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    event["created_at"],
                    event["user_id"],
                    event["sid"],
                    event["op"],
                    event["seq"],
                    event["nonce"],
                    event["level"],
                    1 if event["ok"] else 0,
                    event["error"],
                ),
            )
            event["id"] = cur.lastrowid
            if own_conn:
                conn.commit()
            return event
        finally:
            if own_conn:
                conn.close()


def dashboard() -> str:
    return """<!doctype html>
<html lang="fr">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>T-Bacon Hub</title>
<style>
:root { color-scheme: dark; font-family: Inter, Segoe UI, system-ui, sans-serif; background: #101417; color: #edf4f7; }
body { margin: 0; background: #101417; }
main { max-width: 1180px; margin: auto; padding: 24px; }
header { display: flex; justify-content: space-between; align-items: end; border-bottom: 1px solid #2d3940; padding-bottom: 16px; }
h1 { margin: 0; font-size: 30px; letter-spacing: 0; }
.status { border: 1px solid #537a6e; padding: 8px 12px; color: #a5f2d1; font-weight: 700; }
.grid { display: grid; grid-template-columns: repeat(2, minmax(320px, 1fr)); gap: 18px; margin-top: 18px; }
section { border: 1px solid #2d3940; background: #151b20; padding: 16px; }
button { border: 1px solid #537a6e; background: #214d42; color: #f1fffb; padding: 8px 10px; font-weight: 700; cursor: pointer; }
input, select, textarea { width: 100%; box-sizing: border-box; margin: 6px 0; border: 1px solid #34434b; background: #0d1114; color: white; padding: 8px; }
table { width: 100%; border-collapse: collapse; font-size: 13px; }
td, th { border-bottom: 1px solid #26323a; padding: 6px; text-align: left; }
.online { color: #a5f2d1; }
.offline { color: #ffb4a5; }
pre { max-height: 240px; overflow: auto; background: #0d1114; border: 1px solid #34434b; padding: 10px; }
@media (max-width: 820px) { .grid { display: block; } section { margin-top: 14px; } }
</style>
<main>
  <header><div><h1>T-Bacon Hub</h1></div><div class="status" id="status">READY</div></header>
  <div class="grid">
    <section>
      <h2>Users</h2>
      <input id="user-id" placeholder="id ex: charlie">
      <input id="user-name" placeholder="name ex: Charlie">
      <button id="create-user">Create User</button>
      <table id="users"></table>
    </section>
    <section>
      <h2>Rooms</h2>
      <input id="room-id" placeholder="room id">
      <input id="room-name" placeholder="room name">
      <button id="create-room">Create Room</button>
      <table id="rooms"></table>
    </section>
    <section>
      <h2>Beacon Test</h2>
      <select id="beacon-user"></select>
      <select id="beacon-op">
        <option>alive</option><option>ack</option><option>nack</option><option>help</option><option>retry</option>
        <option>cap_l0</option><option>cap_l1</option><option>cap_l2</option><option>cap_l3</option>
      </select>
      <button id="send-beacon">Send Signed Image Beacon</button>
      <pre id="beacon-log"></pre>
    </section>
    <section>
      <h2>Events</h2>
      <pre id="events"></pre>
    </section>
  </div>
</main>
<script>
let users = [];
let seq = 1;
const encoder = new TextEncoder();

async function api(path, options) {
  const response = await fetch(path, options);
  const data = await response.json();
  if (!data.ok) throw new Error(data.error || "api failed");
  return data;
}

async function hmacSha256(secret, material) {
  const key = await crypto.subtle.importKey("raw", encoder.encode(secret), {name: "HMAC", hash: "SHA-256"}, false, ["sign"]);
  const sig = await crypto.subtle.sign("HMAC", key, encoder.encode(material));
  return Array.from(new Uint8Array(sig)).map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function refresh() {
  users = (await api("/api/users")).users;
  const rooms = (await api("/api/rooms")).rooms;
  const events = (await api("/api/events")).events;
  document.querySelector("#users").innerHTML = "<tr><th>User</th><th>Status</th><th>Level</th><th>Action</th></tr>" + users.map((u) =>
    `<tr><td>${u.id}<br>${u.name}</td><td class="${u.online ? "online" : "offline"}">${u.online ? "online" : "offline"}</td><td>${u.level || "L0"}</td><td><button data-rotate="${u.id}">rotate</button></td></tr>`
  ).join("");
  document.querySelector("#rooms").innerHTML = "<tr><th>Room</th><th>Members</th></tr>" + rooms.map((r) =>
    `<tr><td>${r.id}<br>${r.name}</td><td>${r.members.join(", ")}</td></tr>`
  ).join("");
  document.querySelector("#beacon-user").innerHTML = users.map((u) => `<option value="${u.id}">${u.id}</option>`).join("");
  document.querySelector("#events").textContent = JSON.stringify(events, null, 2);
}

async function sendBeacon() {
  const userId = document.querySelector("#beacon-user").value;
  const op = document.querySelector("#beacon-op").value;
  const user = users.find((item) => item.id === userId);
  const secret = (await api(`/api/users/${userId}/secret`)).secret;
  const sid = "dashboard";
  const nonce = crypto.randomUUID();
  const ts = Math.floor(Date.now() / 1000).toString();
  const currentSeq = (seq++).toString();
  const mac = await hmacSha256(secret, [sid, user.id, op, currentSeq, nonce, ts].join("|"));
  const url = `/tbacon/beacon/${op}.png?sid=${encodeURIComponent(sid)}&user=${encodeURIComponent(user.id)}&seq=${currentSeq}&nonce=${encodeURIComponent(nonce)}&ts=${ts}&mac=${mac}`;
  const image = new Image();
  image.src = url;
  document.querySelector("#beacon-log").textContent = url + "\\n" + document.querySelector("#beacon-log").textContent;
  setTimeout(refresh, 250);
}

document.querySelector("#create-user").addEventListener("click", async () => {
  await api("/api/users", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({id: document.querySelector("#user-id").value, name: document.querySelector("#user-name").value})});
  refresh();
});
document.querySelector("#create-room").addEventListener("click", async () => {
  await api("/api/rooms", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({id: document.querySelector("#room-id").value, name: document.querySelector("#room-name").value})});
  refresh();
});
document.querySelector("#send-beacon").addEventListener("click", () => sendBeacon().catch((error) => document.querySelector("#status").textContent = error.message));
document.addEventListener("click", async (event) => {
  const userId = event.target?.dataset?.rotate;
  if (userId) {
    await api(`/api/users/${userId}/rotate`, {method: "POST"});
    refresh();
  }
});
refresh();
setInterval(refresh, 3000);
</script>
</html>
"""


def twookie_dashboard() -> str:
    return """<!doctype html>
<html lang="fr">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>T-Wookie</title>
<style>
:root { color-scheme: dark; font-family: Inter, Segoe UI, system-ui, sans-serif; background: #0b0f10; color: #eef6f2; }
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; background: #0b0f10; }
button, input, textarea, select { font: inherit; }
button { border: 1px solid #6aa58d; background: #1f5a49; color: #f4fffb; min-height: 40px; padding: 9px 12px; font-weight: 700; cursor: pointer; }
button.secondary { background: #11191b; border-color: #3b4b50; }
input, textarea, select { width: 100%; border: 1px solid #314348; background: #101719; color: #f7fffd; padding: 10px; min-height: 42px; }
input[type="checkbox"] { width: auto; min-height: 0; padding: 0; accent-color: #6aa58d; }
textarea { min-height: 96px; resize: vertical; }
main { width: min(1220px, 100%); margin: 0 auto; padding: 18px; }
.top { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 8px 0 18px; border-bottom: 1px solid #263438; }
.brand { display: flex; align-items: center; gap: 12px; }
.mark { width: 42px; height: 42px; display: grid; place-items: center; border: 1px solid #6aa58d; background: #12231f; font-weight: 900; }
h1, h2, h3, p { margin-top: 0; }
h1 { margin-bottom: 2px; font-size: 28px; letter-spacing: 0; }
h2 { font-size: 18px; }
.muted { color: #9fb3af; }
.status { border: 1px solid #314348; padding: 8px 10px; color: #b8f4d9; white-space: nowrap; }
.layout { display: grid; grid-template-columns: 360px 1fr; gap: 18px; margin-top: 18px; }
section { border: 1px solid #263438; background: #101719; padding: 16px; }
.stack { display: grid; gap: 12px; }
.row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.actions { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
.pill { display: inline-flex; align-items: center; gap: 6px; border: 1px solid #314348; padding: 6px 8px; color: #cbe3dc; font-size: 12px; }
.list { display: grid; gap: 8px; }
.item { border: 1px solid #263438; background: #0c1214; padding: 10px; display: flex; justify-content: space-between; gap: 12px; align-items: center; }
.item strong { display: block; }
.danger { color: #ffc1b8; }
pre { white-space: pre-wrap; word-break: break-word; max-height: 240px; overflow: auto; background: #0b1012; border: 1px solid #263438; padding: 10px; }
.hidden { display: none; }
@media (max-width: 880px) {
  main { padding: 12px; }
  .top { align-items: flex-start; }
  .layout { grid-template-columns: 1fr; }
  .actions, .row { grid-template-columns: 1fr; }
  .status { white-space: normal; }
}
</style>
<main>
  <header class="top">
    <div class="brand">
      <div class="mark">T-W</div>
      <div><h1>T-Wookie</h1><div class="muted">Private session hub Â· local first Â· ngrok-ready</div></div>
    </div>
    <div class="status" id="status">Ready</div>
  </header>

  <div class="layout">
    <section class="stack">
      <div>
        <h2>Compte local</h2>
        <p class="muted">La seed est crÃ©Ã©e dans ce navigateur. Lâ€™empreinte appareil est hashÃ©e et loggÃ©e seulement si tu consens.</p>
      </div>
      <input id="handle" placeholder="handle ex: nate">
      <input id="display" placeholder="nom affichÃ© ex: Nate W">
      <label class="pill"><input id="consent" type="checkbox"> Jâ€™autorise un log dâ€™empreinte appareil pour ce test</label>
      <button id="create-account">CrÃ©er clÃ© + seed</button>
      <pre id="account-log"></pre>
    </section>

    <section class="stack">
      <div>
        <h2>Menu dâ€™action</h2>
        <p class="muted">Choisis les fonctions quâ€™on allow pour les tests. Les boutons appel/repo sont des hooks MVP.</p>
      </div>
      <div class="actions">
        <button data-action="add-user">Ajouter user</button>
        <button data-action="send-message">Envoyer message</button>
        <button data-action="call">Faire un appel</button>
        <button data-action="image-repo">Repo images</button>
      </div>
      <div id="panel-add-user" class="stack">
        <h3>Ajouter un user</h3>
        <select id="owner"></select>
        <select id="peer"></select>
        <button id="add-contact">Ajouter au carnet</button>
      </div>
      <div id="panel-send-message" class="stack hidden">
        <h3>Envoyer un message</h3>
        <div class="row"><select id="from"></select><select id="to"></select></div>
        <textarea id="message" placeholder="message de test"></textarea>
        <button id="send-message">CrÃ©er intention dâ€™envoi</button>
      </div>
      <div id="panel-call" class="stack hidden">
        <h3>Appel</h3>
        <div class="row"><select id="caller"></select><select id="callee"></select></div>
        <select id="call-mode"><option>signal</option><option>voice-placeholder</option><option>video-placeholder</option></select>
        <button id="start-call">CrÃ©er signal dâ€™appel</button>
      </div>
      <div id="panel-image-repo" class="stack hidden">
        <h3>Repo dâ€™images encodÃ©es</h3>
        <input id="payload-hash" placeholder="payload hash ou digest">
        <button id="add-node">Ajouter node</button>
        <pre id="nodes"></pre>
      </div>
    </section>
  </div>

  <section class="stack" style="margin-top:18px">
    <h2>RÃ©seau</h2>
    <div class="row">
      <div><div class="muted">Lien local</div><div class="pill" id="local-link"></div></div>
      <div><div class="muted">Ngrok</div><input id="ngrok-link" placeholder="colle ton lien ngrok ici"></div>
    </div>
    <p class="muted">Pour ngrok, garde une URL de session longue, expiration, auth/allowlist si possible, et ne partage jamais les fichiers privÃ©s.</p>
  </section>

  <section class="stack" style="margin-top:18px">
    <h2>Comptes</h2>
    <div class="list" id="accounts"></div>
  </section>
</main>
<script>
const enc = new TextEncoder();
let accounts = [];

async function api(path, options) {
  const response = await fetch(path, options);
  const data = await response.json();
  if (!data.ok) throw new Error(data.error || "api failed");
  return data;
}
function hex(buffer) {
  return Array.from(new Uint8Array(buffer)).map((b) => b.toString(16).padStart(2, "0")).join("");
}
async function sha256(value) {
  return hex(await crypto.subtle.digest("SHA-256", typeof value === "string" ? enc.encode(value) : value));
}
async function exportPublicKey(key) {
  const raw = await crypto.subtle.exportKey("raw", key);
  return btoa(String.fromCharCode(...new Uint8Array(raw)));
}
async function hardwareLog() {
  const screenInfo = window.screen ? {w: screen.width, h: screen.height, depth: screen.colorDepth} : {};
  let webgl = "unavailable";
  try {
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl");
    const ext = gl && gl.getExtension("WEBGL_debug_renderer_info");
    webgl = ext ? `${gl.getParameter(ext.UNMASKED_VENDOR_WEBGL)} | ${gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)}` : "webgl";
  } catch {}
  return {
    userAgent: navigator.userAgent,
    platform: navigator.platform,
    language: navigator.language,
    hardwareConcurrency: navigator.hardwareConcurrency || 0,
    deviceMemory: navigator.deviceMemory || 0,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    screen: screenInfo,
    webgl
  };
}
async function createAccount() {
  if (!document.querySelector("#consent").checked) throw new Error("Consentement requis pour ce test dâ€™empreinte appareil.");
  const id = document.querySelector("#handle").value.trim().toLowerCase().replace(/[^a-z0-9_-]/g, "").slice(0, 48);
  if (!id) throw new Error("handle requis");
  const display = document.querySelector("#display").value.trim() || id;
  const key = await crypto.subtle.generateKey({name: "ECDH", namedCurve: "P-256"}, true, ["deriveBits"]);
  const publicKey = await exportPublicKey(key.publicKey);
  const hw = await hardwareLog();
  const randomSeed = crypto.getRandomValues(new Uint8Array(32));
  const hwHash = await sha256(JSON.stringify(hw));
  const seedHash = await sha256(JSON.stringify({id, hwHash, random: hex(randomSeed)}));
  sessionStorage.setItem(`twookie:${id}:seedHash`, seedHash);
  const data = await api("/api/twookie/accounts", {
    method: "POST",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({id, display_name: display, public_key: publicKey, seed_hash: seedHash, hardware_log_hash: hwHash, hardware_log: hw})
  });
  document.querySelector("#account-log").textContent = JSON.stringify(data.account, null, 2);
  await refresh();
}
function setPanel(name) {
  for (const item of ["add-user", "send-message", "call", "image-repo"]) {
    document.querySelector(`#panel-${item}`).classList.toggle("hidden", item !== name);
  }
}
function fillSelect(selector) {
  document.querySelector(selector).innerHTML = accounts.map((a) => `<option value="${a.id}">${a.display_name} (${a.id})</option>`).join("");
}
async function refresh() {
  accounts = (await api("/api/twookie/accounts")).accounts;
  document.querySelector("#local-link").textContent = location.origin + "/twookie";
  for (const s of ["#owner", "#peer", "#from", "#to", "#caller", "#callee"]) fillSelect(s);
  document.querySelector("#accounts").innerHTML = accounts.map((a) => `<div class="item"><div><strong>${a.display_name}</strong><span class="muted">${a.id}</span></div><span class="pill">${a.revoked ? "revoked" : "active"}</span></div>`).join("") || "<div class='muted'>Aucun compte T-Wookie.</div>";
  const nodes = await api("/api/twookie/image-nodes");
  document.querySelector("#nodes").textContent = JSON.stringify(nodes.nodes, null, 2);
}
async function addContact() {
  await api("/api/twookie/contacts", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({owner_id: document.querySelector("#owner").value, peer_id: document.querySelector("#peer").value})});
  status.textContent = "Contact ajoutÃ©";
}
async function sendMessageIntent() {
  const body = document.querySelector("#message").value.trim();
  if (!body) throw new Error("message requis");
  await api("/api/twookie/messages", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({from: document.querySelector("#from").value, to: document.querySelector("#to").value, body})});
  status.textContent = "Intention message crÃ©Ã©e";
}
async function startCall() {
  await api("/api/twookie/calls", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({from: document.querySelector("#caller").value, to: document.querySelector("#callee").value, mode: document.querySelector("#call-mode").value})});
  status.textContent = "Signal dâ€™appel crÃ©Ã©";
}
async function addNode() {
  const payloadHash = document.querySelector("#payload-hash").value.trim() || await sha256(crypto.randomUUID());
  await api("/api/twookie/image-nodes", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({payload_hash: payloadHash, meta: {source: "twookie-ui"}})});
  await refresh();
}
document.querySelector("#create-account").addEventListener("click", () => createAccount().catch((e) => status.textContent = e.message));
document.querySelectorAll("[data-action]").forEach((b) => b.addEventListener("click", () => setPanel(b.dataset.action)));
document.querySelector("#add-contact").addEventListener("click", () => addContact().catch((e) => status.textContent = e.message));
document.querySelector("#send-message").addEventListener("click", () => sendMessageIntent().catch((e) => status.textContent = e.message));
document.querySelector("#start-call").addEventListener("click", () => startCall().catch((e) => status.textContent = e.message));
document.querySelector("#add-node").addEventListener("click", () => addNode().catch((e) => status.textContent = e.message));
refresh().catch((e) => status.textContent = e.message);
</script>
</html>
"""


def twookie_dashboard_v2() -> str:
    return """<!doctype html>
<html lang="fr">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>T-Wookie</title>
<style>
:root {
  color-scheme: dark;
  font-family: Inter, "Segoe UI", system-ui, sans-serif;
  background: #0f1115;
  color: #f4f7f5;
  --ember: #ff7a1a;
  --ember-2: #c94e16;
  --ember-soft: #2a170f;
  --ember-text: #fff4ea;
  --ember-glass: rgba(255, 214, 168, .76);
  --plum-ink: #241424;
  --plum-muted: #5b4059;
  --charcoal: #242936;
}
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; min-height: var(--app-height, 100dvh); background: #0f1115; overflow: hidden; }
button, input, textarea, select { font: inherit; }
button { border: 0; color: inherit; cursor: pointer; }
input, textarea, select {
  width: 100%;
  border: 1px solid #30343d;
  background: #171a21;
  color: #f8fbfa;
  border-radius: 18px;
  min-height: 42px;
  padding: 10px 13px;
  outline: none;
}
textarea { min-height: 46px; max-height: 124px; resize: none; border-radius: 20px; }
.entry {
  position: fixed;
  inset: 0;
  z-index: 15;
  display: none;
  place-items: center;
  padding: max(18px, env(safe-area-inset-top)) 18px max(18px, env(safe-area-inset-bottom));
  background: #101319;
  overflow: auto;
}
.app.entry-open .entry { display: grid; }
.entry-card {
  width: min(430px, 100%);
  display: grid;
  gap: 16px;
}
.entry-identity {
  display: grid;
  justify-items: center;
  gap: 10px;
  text-align: center;
  padding: 10px 0 4px;
}
.entry-logo {
  width: 112px;
  height: 112px;
  border-radius: 50%;
  overflow: hidden;
  border: 1px solid rgba(255, 173, 74, .72);
  background: #050609;
  box-shadow: 0 12px 38px rgba(0, 0, 0, .28);
}
.entry-logo img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.entry-identity h1 {
  margin: 0;
  font-size: 34px;
  line-height: 1;
  letter-spacing: 0;
}
.entry-identity p {
  margin: 0;
  color: #b9c1cd;
  line-height: 1.4;
}
.entry-panel {
  border: 1px solid #282e38;
  background: rgba(18, 22, 30, .86);
  border-radius: 22px;
  padding: 14px;
  display: grid;
  gap: 10px;
}
.entry-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px;
}
.entry-note {
  color: #9ca5b3;
  font-size: 12px;
  line-height: 1.4;
}
.app {
  height: 100vh;
  height: var(--app-height, 100dvh);
  display: grid;
  grid-template-columns: 330px minmax(0, 1fr) 310px;
  background: #0f1115;
}
.rail {
  border-right: 1px solid #242831;
  background: #151820;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.brand {
  padding: 18px 18px 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}
.logo {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  overflow: hidden;
  background: #050609;
  border: 1px solid #f0a23a;
  box-shadow: 0 0 18px rgba(240, 126, 32, .22);
  flex: 0 0 auto;
}
.logo img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.brand h1 { font-size: 22px; margin: 0; letter-spacing: 0; }
.presence { color: #aab2bd; font-size: 13px; margin-top: 2px; }
.connect {
  margin: 0 14px 12px;
  border: 1px solid #252a34;
  background: #10131a;
  border-radius: 20px;
  padding: 12px;
  display: grid;
  gap: 9px;
}
.account-card {
  display: none;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 10px;
}
.account-card .avatar { width: 42px; height: 42px; }
.account-card strong {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.account-actions {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 8px;
}
.account-actions button {
  min-height: 36px;
  border-radius: 14px;
  font-size: 12px;
  font-weight: 900;
}
.secondary {
  border: 1px solid #30343d;
  background: #171a21;
  color: #d8dedb;
}
.app.is-connected #account-panel { display: none; }
.app.is-connected #account-card { display: grid; }
.app.entry-open > .rail,
.app.entry-open > .chat,
.app.entry-open > .drawer { visibility: hidden; }
.app:not(.is-connected) .invitebox,
.app:not(.is-connected) .search,
.app:not(.is-connected) .channels { display: none; }
.tiny { color: #9ca5b3; font-size: 12px; line-height: 1.35; }
.check { display: flex; gap: 8px; align-items: flex-start; color: #b3bdc8; font-size: 12px; }
.check input { width: auto; min-height: 0; margin-top: 2px; accent-color: var(--ember); }
.primary {
  min-height: 42px;
  border-radius: 18px;
  background: linear-gradient(135deg, var(--ember), var(--ember-2));
  color: #170904;
  font-weight: 800;
  padding: 10px 14px;
  box-shadow: 0 0 18px rgba(255, 122, 26, .14);
}
.ghost {
  min-height: 38px;
  border-radius: 16px;
  border: 1px solid #30343d;
  background: #171a21;
  color: #d8dedb;
  padding: 9px 12px;
}
.search { padding: 0 14px 12px; }
.channels {
  padding: 0 8px 12px;
  overflow: auto;
  flex: 1;
  scrollbar-width: thin;
  scrollbar-color: #3a4150 transparent;
}
.channels::-webkit-scrollbar { height: 6px; width: 6px; }
.channels::-webkit-scrollbar-thumb { background: #3a4150; border-radius: 999px; }
.channels::-webkit-scrollbar-track { background: transparent; }
.channel {
  width: 100%;
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  text-align: left;
  padding: 9px 10px;
  border-radius: 16px;
  background: transparent;
}
.channel:hover, .channel.active { background: #242936; }
.avatar {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: #2b6dd8;
  display: grid;
  place-items: center;
  font-weight: 900;
  color: white;
}
.channel-title { font-weight: 800; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.channel-sub { color: #a0a9b5; font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.dot {
  min-width: 22px;
  height: 22px;
  border-radius: 999px;
  background: var(--ember);
  color: #170904;
  display: none;
  align-items: center;
  justify-content: center;
  padding: 0 6px;
  font-size: 11px;
  font-weight: 900;
}
.dot.unread { display: inline-flex; }
.chat {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: auto 1fr auto;
  background: linear-gradient(180deg, #12161d 0%, #0f1218 100%);
}
.topbar {
  min-height: 74px;
  border-bottom: 1px solid #242831;
  padding: 12px 18px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 14px;
}
.active-peer { display: flex; align-items: center; gap: 12px; min-width: 0; }
.active-peer h2 { margin: 0; font-size: 18px; letter-spacing: 0; }
.actions { display: flex; gap: 8px; flex-wrap: wrap; justify-content: flex-end; }
.backbtn {
  display: none;
  min-height: 36px;
  border-radius: 14px;
  background: #1b1f28;
  border: 1px solid #2d323c;
  color: #d8dedb;
  padding: 8px 10px;
  font-size: 12px;
  font-weight: 900;
}
.iconbtn, .actionbtn {
  min-height: 40px;
  border-radius: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  background: #1b1f28;
  border: 1px solid #2d323c;
  padding: 9px 12px;
  font-size: 13px;
  font-weight: 800;
  white-space: nowrap;
}
.actionbtn.primary-action {
  background: var(--ember-soft);
  border-color: var(--ember);
  color: var(--ember-text);
}
.timeline {
  overflow: auto;
  min-height: 0;
  padding: 20px 18px 28px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-right: 1px solid rgba(255, 255, 255, .05);
  overscroll-behavior: contain;
  scrollbar-width: thin;
  scrollbar-color: #3a4150 transparent;
}
.empty {
  margin: auto;
  max-width: 430px;
  text-align: center;
  color: #adb5bf;
  line-height: 1.45;
}
.msg {
  max-width: min(760px, 94%);
  display: grid;
  gap: 4px;
}
.msg.out { align-self: flex-end; justify-items: end; }
.msg.in { align-self: flex-start; justify-items: start; }
.bubble {
  padding: 12px 15px;
  border-radius: 20px;
  line-height: 1.38;
  word-break: break-word;
  font-size: 16px;
  backdrop-filter: blur(14px);
}
.msg.out .bubble {
  background: var(--ember-glass);
  color: var(--plum-ink);
  border: 1px solid rgba(255, 238, 215, .66);
  border-bottom-right-radius: 6px;
  box-shadow: 0 8px 18px rgba(0, 0, 0, .13);
  text-shadow: none;
}
.msg.in .bubble {
  background: rgba(238, 242, 248, .13);
  color: #f6f8fb;
  border: 1px solid rgba(238, 242, 248, .2);
  border-bottom-left-radius: 6px;
  box-shadow: 0 8px 18px rgba(0, 0, 0, .14);
}
.meta {
  padding: 0 8px;
  color: #98a4b4;
  font-size: 10px;
  opacity: .86;
}
.msg.out .meta { text-align: right; }
.source {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 2px 7px;
  margin-right: 5px;
  background: rgba(255,255,255,.14);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: .02em;
  text-transform: uppercase;
}
.source.discord { background: #5865f2; color: #fff; }
.msg.out .meta { color: #bda8bb; }
.source.web { background: rgba(255, 219, 178, .2); color: #f8d9c0; border: 1px solid rgba(255, 221, 184, .24); }
.source.bridge { background: #6f42c1; color: #fff; }
.source.codex { background: #2f7d68; color: #effff8; }
.source.ai { background: #7a4bd8; color: #fff; }
.platform {
  display: inline-flex;
  width: fit-content;
  border-radius: 999px;
  padding: 2px 7px;
  margin-top: 4px;
  background: #202633;
  color: #bfc8d6;
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}
.composer {
  border-top: 1px solid #242831;
  padding: 12px 16px 16px;
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 10px;
  align-items: end;
  background: rgba(18, 21, 28, .96);
}
.ptt {
  width: 56px;
  min-width: 56px;
  height: 54px;
  border-radius: 18px;
  background: #1b1009;
  color: #1a1200;
  font-weight: 900;
  border: 2px solid var(--ember);
  padding: 8px;
  display: grid;
  place-items: center;
}
.ptt.recording {
  background: #3a120b;
  box-shadow: 0 0 0 4px rgba(255, 122, 26, .18);
}
.ptt img {
  width: 30px;
  height: 30px;
  display: block;
}
.voice-card {
  display: grid;
  gap: 7px;
  min-width: min(260px, 72vw);
}
.voice-card audio {
  width: 100%;
  max-width: 320px;
  height: 34px;
}
.voice-label {
  font-size: 12px;
  font-weight: 900;
  color: inherit;
  opacity: .82;
}
.send {
  min-width: 86px;
  height: 54px;
  border-radius: 18px;
  background: linear-gradient(135deg, var(--ember), var(--ember-2));
  color: #170904;
  font-size: 14px;
  font-weight: 900;
  padding: 0 16px;
}
.langbar {
  display: flex;
  gap: 6px;
  margin: 0 14px 12px;
}
.langbtn {
  flex: 1;
  min-height: 32px;
  border-radius: 14px;
  background: #171a21;
  border: 1px solid #30343d;
  color: #cbd3df;
  font-size: 12px;
  font-weight: 800;
}
.langbtn.active {
  background: var(--ember-soft);
  border-color: var(--ember);
  color: var(--ember-text);
}
.drawer {
  border-left: 1px solid #242831;
  background: #151820;
  padding: 16px;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: 14px;
}
.panel {
  border: 1px solid #252a34;
  background: #10131a;
  border-radius: 20px;
  padding: 14px;
  display: grid;
  gap: 10px;
}
.panel h3 { margin: 0; font-size: 15px; }
.quick { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.invitebox {
  display: grid;
  gap: 9px;
}
.invite-link {
  border: 1px solid #2c3340;
  background: #0c0f14;
  color: #ccd6e2;
  border-radius: 14px;
  padding: 9px;
  font-size: 12px;
  word-break: break-all;
}
.invite-tools {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.invite-tools button {
  min-height: 36px;
  border-radius: 14px;
  border: 1px solid #30343d;
  background: #171a21;
  color: #d8dedb;
  font-size: 12px;
  font-weight: 800;
}
.public-pill {
  width: fit-content;
  border-radius: 999px;
  padding: 3px 8px;
  background: rgba(255, 214, 168, .14);
  border: 1px solid rgba(255, 214, 168, .24);
  color: #ffdcb8;
  font-size: 11px;
  font-weight: 900;
  text-transform: uppercase;
}
.qr {
  width: min(190px, 60vw);
  height: min(190px, 60vw);
  border-radius: 14px;
  background: #fff;
  padding: 8px;
  justify-self: center;
}
.invite-banner {
  margin: 0 14px 12px;
  border: 1px solid #d29a37;
  background: rgba(255, 214, 168, .1);
  border-radius: 18px;
  padding: 12px;
  display: none;
  gap: 9px;
}
.invite-banner.active { display: grid; }
.invite-banner.ready {
  border-color: rgba(81, 210, 163, .5);
  background: rgba(47, 125, 104, .18);
}
.invite-banner.ready #accept-invite { display: none; }
.log {
  min-height: 64px;
  max-height: 150px;
  overflow: auto;
  border-radius: 14px;
  background: #0c0f14;
  border: 1px solid #262b35;
  padding: 9px;
  color: #aeb7c2;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 12px;
  white-space: pre-wrap;
}
.share-modal {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: none;
  place-items: center;
  padding: 18px;
  background: rgba(5, 7, 11, .72);
  backdrop-filter: blur(12px);
}
.share-modal.active { display: grid; }
.share-card {
  width: min(440px, 100%);
  border: 1px solid rgba(255, 214, 168, .26);
  background: rgba(20, 23, 31, .96);
  border-radius: 24px;
  padding: 18px;
  display: grid;
  gap: 12px;
  box-shadow: 0 24px 70px rgba(0, 0, 0, .38);
}
.share-card h3 { margin: 0; font-size: 20px; }
.share-card .qr {
  width: min(300px, 76vw);
  height: min(300px, 76vw);
}
.share-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.share-actions button {
  min-height: 42px;
  border-radius: 16px;
  border: 1px solid #30343d;
  background: #171a21;
  color: #d8dedb;
  font-weight: 900;
}
.mobile-only { display: none; }
@media (max-width: 1080px) {
  .app { grid-template-columns: 300px minmax(0, 1fr); }
  .drawer { display: none; }
  .mobile-only { display: inline-grid; }
}
@media (max-width: 760px) {
  html, body { width: 100%; height: var(--app-height, 100dvh); overflow: hidden; position: fixed; inset: 0; }
  body { min-height: var(--app-height, 100dvh); }
  .entry { align-items: start; padding: max(18px, env(safe-area-inset-top)) 14px max(18px, env(safe-area-inset-bottom)); }
  .entry-card { gap: 12px; }
  .entry-logo { width: 96px; height: 96px; }
  .entry-identity h1 { font-size: 30px; }
  .entry-panel { border-radius: 19px; padding: 12px; }
  .entry-actions { grid-template-columns: 1fr; }
  .app { width: 100%; height: var(--app-height, 100dvh); display: flex; flex-direction: column; overflow: hidden; }
  .chat { display: none; }
  .app.mobile-chat-open .rail { display: none; }
  .app.mobile-chat-open .chat {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    width: 100%;
    height: var(--app-height, 100dvh);
    min-height: 0;
    flex: 1 1 auto;
  }
  .rail { border-right: 0; border-bottom: 0; height: var(--app-height, 100dvh); max-height: none; overflow: auto; flex: 1 1 auto; padding-bottom: max(14px, env(safe-area-inset-bottom)); }
  .brand { padding: 10px 12px 8px; gap: 9px; }
  .logo { width: 40px; height: 40px; }
  .brand h1 { font-size: 20px; }
  .presence { font-size: 12px; }
  .langbar { margin: 0 10px 8px; }
  .langbtn { min-height: 28px; border-radius: 12px; font-size: 11px; }
  .connect { margin: 0 10px 10px; padding: 10px; border-radius: 16px; gap: 7px; }
  .connect { grid-template-columns: 1fr; }
  input, textarea, select { min-height: 38px; border-radius: 15px; padding: 8px 11px; font-size: 14px; }
  .primary { min-height: 38px; border-radius: 15px; padding: 8px 12px; font-size: 14px; }
  .search { padding: 0 10px 9px; }
  .invitebox { display: grid; }
  .channels { display: flex; overflow-x: auto; padding-bottom: 10px; flex: none; }
  .channel { min-width: 185px; grid-template-columns: 36px minmax(0, 1fr) auto; gap: 8px; padding: 7px 8px; border-radius: 14px; }
  .channel .avatar { width: 36px; height: 36px; font-size: 13px; }
  .channel-title { font-size: 14px; }
  .channel-sub { font-size: 11px; }
  .chat { min-height: 0; flex: 1 1 auto; }
  .topbar { min-height: 50px; padding: 6px 8px; gap: 6px; align-items: center; flex: 0 0 auto; }
  .backbtn { display: none; align-items: center; justify-content: center; flex: 0 0 auto; }
  .app.mobile-chat-open .backbtn { display: inline-flex; }
  .active-peer { gap: 8px; flex: 1 1 auto; min-width: 96px; }
  .active-peer .avatar { width: 32px; height: 32px; font-size: 12px; }
  .active-peer h2 { font-size: 15px; line-height: 1.1; max-width: 128px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  #active-sub { max-width: 150px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: 10px; }
  .actions { gap: 4px; flex: 0 0 auto; align-content: center; align-items: center; }
  .actionbtn { min-height: 30px; border-radius: 12px; padding: 5px 7px; font-size: 10px; gap: 4px; }
  #mobile-request { display: none; }
  .timeline { min-height: 0; height: auto; padding: 12px 10px; gap: 9px; flex: 1 1 auto; -webkit-overflow-scrolling: touch; }
  .bubble { padding: 10px 12px; border-radius: 17px; font-size: 15px; }
  .msg { max-width: 98%; }
  .composer { position: relative; bottom: auto; padding: 8px 8px max(8px, env(safe-area-inset-bottom)); transform: translateY(calc(-1 * var(--keyboard-lift, 0px))); grid-template-columns: auto minmax(0, 1fr) auto; gap: 7px; flex: 0 0 auto; }
  .ptt { width: 44px; min-width: 44px; height: 42px; border-radius: 14px; padding: 7px; }
  .ptt img { width: 24px; height: 24px; }
  .send { min-width: 58px; height: 42px; border-radius: 14px; padding: 0 10px; font-size: 12px; }
  textarea { min-height: 42px; max-height: 92px; border-radius: 16px; font-size: 16px; }
}
@media (max-width: 390px) {
  .topbar { padding: 6px 7px; gap: 5px; }
  .active-peer h2 { max-width: 94px; font-size: 14px; }
  #active-sub { max-width: 110px; }
  .actions { gap: 3px; }
  .actionbtn { padding: 5px 6px; font-size: 9px; min-height: 28px; }
  .backbtn { min-height: 28px; padding: 5px 7px; font-size: 10px; }
  .send { min-width: 52px; padding: 0 8px; }
}
</style>
<div class="app entry-open">
  <section class="entry" id="entry-screen">
    <div class="entry-card">
      <div class="entry-identity">
        <div class="entry-logo"><img src="/assets/twookie-logo.png" alt="T-Wookie"></div>
        <h1>T-Wookie</h1>
        <p id="entry-copy">Connexion privee ou mode invite. Choisis ton entree, puis le chat s'ouvre comme un talkie-walkie.</p>
      </div>
      <div class="entry-panel">
        <input id="entry-handle" placeholder="handle ex: nate">
        <input id="entry-display" placeholder="nom affiche">
        <label class="check"><input id="entry-consent" type="checkbox"><span>Log appareil hashe pour ce test</span></label>
        <button class="primary" id="entry-connect">Se connecter</button>
        <div class="entry-actions">
          <button class="secondary" id="entry-guest">Mode invite</button>
          <button class="secondary" id="entry-existing">Continuer session</button>
        </div>
        <div class="entry-note" id="entry-note">Aucune empreinte brute n'est stockee. Les messages restent cote serveur et ne sont pas copies en localStorage.</div>
      </div>
    </div>
  </section>
  <aside class="rail">
    <div class="brand">
      <div class="logo"><img src="/assets/twookie-logo.png" alt="T-Wookie"></div>
      <div>
        <h1>T-Wookie</h1>
        <div class="presence" id="presence">Connecte-toi pour demarrer</div>
      </div>
    </div>
    <div class="langbar">
      <button class="langbtn" data-lang="fr" id="lang-fr">FR</button>
      <button class="langbtn" data-lang="en" id="lang-en">EN</button>
    </div>
    <div class="connect account-card" id="account-card">
      <div class="avatar" id="account-card-avatar">?</div>
      <div>
        <strong id="account-card-name">Non connecte</strong>
        <div class="tiny" id="account-card-sub">Session locale inactive</div>
      </div>
      <div class="account-actions">
        <button class="primary" id="account-invite" data-i18n="invite_btn">Inviter</button>
        <button class="secondary" id="change-account" data-i18n="change_account">Changer</button>
      </div>
    </div>
    <div class="connect" id="account-panel">
      <input id="handle" data-i18n-placeholder="handle_ph" placeholder="handle ex: nate">
      <input id="display" data-i18n-placeholder="display_ph" placeholder="nom affiche">
      <label class="check"><input id="consent" type="checkbox"><span data-i18n="device_log">Log appareil hashe pour ce test</span></label>
      <button class="primary" id="create-account" data-i18n="connect">Creer / connecter</button>
      <div class="tiny" id="account-note" data-i18n="seed_note">Seed locale + cle publique de test. Aucune empreinte brute n'est affichee.</div>
    </div>
    <div class="search"><input id="search" data-i18n-placeholder="search_ph" placeholder="Rechercher un contact"></div>
    <div class="connect invitebox">
      <strong data-i18n="quick_invite">Invitation rapide</strong>
      <input id="invite-label" data-i18n-placeholder="invite_ph" placeholder="nom de l'invite">
      <button class="primary" id="create-invite" data-i18n="create_invite">Creer lien + QR</button>
      <div class="public-pill" id="public-origin">Local</div>
      <div class="invite-link" id="invite-link" data-i18n="no_invite">Aucune invitation creee.</div>
      <img class="qr" id="invite-qr" alt="QR invitation" hidden>
      <div class="invite-tools">
        <button id="copy-invite" data-i18n="copy_invite">Copier</button>
        <button id="open-qr" data-i18n="open_qr">QR plein ecran</button>
      </div>
    </div>
    <div class="invite-banner" id="invite-banner">
      <strong>Invitation T-Wookie</strong>
      <div class="tiny" id="invite-text">Complete ton profil pour rejoindre le canal.</div>
      <button class="primary" id="accept-invite" data-i18n="accept_invite">Accepter l'invitation</button>
    </div>
    <div class="channels" id="channels"></div>
  </aside>

  <main class="chat">
    <header class="topbar">
      <button class="backbtn" id="back-to-list" data-i18n="back">Precedent</button>
      <div class="active-peer">
        <div class="avatar" id="active-avatar">?</div>
        <div>
          <h2 id="active-name">Aucun canal</h2>
          <div class="presence" id="active-sub">Ajoute un user ou choisis un contact</div>
        </div>
      </div>
      <div class="actions">
        <button class="actionbtn mobile-only" id="mobile-request" title="Nouvelle demande" data-i18n="request">+ Demande</button>
        <button class="actionbtn" id="top-invite" title="Invitation" data-i18n="invite_btn">Inviter</button>
        <button class="actionbtn primary-action" id="call-peer" title="Signal appel" data-i18n="call">Appel</button>
        <button class="actionbtn" id="repo-refresh" title="Synchroniser" data-i18n="sync">Sync</button>
        <button class="actionbtn" id="purge-chat" title="Effacer le canal" data-i18n="purge">Effacer</button>
      </div>
    </header>
    <section class="timeline" id="timeline">
      <div class="empty" data-i18n="empty">Connecte un compte, choisis un contact, puis ecris directement. T-Wookie garde le dernier canal actif comme un talkie-walkie texte.</div>
    </section>
    <footer class="composer">
      <button class="ptt" id="ptt" title="Micro"><img src="/assets/twookie-mic.png" alt="Micro"></button>
      <textarea id="message" data-i18n-placeholder="message_ph" placeholder="Message au dernier contact actif"></textarea>
      <button class="send" id="send-message" title="Envoyer" data-i18n="send">Envoyer</button>
    </footer>
  </main>

  <aside class="drawer">
    <section class="panel">
      <h3>Nouvelle demande</h3>
      <select id="owner"></select>
      <select id="peer"></select>
      <button class="primary" id="add-contact">Ajouter canal</button>
    </section>
    <section class="panel">
      <h3>Actions rapides</h3>
      <div class="quick">
        <button class="ghost" id="switch-first">Canal suivant</button>
        <button class="ghost" id="start-call">Lancer appel</button>
      </div>
      <input id="ngrok-link" placeholder="lien ngrok a partager">
      <div class="tiny">Partage l'URL ngrok seulement a des users autorises. Le serveur local reste la source.</div>
    </section>
    <section class="panel">
      <h3>Repo images</h3>
      <button class="ghost" id="add-node">Ajouter node test</button>
      <div class="log" id="nodes">Chargement...</div>
    </section>
    <section class="panel">
      <h3>Etat</h3>
      <div class="log" id="status">Ready</div>
    </section>
  </aside>
</div>
<div class="share-modal" id="share-modal" aria-hidden="true">
  <div class="share-card">
    <h3>Invitation T-Wookie</h3>
    <div class="public-pill" id="share-origin">Ngrok public</div>
    <img class="qr" id="share-qr" alt="QR invitation">
    <div class="invite-link" id="share-link"></div>
    <div class="share-actions">
      <button id="share-copy">Copier lien</button>
      <button id="share-open">Ouvrir QR</button>
    </div>
    <button class="primary" id="share-close">Fermer</button>
  </div>
</div>
<script>
const enc = new TextEncoder();
const i18n = {
  fr: {
    handle_ph: "handle ex: nate",
    display_ph: "nom affiche",
    device_log: "Log appareil hashe pour ce test",
    connect: "Creer / connecter",
    seed_note: "Seed locale + cle publique de test. Aucune empreinte brute n'est affichee.",
    search_ph: "Rechercher un contact",
    quick_invite: "Invitation rapide",
    invite_ph: "nom de l'invite",
    create_invite: "Creer lien + QR",
    invite_btn: "Inviter",
    change_account: "Changer",
    copy_invite: "Copier",
    open_qr: "QR plein ecran",
    no_invite: "Aucune invitation creee.",
    accept_invite: "Accepter l'invitation",
    guest_ready: "Canal invite pret",
    back: "Precedent",
    request: "+ Demande",
    call: "Appel",
    sync: "Sync",
    purge: "Effacer",
    empty: "Connecte un compte, choisis un contact, puis ecris directement. T-Wookie garde le dernier canal actif.",
    message_ph: "Message au dernier contact actif",
    send: "Envoyer",
    no_channel: "Aucun canal",
    add_contact: "Ajoute un user ou choisis un contact",
    direct: "Canal direct",
  },
  en: {
    handle_ph: "handle e.g. nate",
    display_ph: "display name",
    device_log: "Hash device log for this test",
    connect: "Create / connect",
    seed_note: "Local seed + test public key. Raw device fingerprint is not shown.",
    search_ph: "Search contacts",
    quick_invite: "Quick invite",
    invite_ph: "invite name",
    create_invite: "Create link + QR",
    invite_btn: "Invite",
    change_account: "Switch",
    copy_invite: "Copy",
    open_qr: "Full QR",
    no_invite: "No invite created.",
    accept_invite: "Accept invite",
    guest_ready: "Guest channel ready",
    back: "Back",
    request: "+ Request",
    call: "Call",
    sync: "Sync",
    purge: "Clear",
    empty: "Connect an account, choose a contact, then type directly. T-Wookie keeps the last active channel.",
    message_ph: "Message last active contact",
    send: "Send",
    no_channel: "No channel",
    add_contact: "Add a user or choose a contact",
    direct: "Direct channel",
  }
};
const state = {
  accounts: [],
  owner: localStorage.getItem("twookie.owner") || "",
  activePeer: localStorage.getItem("twookie.activePeer") || "",
  messages: [],
  lastBridgeId: Number(localStorage.getItem("twookie.lastBridgeId") || "0"),
  lastGlobalBridgeId: Number(localStorage.getItem("twookie.lastGlobalBridgeId") || "0"),
  unread: JSON.parse(localStorage.getItem("twookie.unread") || "{}"),
  inviteToken: new URLSearchParams(location.search).get("invite") || "",
  lang: localStorage.getItem("twookie.lang") || "fr",
  publicOrigin: "",
  inviteLink: "",
  inviteQr: "",
  inviteReady: false,
  recorder: null,
  recordChunks: [],
  recordStartedAt: 0,
  recordTimer: null,
  _pixelKeyCache: {},
};

const $ = (id) => document.getElementById(id);
const isMobile = () => window.matchMedia("(max-width: 760px)").matches;
localStorage.removeItem("twookie.messages");

function setAppHeight() {
  const viewport = window.visualViewport;
  const height = viewport?.height || window.innerHeight;
  const lift = viewport ? Math.max(0, window.innerHeight - viewport.height - viewport.offsetTop) : 0;
  document.documentElement.style.setProperty("--app-height", `${height}px`);
  document.documentElement.style.setProperty("--keyboard-lift", `${lift}px`);
  if (state.activePeer && document.activeElement === $("message")) {
    requestAnimationFrame(() => {
      $("timeline").scrollTop = $("timeline").scrollHeight;
      $("message").scrollIntoView({block: "nearest"});
    });
  }
}
function openMobileChat() {
  if (isMobile() && state.activePeer) document.querySelector(".app").classList.add("mobile-chat-open");
}
function closeMobileChat() {
  document.querySelector(".app").classList.remove("mobile-chat-open");
}

async function api(path, options) {
  const response = await fetch(path, options);
  const data = await response.json();
  if (!data.ok) throw new Error(data.error || "api failed");
  return data;
}
function setStatus(text) {
  $("status").textContent = `${new Date().toLocaleTimeString()}  ${text}`;
}
function t(key) {
  return (i18n[state.lang] || i18n.fr)[key] || i18n.fr[key] || key;
}
function applyLanguage() {
  document.documentElement.lang = state.lang;
  document.querySelectorAll("[data-i18n]").forEach((el) => el.textContent = t(el.dataset.i18n));
  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => el.placeholder = t(el.dataset.i18nPlaceholder));
  $("lang-fr").classList.toggle("active", state.lang === "fr");
  $("lang-en").classList.toggle("active", state.lang === "en");
}
function setLanguage(lang) {
  state.lang = lang === "en" ? "en" : "fr";
  localStorage.setItem("twookie.lang", state.lang);
  applyLanguage();
  renderActive();
  renderMessages();
}
function initials(account) {
  const value = (account?.display_name || account?.id || "?").trim();
  return value.split(/\\s+/).slice(0, 2).map((p) => p[0]).join("").toUpperCase() || "?";
}
function shortId(value) {
  return value ? value.slice(0, 8) : "local";
}
function hex(buffer) {
  return Array.from(new Uint8Array(buffer)).map((b) => b.toString(16).padStart(2, "0")).join("");
}
async function sha256(value) {
  return hex(await crypto.subtle.digest("SHA-256", typeof value === "string" ? enc.encode(value) : value));
}
function randomHex(byteCount) {
  return hex(crypto.getRandomValues(new Uint8Array(byteCount)));
}
function hardwareLog(consent) {
  if (!consent) return { consent: false, platform: "redacted" };
  return {
    consent: true,
    platform: navigator.platform || "unknown",
    languages: navigator.languages || [],
    cores: navigator.hardwareConcurrency || 0,
    memory: navigator.deviceMemory || 0,
    screen: `${screen.width}x${screen.height}x${screen.colorDepth}`,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "unknown",
  };
}
async function buildAccountPayload(id, displayName, hardware = null) {
  const seed = crypto.getRandomValues(new Uint8Array(32));
  const hardwareData = hardware || hardwareLog(false);
  const hardwareJson = JSON.stringify(hardwareData);
  return {
    id,
    display_name: displayName,
    public_key: await sha256(`twookie-public|${id}|${hex(seed)}`),
    seed_hash: await sha256(`seed|${id}|${hex(seed)}`),
    hardware_log_hash: await sha256(hardwareJson),
    hardware_log: hardwareData,
  };
}
async function createAccount(options = {}) {
  const rawHandle = options.handle ?? $("handle").value;
  const rawDisplay = options.display ?? $("display").value;
  const consent = Boolean(options.consent ?? $("consent").checked);
  const id = rawHandle.trim().toLowerCase().replace(/[^a-z0-9_.-]/g, "-");
  if (!id) throw new Error("Handle requis");
  const displayName = rawDisplay.trim() || id;
  const hardware = hardwareLog(consent);
  const body = await buildAccountPayload(id, displayName, hardware);
  await api("/api/twookie/accounts", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify(body)});
  state.owner = id;
  localStorage.setItem("twookie.owner", id);
  $("account-note").textContent = `Connecte comme ${displayName} / cle ${shortId(body.public_key)}`;
  if (state.inviteToken) await acceptInviteWithPayload(body);
  setStatus(`Compte actif: ${id}`);
  await refresh();
  await createQuickInvite();
}
async function entryConnect() {
  const handle = $("entry-handle").value || $("handle").value;
  const display = $("entry-display").value || $("display").value;
  await createAccount({handle, display, consent: $("entry-consent").checked});
}
async function entryGuest() {
  const suffix = randomHex(3);
  const handle = $("entry-handle").value.trim() || `invite-${suffix}`;
  const display = $("entry-display").value.trim() || `Invite ${suffix.toUpperCase()}`;
  await createAccount({handle, display, consent: false});
}
function continueExistingSession() {
  if (!state.owner) {
    $("entry-note").textContent = "Aucune session locale trouvee. Choisis un handle ou le mode invite.";
    return;
  }
  updateSessionUi();
  if (state.activePeer) openMobileChat();
}
function saveMessages() {
  localStorage.removeItem("twookie.messages");
  state.messages = state.messages.slice(-120);
  localStorage.setItem("twookie.lastBridgeId", String(state.lastBridgeId));
  localStorage.setItem("twookie.lastGlobalBridgeId", String(state.lastGlobalBridgeId));
  localStorage.setItem("twookie.unread", JSON.stringify(state.unread));
}
function accountById(id) {
  return state.accounts.find((item) => item.id === id);
}
function currentOwner() {
  return accountById(state.owner) || (state.owner ? {id: state.owner, display_name: state.owner} : null);
}
function updateSessionUi() {
  const owner = currentOwner();
  document.querySelector(".app").classList.toggle("is-connected", Boolean(owner));
  document.querySelector(".app").classList.toggle("entry-open", !owner && !state.inviteReady);
  if (!owner) {
    $("account-card-avatar").textContent = "?";
    $("account-card-name").textContent = "Non connecte";
    $("account-card-sub").textContent = "Cree un compte ou ouvre une invitation";
    return;
  }
  $("account-card-avatar").textContent = initials(owner);
  $("account-card-name").textContent = owner.display_name || owner.id;
  const seen = owner.last_seen ? `actif ${new Date(owner.last_seen * 1000).toLocaleTimeString()}` : "session locale active";
  $("account-card-sub").textContent = `${owner.id} - ${seen}`;
}
function switchAccount() {
  state.owner = "";
  state.activePeer = "";
  state.inviteLink = "";
  state.inviteQr = "";
  state.inviteReady = false;
  localStorage.removeItem("twookie.owner");
  localStorage.removeItem("twookie.activePeer");
  closeMobileChat();
  updateSessionUi();
  renderChannels();
  renderActive();
  renderMessages();
  setStatus("Session locale deconnectee");
}
function platformFor(id, source) {
  const text = `${id || ""} ${source || ""}`.toLowerCase();
  if (text.includes("discord")) return "discord";
  if (text.includes("codex")) return "codex";
  if (text.includes("ai")) return "ai";
  if (text.includes("bridge")) return "bridge";
  return "web";
}
function sourceLabel(source, id) {
  const value = String(source || platformFor(id, "") || "web");
  if (value === "ai_public") return "AI public";
  if (value === "discord_bot") return "Discord";
  if (value === "twookie_web") return "Web";
  if (value === "local_web") return "Web local";
  if (value === "codex_local") return "Codex";
  return value.replaceAll("_", " ");
}
function inviteOrigin() {
  return state.publicOrigin || location.origin;
}
function showShareModal() {
  if (!state.inviteLink || !state.inviteQr) return;
  $("share-link").textContent = state.inviteLink;
  $("share-qr").src = state.inviteQr;
  $("share-origin").textContent = state.publicOrigin ? "Ngrok public" : "Local";
  $("share-modal").classList.add("active");
  $("share-modal").setAttribute("aria-hidden", "false");
}
function hideShareModal() {
  $("share-modal").classList.remove("active");
  $("share-modal").setAttribute("aria-hidden", "true");
}
async function loadPublicSession() {
  try {
    const data = await api("/api/twookie/public-url");
    const publicUrl = (data.session?.public_url || "").replace(/\\/$/, "");
    state.publicOrigin = publicUrl || "";
    if (state.publicOrigin) {
      $("public-origin").textContent = "Ngrok public";
      $("ngrok-link").value = `${state.publicOrigin}/`;
    } else {
      $("public-origin").textContent = "Local";
    }
  } catch {
    state.publicOrigin = "";
    $("public-origin").textContent = "Local";
  }
}
async function refresh() {
  applyLanguage();
  await loadPublicSession();
  await loadInvite();
  if (state.owner) {
    await api("/api/twookie/touch", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({id: state.owner})}).catch(() => {});
  }
  const data = await api("/api/twookie/accounts");
  state.accounts = data.accounts || [];
  updateSessionUi();
  if (state.owner && !state.activePeer) {
    const firstPeer = state.accounts.find((item) => item.id !== state.owner);
    if (firstPeer) setActivePeer(firstPeer.id, false);
  }
  renderSelectors();
  await loadRecentBridgeMessages();
  if (state.owner && state.activePeer) await loadBridgeMessages(0);
  renderChannels();
  renderActive();
  renderMessages();
  updateSessionUi();
  if (state.inviteReady && state.activePeer) openMobileChat();
  await refreshNodes();
}
async function loadInvite() {
  if (!state.inviteToken) return;
  try {
    const data = await api(`/api/twookie/invites/${encodeURIComponent(state.inviteToken)}`);
    $("invite-banner").classList.add("active");
    $("invite-text").textContent = `Invite par ${data.invite.inviter_id}. Canal prive en preparation...`;
    await ensureInviteChannel(data.invite);
  } catch (error) {
    $("invite-banner").classList.add("active");
    $("invite-text").textContent = `Invitation invalide: ${error.message}`;
  }
}
async function acceptInviteWithPayload(body) {
  const data = await api(`/api/twookie/invites/${encodeURIComponent(state.inviteToken)}/accept`, {
    method: "POST",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify(body),
  });
  state.owner = data.account.id;
  state.activePeer = data.peer_id;
  localStorage.setItem("twookie.owner", state.owner);
  localStorage.setItem("twookie.activePeer", state.activePeer);
  return data;
}
async function ensureInviteChannel(invite) {
  if (!state.inviteToken || state.inviteReady) return;
  const storedOwner = localStorage.getItem(`twookie.invite.${state.inviteToken}.owner`);
  let id = state.owner || storedOwner;
  let displayName = id || "";
  if (!id) {
    id = `invite-${state.inviteToken.slice(0, 6).toLowerCase()}-${randomHex(3)}`;
    displayName = `Invite ${state.inviteToken.slice(0, 4).toUpperCase()}`;
  } else {
    displayName = currentOwner()?.display_name || id;
  }
  const body = await buildAccountPayload(id, displayName, { consent: false, source: "invite_link", invite: state.inviteToken.slice(0, 8) });
  const data = await acceptInviteWithPayload(body);
  localStorage.setItem(`twookie.invite.${state.inviteToken}.owner`, data.account.id);
  state.inviteReady = true;
  $("invite-banner").classList.add("ready");
  $("invite-text").textContent = `${t("guest_ready")}: ${data.account.display_name || data.account.id} -> ${invite.inviter_id}`;
  setStatus(`Canal invite pret avec ${invite.inviter_id}`);
  updateSessionUi();
  renderActive();
  renderMessages();
  openMobileChat();
}
function renderSelectors() {
  const options = state.accounts.map((a) => `<option value="${a.id}">${a.display_name || a.id}</option>`).join("");
  $("owner").innerHTML = options;
  $("peer").innerHTML = options;
  if (state.owner) $("owner").value = state.owner;
  if (state.activePeer) $("peer").value = state.activePeer;
}
function renderChannels() {
  const query = $("search").value.trim().toLowerCase();
  const peers = state.accounts.filter((item) => item.id !== state.owner).filter((item) => {
    const text = `${item.id} ${item.display_name || ""}`.toLowerCase();
    return !query || text.includes(query);
  });
  $("channels").innerHTML = peers.map((peer) => `
    <button class="channel ${peer.id === state.activePeer ? "active" : ""}" data-peer="${peer.id}">
      <div class="avatar">${initials(peer)}</div>
      <div>
        <div class="channel-title">${peer.display_name || peer.id}</div>
        <span class="platform">${platformFor(peer.id, "")}</span>
        <div class="channel-sub">${peer.id} Â· cle ${shortId(peer.public_key)}</div>
      </div>
      <span class="dot ${(state.unread[peer.id] || 0) > 0 ? "unread" : ""}">${state.unread[peer.id] || ""}</span>
    </button>
  `).join("") || `<div class="tiny" style="padding:12px">Aucun contact. Cree un autre compte ou envoie une demande.</div>`;
}
function setActivePeer(peerId, render = true) {
  state.activePeer = peerId || "";
  localStorage.setItem("twookie.activePeer", state.activePeer);
  if (state.activePeer) {
    delete state.unread[state.activePeer];
    saveMessages();
  }
  if (render) {
    renderChannels();
    renderActive();
    renderMessages();
    openMobileChat();
    loadBridgeMessages(0).then((changed) => {
      if (changed) {
        renderChannels();
        renderMessages();
      }
    }).catch(() => {});
  }
}
function renderActive() {
  const owner = accountById(state.owner);
  const peer = accountById(state.activePeer);
  $("presence").textContent = owner ? `Connecte comme ${owner.display_name || owner.id}` : "Connecte-toi pour demarrer";
  $("active-avatar").textContent = initials(peer);
  $("active-name").textContent = peer ? (peer.display_name || peer.id) : t("no_channel");
  const seen = peer?.last_seen ? `vu ${new Date(peer.last_seen * 1000).toLocaleTimeString()}` : "jamais vu";
  $("active-sub").textContent = peer ? `${t("direct")} - ${seen}` : t("add_contact");
}
function renderMessages() {
  const peer = state.activePeer;
  const relevant = state.messages.filter((m) => {
    return (m.from === state.owner && m.to === peer) || (m.from === peer && m.to === state.owner);
  });
  if (!peer) {
    $("timeline").innerHTML = `<div class="empty">${t("empty")}</div>`;
    return;
  }
  $("timeline").innerHTML = relevant.map((m) => `
    <div class="msg ${m.from === state.owner ? "out" : "in"}">
      <div class="bubble">${renderMessageBody(m)}</div>
      <div class="meta"><span class="source ${platformFor(m.from, m.source)}">${sourceLabel(m.source, m.from)}</span>${new Date(m.at).toLocaleTimeString()} - ${m.node ? `node ${shortId(m.node)}` : "local"}</div>
    </div>
  `).join("") || `<div class="empty">Canal pret avec ${escapeHtml(peer)}. Envoie ton premier message.</div>`;
  $("timeline").scrollTop = $("timeline").scrollHeight;
}
function peerForMessage(msg) {
  return msg.from_id === state.owner ? msg.to_id : msg.from_id;
}
function ingestBridgeMessages(messages) {
  let changed = false;
  for (const msg of messages || []) {
    state.lastBridgeId = Math.max(state.lastBridgeId, Number(msg.id || 0));
    state.lastGlobalBridgeId = Math.max(state.lastGlobalBridgeId, Number(msg.id || 0));
    const peer = peerForMessage(msg);
    const meta = parseBridgeMeta(msg);
    if (!state.messages.some((item) => item.bridgeId === msg.id)) {
      state.messages.push({
        from: msg.from_id,
        to: msg.to_id,
        body: msg.body,
        at: (msg.created_at || Date.now() / 1000) * 1000,
        node: msg.status,
        bridgeId: msg.id,
        source: msg.source || msg.direction,
        kind: meta.kind || "text",
        voiceId: meta.voice_id || "",
        pixelId: meta.pixel_id || "",
        cipher: meta.cipher || "",
        nonceB64: meta.nonce_b64 || "",
        chunkCount: Number(meta.chunk_count || 0),
        durationMs: Number(meta.duration_ms || 0),
      });
      if (peer && peer !== state.activePeer && msg.to_id === state.owner) {
        state.unread[peer] = (state.unread[peer] || 0) + 1;
      }
      changed = true;
    }
  }
  if (changed) saveMessages();
  return changed;
}
async function loadRecentBridgeMessages() {
  if (!state.owner) return false;
  const params = new URLSearchParams({owner: state.owner, after: String(state.lastGlobalBridgeId)});
  const data = await api(`/api/twookie/bridge/recent?${params.toString()}`);
  return ingestBridgeMessages(data.messages || []);
}
async function loadBridgeMessages(afterOverride = null) {
  if (!state.owner || !state.activePeer) return false;
  const params = new URLSearchParams({owner: state.owner, peer: state.activePeer, after: String(afterOverride ?? state.lastBridgeId)});
  const data = await api(`/api/twookie/bridge/messages?${params.toString()}`);
  return ingestBridgeMessages(data.messages || []);
}
async function syncConversation() {
  if (state.owner && state.activePeer) {
    await loadBridgeMessages(0);
    renderMessages();
  }
  const data = await api("/api/twookie/accounts");
  state.accounts = data.accounts || [];
  renderChannels();
  renderActive();
  setStatus("Conversation synchronisee");
}
async function pollUpdates() {
  const changed = await loadRecentBridgeMessages();
  if (changed) {
    renderChannels();
    renderMessages();
  }
}
async function loadBridgeMessagesLegacy() {
  if (!state.owner || !state.activePeer) return;
  const params = new URLSearchParams({owner: state.owner, peer: state.activePeer, after: String(state.lastBridgeId)});
  const data = await api(`/api/twookie/bridge/messages?${params.toString()}`);
  for (const msg of data.messages || []) {
    state.lastBridgeId = Math.max(state.lastBridgeId, Number(msg.id || 0));
    if (!state.messages.some((item) => item.bridgeId === msg.id)) {
      const meta = parseBridgeMeta(msg);
      state.messages.push({from: msg.from_id, to: msg.to_id, body: msg.body, at: (msg.created_at || Date.now() / 1000) * 1000, node: msg.status, bridgeId: msg.id, source: msg.source || msg.direction, kind: meta.kind || "text", voiceId: meta.voice_id || "", pixelId: meta.pixel_id || "", cipher: meta.cipher || "", nonceB64: meta.nonce_b64 || "", chunkCount: Number(meta.chunk_count || 0), durationMs: Number(meta.duration_ms || 0)});
    }
  }
  saveMessages();
}
function escapeHtml(value) {
  return String(value || "").replace(/[&<>"']/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#039;" }[char]));
}
function escapeAttr(value) {
  return escapeHtml(String(value || ""));
}
function parseBridgeMeta(msg) {
  if (msg.meta) return msg.meta;
  try {
    return JSON.parse(msg.meta_json || "{}");
  } catch {
    return {};
  }
}
function parseMetaJson(value) {
  try {
    return JSON.parse(value || "{}");
  } catch {
    return {};
  }
}
function renderMessageBody(message) {
  if (message.kind === "voice_memo" && message.voiceId) {
    const seconds = Math.max(1, Math.round((message.durationMs || 0) / 1000));
    return `
      <div class="voice-card">
        <div class="voice-label">Memo vocal ${seconds}s</div>
        <audio controls preload="none" src="/api/twookie/voice/${encodeURIComponent(message.voiceId)}"></audio>
      </div>
    `;
  }
  if (message.kind === "pixel_voice_memo" && message.pixelId) {
    const seconds = Math.max(1, Math.round((message.durationMs || 0) / 1000));
    return `
      <div class="voice-card pixel-voice" data-pixel-id="${escapeAttr(message.pixelId)}">
        <div class="voice-label">Memo vocal pixel ${seconds}s · ${message.chunkCount || "?"} images</div>
        <button class="secondary pixel-rebuild" data-pixel-id="${escapeAttr(message.pixelId)}" data-from="${escapeAttr(message.from)}" data-to="${escapeAttr(message.to)}" data-nonce="${escapeAttr(message.nonceB64)}">Reconstruire</button>
        <div class="tiny pixel-status">Payload fragmente en images PNG.</div>
      </div>
    `;
  }
  return escapeHtml(message.body || "");
}
function blobToBase64(blob) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => resolve(String(reader.result || "").split(",")[1] || "");
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}
function base64ToBytes(value) {
  const bin = atob(value || "");
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}
function bytesToBase64(bytes) {
  let bin = "";
  const step = 0x8000;
  for (let i = 0; i < bytes.length; i += step) {
    bin += String.fromCharCode(...bytes.slice(i, i + step));
  }
  return btoa(bin);
}
function concatBytes(chunks) {
  const size = chunks.reduce((total, chunk) => total + chunk.length, 0);
  const out = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}
function xorCipher(bytes, keyText) {
  const key = new TextEncoder().encode(keyText || "twookie-pixel-demo");
  const out = new Uint8Array(bytes.length);
  for (let i = 0; i < bytes.length; i++) out[i] = bytes[i] ^ key[i % key.length];
  return out;
}
async function derivePixelKey(fromId, toId) {
  const cacheKey = `${fromId}|${toId}`;
  if (state._pixelKeyCache[cacheKey]) return state._pixelKeyCache[cacheKey];
  const material = await crypto.subtle.importKey(
    "raw",
    enc.encode(`twookie-pixel-voice|${fromId}|${toId}`),
    "PBKDF2",
    false,
    ["deriveKey"],
  );
  const key = await crypto.subtle.deriveKey(
    {name: "PBKDF2", salt: enc.encode("t-wookie-pixel-v1"), iterations: 120000, hash: "SHA-256"},
    material,
    {name: "AES-GCM", length: 256},
    false,
    ["encrypt", "decrypt"],
  );
  state._pixelKeyCache[cacheKey] = key;
  return key;
}
async function encryptPixelBytes(bytes, fromId, toId) {
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const key = await derivePixelKey(fromId, toId);
  const encrypted = new Uint8Array(await crypto.subtle.encrypt({name: "AES-GCM", iv: nonce}, key, bytes));
  return {encrypted, nonceB64: bytesToBase64(nonce)};
}
async function decryptPixelBytes(encrypted, fromId, toId, nonceB64) {
  const key = await derivePixelKey(fromId, toId);
  return new Uint8Array(await crypto.subtle.decrypt({name: "AES-GCM", iv: base64ToBytes(nonceB64)}, key, encrypted));
}
async function sha256Bytes(bytes) {
  return hex(await crypto.subtle.digest("SHA-256", bytes));
}
function u32be(value) {
  return [(value >>> 24) & 255, (value >>> 16) & 255, (value >>> 8) & 255, value & 255];
}
function readU32be(bytes, offset) {
  return ((bytes[offset] * 0x1000000) + (bytes[offset + 1] << 16) + (bytes[offset + 2] << 8) + bytes[offset + 3]) >>> 0;
}
function makePixelPacket(chunk, index, count) {
  const header = new Uint8Array([84, 80, 88, 86, 1, ...u32be(index), ...u32be(count), ...u32be(chunk.length)]);
  const packet = new Uint8Array(header.length + chunk.length);
  packet.set(header, 0);
  packet.set(chunk, header.length);
  return packet;
}
function parsePixelPacket(packet) {
  if (packet[0] !== 84 || packet[1] !== 80 || packet[2] !== 88 || packet[3] !== 86) throw new Error("bad pixel packet");
  const index = readU32be(packet, 5);
  const count = readU32be(packet, 9);
  const length = readU32be(packet, 13);
  return {index, count, chunk: packet.slice(17, 17 + length)};
}
async function packetToPngBase64(packet, width = 96, height = 96) {
  const capacity = width * height * 3;
  if (packet.length > capacity) throw new Error("chunk trop grand pour carrier PNG");
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d", {willReadFrequently: true});
  if (!ctx) throw new Error("Canvas non disponible (contexte 2D refuse par le navigateur)");
  const image = ctx.createImageData(width, height);
  for (let i = 0; i < image.data.length; i += 4) {
    image.data[i] = 17;
    image.data[i + 1] = 23;
    image.data[i + 2] = 31;
    image.data[i + 3] = 255;
  }
  for (let i = 0; i < packet.length; i++) {
    const pixel = Math.floor(i / 3);
    const channel = i % 3;
    image.data[pixel * 4 + channel] = packet[i];
  }
  ctx.putImageData(image, 0, 0);
  return canvas.toDataURL("image/png").split(",")[1] || "";
}
async function pngUrlToPacket(url, width = 96, height = 96) {
  const img = new Image();
  img.src = `${url}${url.includes("?") ? "&" : "?"}t=${Date.now()}`;
  await img.decode();
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d", {willReadFrequently: true});
  ctx.drawImage(img, 0, 0);
  const data = ctx.getImageData(0, 0, width, height).data;
  const header = new Uint8Array(17);
  for (let i = 0; i < 17; i++) header[i] = data[Math.floor(i / 3) * 4 + (i % 3)];
  const length = readU32be(header, 13);
  const packet = new Uint8Array(17 + length);
  packet.set(header, 0);
  for (let i = 17; i < packet.length; i++) packet[i] = data[Math.floor(i / 3) * 4 + (i % 3)];
  return packet;
}
async function addContact() {
  if (!$("owner").value || !$("peer").value || $("owner").value === $("peer").value) throw new Error("Choisis deux users differents");
  await api("/api/twookie/contacts", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({owner_id: $("owner").value, peer_id: $("peer").value})});
  state.owner = $("owner").value;
  localStorage.setItem("twookie.owner", state.owner);
  setActivePeer($("peer").value);
  setStatus(`Canal ajoute: ${state.owner} -> ${state.activePeer}`);
}
async function createInvite() {
  if (!state.owner) throw new Error("Connecte ton compte avant de creer une invitation");
  const label = $("invite-label").value.trim() || `Invitation de ${state.owner}`;
  await loadPublicSession();
  const data = await api("/api/twookie/invites", {
    method: "POST",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({inviter_id: state.owner, label, ttl_seconds: 86400})
  });
  const origin = inviteOrigin();
  const link = `${origin}/?invite=${encodeURIComponent(data.invite.token)}`;
  const qr = `/api/twookie/invites/${encodeURIComponent(data.invite.token)}/qr?origin=${encodeURIComponent(origin)}`;
  state.inviteLink = link;
  state.inviteQr = qr;
  $("invite-link").textContent = link;
  $("invite-qr").src = qr;
  $("invite-qr").hidden = false;
  showShareModal();
  setStatus(state.publicOrigin ? "Invitation publique ngrok creee" : "Invitation locale creee");
}
async function copyInvite() {
  if (!state.inviteLink) throw new Error("Cree une invitation d'abord");
  await navigator.clipboard.writeText(state.inviteLink);
  setStatus("Lien invitation copie");
}
function openInviteQr() {
  if (!state.inviteQr) throw new Error("Cree une invitation d'abord");
  window.open(state.inviteQr, "_blank", "noopener");
}
async function createQuickInvite() {
  if (!$("invite-label").value.trim()) $("invite-label").value = `Invitation de ${state.owner || "T-Wookie"}`;
  await createInvite();
}
async function acceptInvite() {
  if (!state.inviteToken) throw new Error("Aucune invitation active");
  const id = $("handle").value.trim().toLowerCase().replace(/[^a-z0-9_.-]/g, "-") || state.owner;
  if (!id) throw new Error("Entre un handle avant d'accepter");
  const displayName = $("display").value.trim() || id;
  const hardware = hardwareLog($("consent").checked);
  const data = await acceptInviteWithPayload(await buildAccountPayload(id, displayName, hardware));
  state.inviteReady = true;
  setActivePeer(data.peer_id, false);
  setStatus(`Invitation acceptee: canal avec ${data.peer_id}`);
  await refresh();
}
async function sendMessage() {
  const body = $("message").value.trim();
  if (!state.owner) throw new Error("Connecte un compte avant d'envoyer");
  if (!state.activePeer) throw new Error("Choisis un contact actif");
  if (!body) return;
  const result = await api("/api/twookie/messages", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({from: state.owner, to: state.activePeer, body})});
  const node = result.intent?.node?.id || result.intent?.node?.payload_hash || "";
  const bridge = result.intent?.bridge || {};
  const source = state.activePeer.startsWith("discord-") ? "twookie_web" : "local_web";
  state.lastGlobalBridgeId = Math.max(state.lastGlobalBridgeId, Number(bridge.id || 0));
  state.lastBridgeId = Math.max(state.lastBridgeId, Number(bridge.id || 0));
  state.messages.push({from: state.owner, to: state.activePeer, body, at: Date.now(), node: bridge.status || node, source: bridge.source || source, bridgeId: bridge.id});
  saveMessages();
  $("message").value = "";
  setStatus(`Message prepare pour ${state.activePeer}`);
  renderMessages();
  await refreshNodes();
}
async function sendVoiceMemo(blob, durationMs) {
  if (!state.owner) throw new Error("Connecte un compte avant d'envoyer");
  if (!state.activePeer) throw new Error("Choisis un contact actif");
  if (!blob || !blob.size) throw new Error("Memo vocal vide");
  if (durationMs > 10500) throw new Error("Memo vocal limite a 10 secondes");
  const rawBytes = base64ToBytes(await blobToBase64(blob));
  const encryptedPack = await encryptPixelBytes(rawBytes, state.owner, state.activePeer);
  const encrypted = encryptedPack.encrypted;
  const width = 96;
  const height = 96;
  const chunkSize = width * height * 3 - 17;
  const chunkCount = Math.ceil(encrypted.length / chunkSize);
  const payloadHash = await sha256Bytes(encrypted);
  const chunks = [];
  for (let index = 0; index < chunkCount; index++) {
    const chunk = encrypted.slice(index * chunkSize, Math.min(encrypted.length, (index + 1) * chunkSize));
    const packet = makePixelPacket(chunk, index, chunkCount);
    chunks.push({
      index,
      chunk_hash: await sha256Bytes(chunk),
      png_base64: await packetToPngBase64(packet, width, height),
    });
  }
  const result = await api("/api/twookie/messages", {
    method: "POST",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({
      from: state.owner,
      to: state.activePeer,
      kind: "pixel_voice",
      body: "[pixel voice memo]",
      chunks,
      payload_hash: payloadHash,
      width,
      height,
      cipher: "aes-256-gcm",
      nonce_b64: encryptedPack.nonceB64,
      mime_type: blob.type || "audio/webm",
      duration_ms: durationMs,
    })
  });
  const bridge = result.intent?.bridge || {};
  const meta = parseBridgeMeta(bridge);
  state.lastGlobalBridgeId = Math.max(state.lastGlobalBridgeId, Number(bridge.id || 0));
  state.lastBridgeId = Math.max(state.lastBridgeId, Number(bridge.id || 0));
  state.messages.push({
    from: state.owner,
    to: state.activePeer,
    body: "[voice memo]",
    at: Date.now(),
    node: bridge.status || "delivered",
    source: bridge.source || "local_web",
    bridgeId: bridge.id,
    kind: "pixel_voice_memo",
    pixelId: meta.pixel_id || "",
    chunkCount: Number(meta.chunk_count || chunkCount),
    durationMs: Number(meta.duration_ms || durationMs),
    cipher: meta.cipher || "aes-256-gcm",
    nonceB64: meta.nonce_b64 || encryptedPack.nonceB64,
  });
  saveMessages();
  setStatus(`Memo vocal envoye a ${state.activePeer}`);
  renderMessages();
  await refreshNodes();
}
function preferredAudioMime() {
  const candidates = ["audio/webm;codecs=opus", "audio/webm", "audio/mp4", "audio/aac"];
  return candidates.find((mime) => window.MediaRecorder?.isTypeSupported?.(mime)) || "";
}
async function startVoiceMemo() {
  if (!navigator.mediaDevices?.getUserMedia || !window.MediaRecorder) throw new Error("Micro non supporte par ce navigateur");
  if (!state.owner || !state.activePeer) throw new Error("Canal actif requis");
  const stream = await navigator.mediaDevices.getUserMedia({audio: true});
  const mimeType = preferredAudioMime();
  const recorder = new MediaRecorder(stream, mimeType ? {mimeType} : undefined);
  state.recordChunks = [];
  state.recordStartedAt = Date.now();
  state.recorder = recorder;
  $("ptt").classList.add("recording");
  $("ptt").title = "Arreter";
  recorder.addEventListener("dataavailable", (event) => {
    if (event.data && event.data.size) state.recordChunks.push(event.data);
  });
  recorder.addEventListener("stop", async () => {
    const durationMs = Math.min(Date.now() - state.recordStartedAt, 10000);
    stream.getTracks().forEach((track) => track.stop());
    clearTimeout(state.recordTimer);
    state.recordTimer = null;
    state.recorder = null;
    $("ptt").classList.remove("recording");
    $("ptt").title = "Micro";
    try {
      const blob = new Blob(state.recordChunks, {type: recorder.mimeType || mimeType || "audio/webm"});
      state.recordChunks = [];
      await sendVoiceMemo(blob, durationMs);
    } catch (error) {
      setStatus(error.message);
    }
  });
  recorder.start();
  setStatus("Enregistrement vocal... max 10s");
  state.recordTimer = setTimeout(() => stopVoiceMemo(), 10000);
}
function stopVoiceMemo() {
  if (state.recorder && state.recorder.state !== "inactive") {
    state.recorder.stop();
  }
}
async function toggleVoiceMemo() {
  if (state.recorder && state.recorder.state === "recording") {
    stopVoiceMemo();
  } else {
    await startVoiceMemo();
  }
}
async function purgeConversation() {
  if (!state.owner || !state.activePeer) throw new Error("Canal actif requis");
  const peer = state.activePeer;
  await api("/api/twookie/bridge/purge", {
    method: "POST",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({owner: state.owner, peer}),
  });
  state.messages = state.messages.filter((m) => !((m.from === state.owner && m.to === peer) || (m.from === peer && m.to === state.owner)));
  state.lastBridgeId = 0;
  state.lastGlobalBridgeId = 0;
  saveMessages();
  renderMessages();
  setStatus(`Canal efface avec ${peer}`);
}
async function rebuildPixelVoice(pixelId, button = null) {
  if (!pixelId) throw new Error("payload pixel manquant");
  const card = button?.closest(".pixel-voice") || document.querySelector(`[data-pixel-id="${CSS.escape(pixelId)}"]`);
  const status = card?.querySelector(".pixel-status");
  if (status) status.textContent = "Reconstruction des images...";
  const data = await api(`/api/twookie/pixel-payloads/${encodeURIComponent(pixelId)}/manifest`);
  const payload = data.payload;
  const packets = [];
  for (const chunk of data.chunks || []) {
    if (status) status.textContent = `Lecture image ${Number(chunk.index) + 1}/${data.chunks.length}`;
    const packet = await pngUrlToPacket(chunk.url);
    const parsed = parsePixelPacket(packet);
    const hash = await sha256Bytes(parsed.chunk);
    if (hash !== chunk.chunk_hash) throw new Error(`checksum chunk ${chunk.index} invalide`);
    packets[parsed.index] = parsed.chunk;
  }
  const encrypted = concatBytes(packets);
  const payloadHash = await sha256Bytes(encrypted);
  if (payloadHash !== payload.payload_hash) throw new Error("checksum payload invalide");
  const nonceB64 = button?.dataset?.nonce || parseMetaJson(payload.meta_json).nonce_b64 || "";
  const raw = await decryptPixelBytes(encrypted, payload.from_id, payload.to_id, nonceB64);
  const audioBlob = new Blob([raw], {type: payload.mime_type || "audio/webm"});
  const audioUrl = URL.createObjectURL(audioBlob);
  if (card) {
    card.innerHTML = `
      <div class="voice-label">Memo vocal pixel reconstruit · ${data.chunks.length} images</div>
      <audio controls preload="metadata" src="${audioUrl}"></audio>
      <div class="tiny">Payload verifie puis dechiffre localement.</div>
    `;
  }
  setStatus(`Memo pixel reconstruit: ${data.chunks.length} images`);
}
async function startCall() {
  if (!state.owner || !state.activePeer) throw new Error("Canal actif requis");
  await api("/api/twookie/calls", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({from: state.owner, to: state.activePeer, mode: "signal"})});
  setStatus(`Signal appel cree vers ${state.activePeer}`);
}
async function addNode() {
  const payloadHash = await sha256(`twookie-node|${crypto.randomUUID()}|${Date.now()}`);
  await api("/api/twookie/image-nodes", {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify({payload_hash: payloadHash, meta: {source: "twookie-talkie-ui"}})});
  setStatus(`Node image ajoute ${shortId(payloadHash)}`);
  await refreshNodes();
}
async function refreshNodes() {
  const data = await api("/api/twookie/image-nodes");
  const nodes = data.nodes || [];
  $("nodes").textContent = nodes.slice(0, 7).map((node) => `${shortId(node.id)}  ${shortId(node.payload_hash)}  ${node.created_at}`).join("\\n") || "Aucun node";
}

$("create-account").addEventListener("click", () => createAccount().catch((error) => setStatus(error.message)));
$("entry-connect").addEventListener("click", () => entryConnect().catch((error) => {
  $("entry-note").textContent = error.message;
  setStatus(error.message);
}));
$("entry-guest").addEventListener("click", () => entryGuest().catch((error) => {
  $("entry-note").textContent = error.message;
  setStatus(error.message);
}));
$("entry-existing").addEventListener("click", continueExistingSession);
$("entry-handle").addEventListener("input", () => { $("handle").value = $("entry-handle").value; });
$("entry-display").addEventListener("input", () => { $("display").value = $("entry-display").value; });
setAppHeight();
window.visualViewport?.addEventListener("resize", setAppHeight);
window.visualViewport?.addEventListener("scroll", setAppHeight);
window.addEventListener("orientationchange", () => setTimeout(setAppHeight, 250));
document.querySelectorAll("[data-lang]").forEach((button) => button.addEventListener("click", () => setLanguage(button.dataset.lang)));
$("change-account").addEventListener("click", switchAccount);
$("account-invite").addEventListener("click", () => createQuickInvite().catch((error) => setStatus(error.message)));
$("back-to-list").addEventListener("click", closeMobileChat);
window.addEventListener("resize", () => {
  if (!isMobile()) closeMobileChat();
});
$("search").addEventListener("input", renderChannels);
$("channels").addEventListener("click", (event) => {
  const button = event.target.closest("[data-peer]");
  if (button) setActivePeer(button.dataset.peer);
});
$("add-contact").addEventListener("click", () => addContact().catch((error) => setStatus(error.message)));
$("create-invite").addEventListener("click", () => createInvite().catch((error) => setStatus(error.message)));
$("copy-invite").addEventListener("click", () => copyInvite().catch((error) => setStatus(error.message)));
$("open-qr").addEventListener("click", () => {
  try { showShareModal(); } catch (error) { setStatus(error.message); }
});
$("top-invite").addEventListener("click", () => createQuickInvite().catch((error) => setStatus(error.message)));
$("share-copy").addEventListener("click", () => copyInvite().catch((error) => setStatus(error.message)));
$("share-open").addEventListener("click", () => {
  try { openInviteQr(); } catch (error) { setStatus(error.message); }
});
$("share-close").addEventListener("click", hideShareModal);
$("share-modal").addEventListener("click", (event) => {
  if (event.target === $("share-modal")) hideShareModal();
});
$("accept-invite").addEventListener("click", () => acceptInvite().catch((error) => setStatus(error.message)));
$("send-message").addEventListener("click", () => sendMessage().catch((error) => setStatus(error.message)));
$("timeline").addEventListener("click", (event) => {
  const button = event.target.closest(".pixel-rebuild");
  if (button) rebuildPixelVoice(button.dataset.pixelId, button).catch((error) => setStatus(error.message));
});
$("message").addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    sendMessage().catch((error) => setStatus(error.message));
  }
});
$("message").addEventListener("focus", () => setTimeout(setAppHeight, 80));
$("message").addEventListener("blur", () => setTimeout(setAppHeight, 120));
$("call-peer").addEventListener("click", () => startCall().catch((error) => setStatus(error.message)));
$("start-call").addEventListener("click", () => startCall().catch((error) => setStatus(error.message)));
$("ptt").addEventListener("click", () => toggleVoiceMemo().catch((error) => setStatus(error.message)));
$("repo-refresh").addEventListener("click", () => syncConversation().catch((error) => setStatus(error.message)));
$("purge-chat").addEventListener("click", () => purgeConversation().catch((error) => setStatus(error.message)));
$("add-node").addEventListener("click", () => addNode().catch((error) => setStatus(error.message)));
$("switch-first").addEventListener("click", () => {
  const peers = state.accounts.filter((item) => item.id !== state.owner);
  if (!peers.length) return setStatus("Aucun peer disponible");
  const index = Math.max(0, peers.findIndex((item) => item.id === state.activePeer));
  setActivePeer(peers[(index + 1) % peers.length].id);
});
$("mobile-request").addEventListener("click", () => {
  document.querySelector(".rail").scrollIntoView({behavior: "smooth", block: "start"});
});
refresh().catch((error) => setStatus(error.message));
setInterval(() => pollUpdates().catch((error) => setStatus(error.message)), 2500);
</script>
</html>
"""


def room_page(room_id: str) -> str:
    return f"""<!doctype html>
<html lang="fr">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>T-Bacon Room {room_id}</title>
<style>
:root {{ color-scheme: dark; font-family: Inter, Segoe UI, system-ui, sans-serif; background: #101417; color: #edf4f7; }}
body {{ margin: 0; background: #101417; }}
main {{ max-width: 1100px; margin: auto; padding: 24px; }}
header {{ display: flex; justify-content: space-between; border-bottom: 1px solid #2d3940; padding-bottom: 16px; }}
h1 {{ margin: 0; font-size: 28px; }}
.status {{ border: 1px solid #537a6e; padding: 8px 12px; color: #a5f2d1; font-weight: 700; }}
.grid {{ display: grid; grid-template-columns: 360px 1fr; gap: 18px; margin-top: 18px; }}
section {{ border: 1px solid #2d3940; background: #151b20; padding: 16px; }}
input, select, textarea {{ width: 100%; box-sizing: border-box; background: #0d1114; color: white; border: 1px solid #34434b; padding: 8px; margin: 6px 0; }}
textarea {{ min-height: 100px; }}
button {{ width: 100%; border: 1px solid #537a6e; background: #214d42; color: #f1fffb; padding: 9px; font-weight: 700; cursor: pointer; margin-top: 8px; }}
canvas, img {{ width: 100%; image-rendering: pixelated; background: #000; border: 1px solid #34434b; box-sizing: border-box; }}
.chat {{ min-height: 260px; max-height: 430px; overflow: auto; background: #0d1114; border: 1px solid #34434b; padding: 10px; }}
.msg {{ border-bottom: 1px solid #26323a; padding: 8px 0; }}
.meta {{ color: #9fb0b8; font-size: 12px; }}
@media (max-width: 820px) {{ .grid {{ display: block; }} section {{ margin-top: 14px; }} }}
</style>
<main>
<header><h1>T-Bacon Room: {room_id}</h1><div class="status" id="status">BOOT</div></header>
<div class="grid">
  <section>
    <label>User</label>
    <select id="user"></select>
    <label>Shared room secret</label>
    <input id="room-secret" type="password" value="demo-high-entropy-room-secret-change-me">
    <label>Message</label>
    <textarea id="message">Salut via T-Bacon + TPXV.</textarea>
    <button id="send">Encode TPXV Pixels + Send</button>
    <button id="proof">Local Encode/Decode Proof</button>
    <p id="info"></p>
    <canvas id="carrier" width="640" height="360"></canvas>
  </section>
  <section>
    <div class="chat" id="chat"></div>
  </section>
</div>
</main>
<script>
const roomId = "{room_id}";
const MAGIC = [0x54,0x50,0x58,0x56,0x43,0x48,0x41,0x54];
const encoder = new TextEncoder();
const decoder = new TextDecoder();
let users = [];
let carrier = null;
let lastId = 0;
let baseImage = null;
let chainRoot = null;
let messageCounter = 1;

function bytesToBase64(bytes) {{
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}}
function base64ToBytes(value) {{
  const binary = atob(value);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
  return out;
}}
async function sha256Hex(bytes) {{
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest)).map(b => b.toString(16).padStart(2, "0")).join("");
}}
async function deriveChainRoot() {{
  const sample = carrier.mask.positions.slice(0, 256).map(pos => `${{pos.x}}:${{pos.y}}`).join("|");
  return await sha256Hex(encoder.encode(`tbacon-browser-chain-v1|${{roomId}}|${{carrier.mask.mask_id}}|${{carrier.mask.positions.length}}|${{sample}}`));
}}
async function deriveMessageKey() {{
  const secret = document.querySelector("#room-secret").value;
  if (secret.length < 16) throw new Error("shared secret too short");
  const baseKey = await crypto.subtle.importKey("raw", encoder.encode(secret), "HKDF", false, ["deriveKey"]);
  return await crypto.subtle.deriveKey(
    {{
      name: "HKDF",
      hash: "SHA-256",
      salt: encoder.encode(`tbacon-room-key-v1|${{roomId}}|${{carrier.mask.mask_id}}|${{chainRoot}}`),
      info: encoder.encode("message_key"),
    }},
    baseKey,
    {{name: "AES-GCM", length: 256}},
    false,
    ["encrypt", "decrypt"]
  );
}}
async function encryptPayload(payload) {{
  const key = await deriveMessageKey();
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const aad = {{room_id: roomId, mask_id: carrier.mask.mask_id, chain_root: chainRoot, sender: payload.from, counter: messageCounter++}};
  const ciphertext = new Uint8Array(await crypto.subtle.encrypt(
    {{name: "AES-GCM", iv: nonce, additionalData: encoder.encode(JSON.stringify(aad))}},
    key,
    encoder.encode(JSON.stringify(payload))
  ));
  return {{type: "tbacon_encrypted_v1", version: 1, alg: "AES-GCM", aad, nonce: bytesToBase64(nonce), ciphertext: bytesToBase64(ciphertext)}};
}}
async function decryptPayload(envelope) {{
  if (envelope.type !== "tbacon_encrypted_v1") return envelope;
  const key = await deriveMessageKey();
  const plaintext = await crypto.subtle.decrypt(
    {{name: "AES-GCM", iv: base64ToBytes(envelope.nonce), additionalData: encoder.encode(JSON.stringify(envelope.aad))}},
    key,
    base64ToBytes(envelope.ciphertext)
  );
  return JSON.parse(decoder.decode(plaintext));
}}

function crc32(bytes) {{
  let c = 0xFFFFFFFF;
  for (const byte of bytes) {{
    c ^= byte;
    for (let i = 0; i < 8; i += 1) c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
  }}
  return (c ^ 0xFFFFFFFF) >>> 0;
}}
function u32be(value) {{ return [(value>>>24)&255,(value>>>16)&255,(value>>>8)&255,value&255]; }}
function readU32be(bytes, offset) {{ return ((bytes[offset]*0x1000000)+(bytes[offset+1]<<16)+(bytes[offset+2]<<8)+bytes[offset+3])>>>0; }}
function bytesToBits(bytes) {{ const bits=[]; for (const byte of bytes) for (let s=7;s>=0;s--) bits.push((byte>>>s)&1); return bits; }}
function bitsToBytes(bits) {{
  const out = new Uint8Array(Math.ceil(bits.length/8));
  for (let i=0;i<bits.length;i+=8) {{ let b=0; for (let j=0;j<8;j++) b=(b<<1)|(bits[i+j]||0); out[i/8]=b; }}
  return out;
}}
function buildPacket(payload) {{
  const body = encoder.encode(JSON.stringify(payload));
  const header = new Uint8Array([...MAGIC, ...u32be(body.length)]);
  const crcInput = new Uint8Array(header.length + body.length);
  crcInput.set(header, 0); crcInput.set(body, header.length);
  const packet = new Uint8Array(header.length + 4 + body.length);
  packet.set(header, 0); packet.set(u32be(crc32(crcInput)), header.length); packet.set(body, header.length + 4);
  return packet;
}}
function parsePacket(packet) {{
  for (let i=0;i<MAGIC.length;i++) if (packet[i] !== MAGIC[i]) throw new Error("bad magic");
  const len = readU32be(packet, 8);
  const expected = readU32be(packet, 12);
  const body = packet.slice(16, 16 + len);
  const crcInput = new Uint8Array(12 + body.length);
  crcInput.set(packet.slice(0, 12), 0); crcInput.set(body, 12);
  if (crc32(crcInput) !== expected) throw new Error("CRC mismatch");
  return JSON.parse(decoder.decode(body));
}}
function drawBase(ctx) {{ ctx.drawImage(baseImage, 0, 0, 640, 360); }}
function encode(payload) {{
  const packet = buildPacket(payload);
  const bits = bytesToBits(packet);
  if (bits.length > carrier.mask.positions.length) throw new Error("message too large");
  const canvas = document.querySelector("#carrier");
  const ctx = canvas.getContext("2d", {{willReadFrequently:true}});
  drawBase(ctx);
  const image = ctx.getImageData(0,0,640,360);
  for (let i=0;i<bits.length;i++) {{
    const pos = carrier.mask.positions[i];
    const v = bits[i] ? 240 : 16;
    for (let dy=-1;dy<=1;dy++) for (let dx=-1;dx<=1;dx++) {{
      const offset = ((pos.y+dy)*640 + pos.x+dx)*4;
      image.data[offset]=v; image.data[offset+1]=v; image.data[offset+2]=v; image.data[offset+3]=255;
    }}
  }}
  ctx.putImageData(image, 0, 0);
  return canvas.toDataURL("image/png");
}}
async function decode(dataUrl) {{
  const img = new Image(); img.src = dataUrl; await img.decode();
  const canvas = document.createElement("canvas"); canvas.width=640; canvas.height=360;
  const ctx = canvas.getContext("2d", {{willReadFrequently:true}}); ctx.drawImage(img,0,0);
  const data = ctx.getImageData(0,0,640,360).data;
  const bits = [];
  for (const pos of carrier.mask.positions) {{
    let total = 0;
    for (let dy=-1;dy<=1;dy++) for (let dx=-1;dx<=1;dx++) {{
      const offset = ((pos.y+dy)*640 + pos.x+dx)*4;
      total += data[offset]+data[offset+1]+data[offset+2];
    }}
    bits.push(total / 27 >= 128 ? 1 : 0);
    if (bits.length >= 128) {{
      const header = bitsToBytes(bits.slice(0,128));
      if (MAGIC.every((v,i) => header[i] === v)) {{
        const len = readU32be(header, 8);
        const needed = (16 + len) * 8;
        if (bits.length >= needed) return parsePacket(bitsToBytes(bits.slice(0, needed)));
      }}
    }}
  }}
  throw new Error("incomplete packet");
}}
function addMessage(payload, direction) {{
  const el = document.createElement("div"); el.className = "msg";
  el.innerHTML = `<div>${{payload.message}}</div><div class="meta">${{direction}} Â· ${{payload.from}} -> room ${{roomId}} Â· ${{new Date(payload.sentAt).toLocaleTimeString()}}</div>`;
  document.querySelector("#chat").append(el); el.scrollIntoView();
}}
async function send() {{
  const user = document.querySelector("#user").value;
  const payload = {{type:"tbacon_tpxv_chat", version:1, room:roomId, from:user, message:document.querySelector("#message").value, sentAt:new Date().toISOString()}};
  const encodedPng = encode(await encryptPayload(payload));
  await fetch(`/api/rooms/${{roomId}}/pixel-messages`, {{method:"POST", headers:{{"Content-Type":"application/json"}}, body:JSON.stringify({{user_id:user, encodedPng}})}}).then(r=>r.json()).then(d=>{{ if(!d.ok) throw new Error(d.error); }});
  addMessage(payload, "sent");
  document.querySelector("#status").textContent = "SENT ENCRYPTED";
}}
async function poll() {{
  if (!carrier) return;
  const data = await fetch(`/api/rooms/${{roomId}}/messages?after=${{lastId}}`).then(r=>r.json());
  for (const msg of data.messages) {{
    lastId = Math.max(lastId, msg.id);
    if (msg.transport !== "tpxv_png") continue;
    if (msg.user_id === document.querySelector("#user").value) continue;
    try {{
      const payload = await fetch(msg.payload_url).then(r=>r.json());
      if (!payload.ok) throw new Error(payload.error || "payload failed");
      addMessage(await decryptPayload(await decode(payload.body)), "received encrypted");
      document.querySelector("#status").textContent = "RECEIVED ENCRYPTED";
    }}
    catch (error) {{ document.querySelector("#status").textContent = "DECODE FAIL"; }}
  }}
}}
async function boot() {{
  users = (await fetch("/api/users").then(r=>r.json())).users;
  document.querySelector("#user").innerHTML = users.map(u => `<option value="${{u.id}}">${{u.id}}</option>`).join("");
  carrier = await fetch(`/api/rooms/${{roomId}}/carrier`).then(r=>r.json());
  if (!carrier.ok) throw new Error(carrier.error);
  chainRoot = await deriveChainRoot();
  baseImage = new Image(); baseImage.src = carrier.carrier_frame_url; await baseImage.decode();
  drawBase(document.querySelector("#carrier").getContext("2d"));
  document.querySelector("#info").textContent = `mask ${{carrier.mask.mask_id}} Â· ${{carrier.mask.positions.length}} positions Â· encrypted`;
  document.querySelector("#status").textContent = "READY";
  setInterval(() => poll().catch(()=>{{}}), 1000);
}}
document.querySelector("#send").addEventListener("click", () => send().catch(e => document.querySelector("#status").textContent = e.message));
document.querySelector("#proof").addEventListener("click", async () => {{
  const payload = {{type:"tbacon_tpxv_chat", version:1, room:roomId, from:"alice", message:"preuve locale T-Bacon + TPXV", sentAt:new Date().toISOString()}};
  const decoded = await decryptPayload(await decode(encode(await encryptPayload(payload))));
  addMessage(decoded, "local proof");
  document.querySelector("#status").textContent = "PASS";
}});
boot().catch(e => document.querySelector("#status").textContent = e.message);
</script>
</html>
"""


def _render_security_report(r: dict) -> str:
    import datetime
    def fmt_ts(ts):
        if not ts:
            return "-"
        return datetime.datetime.fromtimestamp(ts).strftime("%Y-%m-%d %H:%M:%S")

    ban_rows = "".join(
        f"<tr><td>{b['ip']}</td><td>{'AUTO' if b['auto'] else 'MANUAL'}</td>"
        f"<td>{b['reason']}</td><td>{fmt_ts(b['banned_at'])}</td>"
        f"<td>{'permanent' if not b['expires_at'] else fmt_ts(b['expires_at'])}</td>"
        f"<td><button onclick=\"unban('{b['ip']}')\">Unban</button></td></tr>"
        for b in r["bans"]
    )
    ip_rows = "".join(
        f"<tr class=\"{'danger' if row['recon_hits'] else ''}\"><td>{row['ip']}</td>"
        f"<td>{row['hits']}</td><td>{row['recon_hits']}</td>"
        f"<td>{row['not_found']}</td><td>{fmt_ts(row['last_seen'])}</td>"
        f"<td><button onclick=\"ban('{row['ip']}')\">Ban</button></td></tr>"
        for row in r["top_ips"]
    )
    recon_rows = "".join(
        f"<tr><td>{fmt_ts(row['ts'])}</td><td>{row['ip']}</td>"
        f"<td>{row['method']}</td><td style='word-break:break-all'>{row['path']}</td>"
        f"<td style='font-size:.75rem;color:#8b949e'>{(row['ua'] or '')[:60]}</td></tr>"
        for row in r["recent_recon"]
    )
    return f"""<!DOCTYPE html><html><head><meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1">
<title>T-Wookie Security</title>
<style>
*{{box-sizing:border-box;}}
body{{font-family:monospace;background:#0d1117;color:#e6edf3;margin:0;padding:1.5rem;}}
h1{{color:#f85149;}}h2{{color:#58a6ff;margin-top:2rem;border-bottom:1px solid #30363d;padding-bottom:.4rem;}}
.stats{{display:flex;gap:1.5rem;flex-wrap:wrap;margin:1rem 0;}}
.stat{{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:1rem 1.5rem;min-width:160px;}}
.stat .val{{font-size:2rem;font-weight:bold;color:#58a6ff;}}
.stat .lbl{{font-size:.8rem;color:#8b949e;}}
table{{width:100%;border-collapse:collapse;font-size:.85rem;}}
th{{background:#161b22;color:#8b949e;text-align:left;padding:.4rem .6rem;position:sticky;top:0;}}
td{{padding:.35rem .6rem;border-bottom:1px solid #21262d;}}
tr.danger td{{color:#f85149;}}
button{{background:#21262d;color:#e6edf3;border:1px solid #30363d;border-radius:4px;padding:.2rem .6rem;cursor:pointer;font-size:.8rem;}}
button:hover{{background:#f85149;color:#fff;border-color:#f85149;}}
.ban-form{{display:flex;gap:.5rem;margin-top:.5rem;}}
.ban-form input{{background:#161b22;color:#e6edf3;border:1px solid #30363d;border-radius:4px;padding:.3rem .6rem;font-family:monospace;}}
#msg{{margin-top:.5rem;color:#3fb950;min-height:1.2em;}}
</style></head><body>
<h1>&#128737; T-Wookie Security Report</h1>
<div class=stats>
  <div class=stat><div class=val>{r["total_requests"]}</div><div class=lbl>Total requests</div></div>
  <div class=stat><div class=val>{r["requests_1h"]}</div><div class=lbl>Last hour</div></div>
  <div class=stat><div class=val style="color:#f85149">{r["recon_24h"]}</div><div class=lbl>Recon probes 24h</div></div>
  <div class=stat><div class=val style="color:#f85149">{len(r["bans"])}</div><div class=lbl>Banned IPs</div></div>
</div>

<h2>Ban list</h2>
<table><tr><th>IP</th><th>Type</th><th>Reason</th><th>Banned at</th><th>Expires</th><th>Action</th></tr>
{ban_rows or '<tr><td colspan=6 style="color:#8b949e">No bans</td></tr>'}
</table>
<div class=ban-form>
  <input id=banip placeholder="IP to ban manually" size=20>
  <input id=banreason placeholder="Reason" size=30>
  <button onclick="banManual()">Ban IP</button>
</div>
<div id=msg></div>

<h2>Top IPs — last 24h <span style="font-size:.8rem;color:#8b949e">(red = recon detected)</span></h2>
<table><tr><th>IP</th><th>Hits</th><th>Recon</th><th>404s</th><th>Last seen</th><th>Action</th></tr>
{ip_rows or '<tr><td colspan=6 style="color:#8b949e">No data</td></tr>'}
</table>

<h2>Recent recon probes</h2>
<table><tr><th>Time</th><th>IP</th><th>Method</th><th>Path</th><th>UA</th></tr>
{recon_rows or '<tr><td colspan=5 style="color:#8b949e">No recon detected</td></tr>'}
</table>

<script>
async function ban(ip) {{
  const r = await fetch('/admin/ban', {{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{ip,reason:'manual ban from report'}})}});
  const d = await r.json();
  document.getElementById('msg').textContent = d.ok ? 'Banned: '+ip : 'Error: '+d.error;
  setTimeout(()=>location.reload(), 1200);
}}
async function unban(ip) {{
  const r = await fetch('/admin/unban', {{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{ip}})}});
  const d = await r.json();
  document.getElementById('msg').textContent = d.ok ? 'Unbanned: '+ip : 'Error: '+d.error;
  setTimeout(()=>location.reload(), 1200);
}}
function banManual() {{
  const ip = document.getElementById('banip').value.trim();
  const reason = document.getElementById('banreason').value.trim() || 'manual';
  if (ip) ban(ip);
}}
</script>
</body></html>"""


def make_handler(hub: Hub):
    class Handler(BaseHTTPRequestHandler):
        def json_body(self) -> dict:
            length = int(self.headers.get("Content-Length", "0"))
            if length == 0:
                return {}
            if length > MAX_JSON_BYTES:
                raise ValueError("json body too large")
            content_type = self.headers.get("Content-Type", "")
            if "application/json" not in content_type:
                raise ValueError("Content-Type must be application/json")
            try:
                body = json.loads(self.rfile.read(length).decode("utf-8"))
            except Exception as exc:
                raise ValueError("invalid json body") from exc
            if not isinstance(body, dict):
                raise ValueError("json object required")
            return body

        def send_json(self, status: int, payload: dict) -> None:
            body = json.dumps(payload, indent=2).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Access-Control-Allow-Headers", "Content-Type, X-TBacon-Bridge-Token, Idempotency-Key")
            self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def send_png(self, status: int) -> None:
            self.send_response(status)
            self.send_header("Content-Type", "image/png")
            self.send_header("Cache-Control", "no-store, max-age=0")
            self.send_header("Content-Length", str(len(PNG_1X1)))
            self.end_headers()
            self.wfile.write(PNG_1X1)

        def bridge_authorized(self) -> bool:
            token = os.getenv("TBACON_BRIDGE_TOKEN", "").strip()
            if not token:
                return True
            return self.headers.get("X-TBacon-Bridge-Token", "") == token

        def do_OPTIONS(self) -> None:
            self.send_response(204)
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Access-Control-Allow-Headers", "Content-Type, X-TBacon-Bridge-Token, Idempotency-Key")
            self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            self.send_header("Cache-Control", "no-store")
            self.end_headers()

        def do_GET(self) -> None:
            if self._sec_check("GET"):
                return
            parsed = urlparse(self.path)
            if parsed.path == "/" or parsed.path == "/twookie":
                body = twookie_dashboard_v2().encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if parsed.path == "/ai-context":
                origin = f"{'https' if self.headers.get('X-Forwarded-Proto') == 'https' else 'http'}://{self.headers.get('Host', '127.0.0.1:8894')}"
                body = twookie_ai_context_page(origin).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if parsed.path == "/api/twookie/context":
                origin = f"{'https' if self.headers.get('X-Forwarded-Proto') == 'https' else 'http'}://{self.headers.get('Host', '127.0.0.1:8894')}"
                self.send_json(200, {"ok": True, "context": twookie_context_pack(origin)})
                return
            if parsed.path in {"/api/twookie/public-url", "/api/twookie/public_url", "/api/twookie/ngrok"}:
                self.send_json(200, {"ok": True, "session": twookie_public_session()})
                return
            if parsed.path in {"/api/v1/server/health", "/api/twookie/server/health"}:
                self.send_json(200, hub.server_health())
                return
            if parsed.path in {"/api/v1/server/routes", "/api/twookie/server/routes"}:
                self.send_json(200, hub.route_contract())
                return
            if parsed.path == "/assets/twookie-logo.png":
                logo_path = Path("reports/tbacon/latest/twookie-logo.png")
                if not logo_path.exists():
                    self.send_json(404, {"ok": False, "error": "logo not found"})
                    return
                data = logo_path.read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "image/png")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            if parsed.path == "/assets/twookie-mic.png":
                mic_path = Path("reports/tbacon/latest/twookie-mic.png")
                if not mic_path.exists():
                    self.send_json(404, {"ok": False, "error": "mic not found"})
                    return
                data = mic_path.read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "image/png")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            if parsed.path == "/download":
                apk_path = Path(__file__).parent.parent / "twookie-native/android/app/build/outputs/apk/debug/app-debug.apk"
                host = self.headers.get("X-Forwarded-Host") or self.headers.get("Host") or "localhost:8895"
                scheme = "https" if "ngrok" in host else "http"
                apk_url = f"{scheme}://{host}/download/twookie.apk"
                apk_size = f"{apk_path.stat().st_size // 1024 // 1024} MB" if apk_path.exists() else "not built"
                import io as _io
                qr = qrcode.QRCode(box_size=6, border=2)
                qr.add_data(apk_url)
                qr.make(fit=True)
                img = qr.make_image(fill_color="black", back_color="white")
                buf = _io.BytesIO()
                img.save(buf, format="PNG")
                import base64 as _b64
                qr_b64 = _b64.b64encode(buf.getvalue()).decode()
                html = f"""<!DOCTYPE html><html><head><meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1">
<title>T-Wookie Android</title>
<style>body{{font-family:sans-serif;background:#0d1117;color:#e6edf3;display:flex;flex-direction:column;align-items:center;padding:2rem;}}
h1{{color:#58a6ff;margin-bottom:.5rem;}}p{{color:#8b949e;margin:.25rem 0;}}
a.btn{{display:inline-block;margin-top:1.5rem;padding:.75rem 2rem;background:#238636;color:#fff;border-radius:8px;text-decoration:none;font-size:1.1rem;}}
img{{margin-top:1.5rem;border-radius:8px;border:2px solid #30363d;}}
</style></head><body>
<h1>T-Wookie</h1>
<p>Android APK &mdash; {apk_size}</p>
<a class=btn href="/download/twookie.apk">&#8595; Download APK</a>
<p style="margin-top:1.5rem;font-size:.85rem;color:#8b949e">Or scan to download on your phone:</p>
<img src="data:image/png;base64,{qr_b64}" width=220 height=220 alt="QR">
</body></html>"""
                body = html.encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if parsed.path == "/download/twookie.apk":
                apk_path = Path(__file__).parent.parent / "twookie-native/android/app/build/outputs/apk/debug/app-debug.apk"
                if not apk_path.exists():
                    self.send_json(404, {"ok": False, "error": "APK not built yet"})
                    return
                data = apk_path.read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "application/vnd.android.package-archive")
                self.send_header("Content-Disposition", 'attachment; filename="twookie.apk"')
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "no-store")
                self.end_headers()
                self.wfile.write(data)
                return
            if parsed.path == "/tbacon":
                body = dashboard().encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if parsed.path == "/api/twookie/accounts":
                self.send_json(200, {"ok": True, "accounts": hub.list_twookie_accounts()})
                return
            if parsed.path == "/api/twookie/contacts":
                owner = parse_qs(parsed.query).get("owner", [""])[0]
                self.send_json(200, {"ok": True, "contacts": hub.list_twookie_contacts(owner or None)})
                return
            if parsed.path == "/api/twookie/image-nodes":
                self.send_json(200, {"ok": True, "nodes": hub.list_image_nodes()})
                return
            if parsed.path.startswith("/api/twookie/invites/") and parsed.path.endswith("/ai-message"):
                token = parsed.path.split("/")[4]
                params = parse_qs(parsed.query)
                sender = params.get("from", ["chatgpt"])[0]
                message_body = params.get("body", ["Test public depuis IA via invitation T-Wookie."])[0]
                result = hub.post_invite_scoped_message(token, sender, message_body)
                self.send_json(
                    200,
                    {
                        "ok": True,
                        "transport": "invite_scoped_ai_write",
                        "account": result["account"],
                        "message": result["message"],
                        "visible_in": f"/?invite={token}",
                    },
                )
                return
            if parsed.path.startswith("/api/twookie/invites/") and parsed.path.endswith("/qr"):
                token = parsed.path.split("/")[4]
                invite = hub.get_twookie_invite(token)
                origin = parse_qs(parsed.query).get("origin", [f"http://{self.headers.get('Host', '127.0.0.1:8894')}"])[0]
                link = f"{origin.rstrip('/')}/?invite={invite['token']}"
                img = qrcode.make(link)
                buf = BytesIO()
                img.save(buf, format="PNG")
                data = buf.getvalue()
                self.send_response(200)
                self.send_header("Content-Type", "image/png")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            if parsed.path.startswith("/api/twookie/invites/"):
                token = parsed.path.split("/")[4]
                self.send_json(200, {"ok": True, "invite": hub.get_twookie_invite(token)})
                return
            if parsed.path == "/api/twookie/bridge/messages":
                params = parse_qs(parsed.query)
                owner = params.get("owner", [""])[0]
                peer = params.get("peer", [""])[0]
                after = int(params.get("after", ["0"])[0] or 0)
                self.send_json(200, {"ok": True, "messages": hub.list_bridge_messages(owner, peer, after)})
                return
            if parsed.path == "/api/twookie/bridge/recent":
                params = parse_qs(parsed.query)
                owner = params.get("owner", [""])[0]
                after = int(params.get("after", ["0"])[0] or 0)
                self.send_json(200, {"ok": True, "messages": hub.list_bridge_recent(owner, after)})
                return
            if parsed.path.startswith("/api/twookie/voice/"):
                voice_id = parsed.path.split("/")[4]
                memo = hub.get_voice_memo(voice_id)
                data = base64.b64decode(memo["audio_base64"])
                self.send_response(200)
                self.send_header("Content-Type", memo["mime_type"])
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            if parsed.path.startswith("/api/twookie/pixel-payloads/") and parsed.path.endswith("/manifest"):
                payload_id = parsed.path.split("/")[4]
                payload = hub.get_pixel_payload(payload_id)
                self.send_json(
                    200,
                    {
                        "ok": True,
                        "payload": {
                            key: payload[key]
                            for key in ("id", "created_at", "from_id", "to_id", "mime_type", "duration_ms", "algorithm", "chunk_count", "payload_hash", "meta_json")
                        },
                        "chunks": [
                            {"index": row["chunk_index"], "chunk_hash": row["chunk_hash"], "url": f"/api/twookie/pixel-payloads/{payload_id}/chunks/{row['chunk_index']}.png"}
                            for row in payload["chunks"]
                        ],
                    },
                )
                return
            if parsed.path.startswith("/api/twookie/pixel-payloads/") and "/chunks/" in parsed.path:
                parts = parsed.path.split("/")
                payload_id = parts[4]
                chunk_index = int(parts[6].split(".")[0])
                payload = hub.get_pixel_payload(payload_id)
                match = next((row for row in payload["chunks"] if int(row["chunk_index"]) == chunk_index), None)
                if not match:
                    self.send_json(404, {"ok": False, "error": "unknown chunk"})
                    return
                if match.get("file_path"):
                    chunk_path = (hub.db_path.parent / match["file_path"]).resolve()
                    payload_root = hub.payloads_dir().resolve()
                    if payload_root not in chunk_path.parents:
                        self.send_json(400, {"ok": False, "error": "invalid chunk path"})
                        return
                    data = chunk_path.read_bytes()
                else:
                    data = base64.b64decode(match["png_base64"])
                self.send_response(200)
                self.send_header("Content-Type", "image/png")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            if parsed.path.startswith("/api/twookie/pixel-uploads/"):
                upload_id = parsed.path.split("/")[4]
                self.send_json(200, {"ok": True, "upload": hub.get_pixel_upload(upload_id)})
                return
            if parsed.path.startswith("/api/twookie/oob/"):
                session_id = parsed.path.split("/")[4]
                self.send_json(200, {"ok": True, "session": hub.get_oob_session(session_id)})
                return
            if parsed.path == "/api/twookie/bridge/outbox":
                if not self.bridge_authorized():
                    self.send_json(403, {"ok": False, "error": "bridge token required"})
                    return
                limit = int(parse_qs(parsed.query).get("limit", ["10"])[0] or 10)
                self.send_json(200, {"ok": True, "messages": hub.claim_bridge_outbox(limit)})
                return
            if parsed.path.startswith("/rooms/"):
                room_id = parsed.path.split("/")[2]
                body = room_page(room_id).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if parsed.path == "/api/users":
                self.send_json(200, {"ok": True, "users": [self.public_user(user) for user in hub.list_users()]})
                return
            if parsed.path.startswith("/api/users/") and parsed.path.endswith("/secret"):
                user_id = parsed.path.split("/")[3]
                secret = self.secret_for_dashboard(user_id)
                if not secret:
                    self.send_json(404, {"ok": False, "error": "unknown user"})
                    return
                self.send_json(200, {"ok": True, "user_id": user_id, "secret": secret})
                return
            if parsed.path == "/api/rooms":
                self.send_json(200, {"ok": True, "rooms": hub.list_rooms()})
                return
            if parsed.path == "/api/events":
                self.send_json(200, {"ok": True, "events": hub.list_events()})
                return
            if parsed.path.startswith("/api/messages/") and parsed.path.endswith("/payload"):
                message_id = int(parsed.path.split("/")[3])
                message = hub.get_message_payload(message_id)
                if message is None:
                    self.send_json(404, {"ok": False, "error": "unknown message"})
                    return
                self.send_json(200, {"ok": True, "id": message_id, "transport": message["transport"], "body": message["body"]})
                return
            if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/messages"):
                room_id = parsed.path.split("/")[3]
                after = int(parse_qs(parsed.query).get("after", ["0"])[0])
                self.send_json(200, {"ok": True, "messages": hub.list_messages(room_id, after)})
                return
            if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/carrier/meta"):
                room_id = parsed.path.split("/")[3]
                carrier = hub.get_room_carrier(room_id)
                if carrier is None:
                    self.send_json(404, {"ok": False, "error": "room carrier not configured"})
                    return
                self.send_json(
                    200,
                    {
                        "ok": True,
                        "room_id": room_id,
                        "mask_id": carrier["mask_id"],
                        "positions": len(carrier["mask"].get("positions", [])),
                        "carrier_frame_url": f"/api/rooms/{room_id}/carrier-frame.png",
                        "updated_at": carrier["updated_at"],
                    },
                )
                return
            if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/carrier"):
                room_id = parsed.path.split("/")[3]
                carrier = hub.get_room_carrier(room_id)
                if carrier is None:
                    self.send_json(404, {"ok": False, "error": "room carrier not configured"})
                    return
                self.send_json(
                    200,
                    {
                        "ok": True,
                        "room_id": room_id,
                        "mask_id": carrier["mask_id"],
                        "mask": carrier["mask"],
                        "carrier_frame_url": f"/api/rooms/{room_id}/carrier-frame.png",
                    },
                )
                return
            if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/carrier-frame.png"):
                room_id = parsed.path.split("/")[3]
                carrier = hub.get_room_carrier(room_id)
                if carrier is None:
                    self.send_json(404, {"ok": False, "error": "room carrier not configured"})
                    return
                data = Path(carrier["carrier_frame_path"]).read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "image/png")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            # ── iOS native API ───────────────────────────────────────────
            if parsed.path.startswith("/api/twookie/accounts/") and parsed.path.endswith("/contacts"):
                account_id = parsed.path.split("/")[4]
                contacts = hub.list_account_contacts(account_id)
                self.send_json(200, {"ok": True, "accounts": contacts})
                return
            if parsed.path.startswith("/api/twookie/accounts/") and len(parsed.path.split("/")) == 5:
                account_id = parsed.path.split("/")[4]
                try:
                    acct = hub.get_twookie_account_by_id(account_id)
                    self.send_json(200, {"ok": True, "account": acct})
                except ValueError:
                    self.send_json(404, {"ok": False, "error": "not found"})
                return
            if parsed.path == "/api/twookie/messages":
                params = parse_qs(parsed.query)
                account_id = params.get("account_id", [""])[0]
                since = int(params.get("since", ["0"])[0] or 0)
                msgs = hub.list_account_messages(account_id, since)
                self.send_json(200, {"ok": True, "messages": msgs})
                return
            if parsed.path == "/api/twookie/pixel-payloads":
                params = parse_qs(parsed.query)
                to_id = params.get("to_id", [""])[0]
                since = int(params.get("since", ["0"])[0] or 0)
                chunks = hub.list_flat_chunks(to_id, since)
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                body = json.dumps(chunks, separators=(",", ":")).encode()
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            # ── WebSocket upgrade ────────────────────────────────────────
            if parsed.path.startswith("/ws/"):
                upgrade = self.headers.get("Upgrade", "").lower()
                if upgrade != "websocket":
                    self.send_json(400, {"ok": False, "error": "websocket upgrade required"})
                    return
                account_id = parsed.path[4:].split("?")[0]
                ws_key = self.headers.get("Sec-WebSocket-Key", "")
                accept = base64.b64encode(
                    hashlib.sha1((ws_key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
                ).decode()
                self.send_response(101, "Switching Protocols")
                self.send_header("Upgrade", "websocket")
                self.send_header("Connection", "Upgrade")
                self.send_header("Sec-WebSocket-Accept", accept)
                self.end_headers()
                self._ws_loop(account_id)
                return
            # ────────────────────────────────────────────────────────────
            prefix = "/tbacon/beacon/"
            if parsed.path.startswith(prefix) and parsed.path.endswith(".png"):
                op = parsed.path[len(prefix) : -4]
                params = {key: value[0] for key, value in parse_qs(parsed.query).items()}
                event = hub.validate_beacon(op, params)
                self.send_png(200 if event["ok"] else 403)
                return
            if parsed.path == "/admin/security":
                body = _render_security_report(hub.security_report()).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Cache-Control", "no-store")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            self.send_json(404, {"ok": False, "error": "not found"})

        def do_PATCH(self) -> None:
            parsed = urlparse(self.path)
            try:
                if parsed.path.startswith("/api/twookie/accounts/") and len(parsed.path.split("/")) == 5:
                    account_id = parsed.path.split("/")[4]
                    body = self.json_body()
                    acct = hub.update_account_display_name(account_id, body.get("display_name") or "")
                    self.send_json(200, {"ok": True, "account": acct})
                    return
                self.send_json(404, {"ok": False, "error": "not found"})
            except Exception as exc:
                self.send_json(400, {"ok": False, "error": str(exc)})

        def _ws_loop(self, account_id: str) -> None:
            sock = self.connection

            def _send(msg: str) -> None:
                data = msg.encode("utf-8")
                length = len(data)
                if length <= 125:
                    hdr = bytes([0x81, length])
                elif length <= 65535:
                    hdr = bytes([0x81, 126]) + struct.pack(">H", length)
                else:
                    hdr = bytes([0x81, 127]) + struct.pack(">Q", length)
                sock.sendall(hdr + data)

            def _close() -> None:
                try:
                    sock.close()
                except Exception:
                    pass

            _ws_mgr.add(account_id, _send, _close)
            try:
                _send(json.dumps({"type": "connected", "account_id": account_id}, separators=(",", ":")))
                while True:
                    hdr = sock.recv(2)
                    if len(hdr) < 2:
                        break
                    opcode = hdr[0] & 0x0F
                    masked = (hdr[1] >> 7) & 1
                    length = hdr[1] & 0x7F
                    if length == 126:
                        length = struct.unpack(">H", sock.recv(2))[0]
                    elif length == 127:
                        length = struct.unpack(">Q", sock.recv(8))[0]
                    mask_key = sock.recv(4) if masked else b""
                    payload = sock.recv(length)
                    if masked:
                        payload = bytes(b ^ mask_key[i % 4] for i, b in enumerate(payload))
                    if opcode == 8:  # close
                        break
                    if opcode == 9:  # ping → pong
                        sock.sendall(b"\x8a\x00")
            except Exception:
                pass
            finally:
                _ws_mgr.remove(account_id, _send)

        def do_POST(self) -> None:
            if self._sec_check("POST"):
                return
            parsed = urlparse(self.path)
            try:
                if parsed.path == "/admin/ban":
                    body = self.json_body()
                    ip = body.get("ip", "").strip()
                    reason = body.get("reason", "manual ban").strip() or "manual ban"
                    if not ip:
                        self.send_json(400, {"ok": False, "error": "ip required"})
                        return
                    hub.ban_ip(ip, reason, auto=False)
                    self.send_json(200, {"ok": True, "banned": ip})
                    return
                if parsed.path == "/admin/unban":
                    body = self.json_body()
                    ip = body.get("ip", "").strip()
                    if not ip:
                        self.send_json(400, {"ok": False, "error": "ip required"})
                        return
                    hub.unban_ip(ip)
                    self.send_json(200, {"ok": True, "unbanned": ip})
                    return
                if parsed.path == "/api/users":
                    body = self.json_body()
                    user = hub.create_user(body["id"], body.get("name") or body["id"])
                    self.send_json(200, {"ok": True, "user": user})
                    return
                if parsed.path == "/api/twookie/accounts":
                    body = self.json_body()
                    result = hub.create_twookie_account(body)
                    self.send_json(200, {"ok": True, **result})
                    return
                if parsed.path == "/api/twookie/touch":
                    body = self.json_body()
                    account = hub.touch_twookie_account(body.get("id") or "")
                    self.send_json(200, {"ok": True, "account": account})
                    return
                if parsed.path == "/api/twookie/contacts":
                    body = self.json_body()
                    contact = hub.add_twookie_contact(body["owner_id"], body["peer_id"])
                    self.send_json(200, {"ok": True, "contact": contact})
                    return
                if parsed.path == "/api/twookie/invites":
                    body = self.json_body()
                    invite = hub.create_twookie_invite(body.get("inviter_id") or "", body.get("label") or "", int(body.get("ttl_seconds") or 86400))
                    self.send_json(200, {"ok": True, "invite": invite})
                    return
                if parsed.path.startswith("/api/twookie/invites/") and parsed.path.endswith("/message"):
                    token = parsed.path.split("/")[4]
                    body = self.json_body()
                    result = hub.post_invite_scoped_message(token, body.get("from") or "chatgpt", body.get("body") or "")
                    self.send_json(200, {"ok": True, "transport": "invite_scoped_ai_write", **result})
                    return
                if parsed.path.startswith("/api/twookie/invites/") and parsed.path.endswith("/accept"):
                    token = parsed.path.split("/")[4]
                    result = hub.accept_twookie_invite(token, self.json_body())
                    self.send_json(200, {"ok": True, **result})
                    return
                if parsed.path == "/api/twookie/oob/start":
                    body = self.json_body()
                    idem_key = body.get("idempotency_key") or self.headers.get("Idempotency-Key") or ""
                    from_id = safe_id(body.get("from") or body.get("from_id") or "")
                    to_id = safe_id(body.get("to") or body.get("to_id") or "")
                    result = hub.with_idempotency(
                        f"twookie-oob-start:{from_id}:{to_id}",
                        idem_key,
                        body,
                        lambda: {"ok": True, "session": hub.start_oob_session(body)},
                    )
                    self.send_json(200, result)
                    return
                if parsed.path == "/api/twookie/oob/confirm":
                    body = self.json_body()
                    self.send_json(200, {"ok": True, "session": hub.confirm_oob_session(body)})
                    return
                if parsed.path == "/api/twookie/pixel-uploads":
                    body = self.json_body()
                    idem_key = body.get("idempotency_key") or self.headers.get("Idempotency-Key") or ""
                    from_id = safe_id(body.get("from") or body.get("from_id") or "")
                    to_id = safe_id(body.get("to") or body.get("to_id") or "")
                    result = hub.with_idempotency(
                        f"twookie-pixel-upload-start:{from_id}:{to_id}",
                        idem_key,
                        body,
                        lambda: {"ok": True, "upload": hub.create_pixel_upload(body)},
                    )
                    self.send_json(200, result)
                    return
                if parsed.path.startswith("/api/twookie/pixel-uploads/") and parsed.path.endswith("/chunks"):
                    upload_id = parsed.path.split("/")[4]
                    body = self.json_body()
                    idem_key = body.get("idempotency_key") or self.headers.get("Idempotency-Key") or ""
                    result = hub.with_idempotency(
                        f"twookie-pixel-upload-chunk:{upload_id}:{body.get('index')}",
                        idem_key,
                        body,
                        lambda: {"ok": True, "upload": hub.add_pixel_upload_chunk(upload_id, body)},
                    )
                    self.send_json(200, result)
                    return
                if parsed.path.startswith("/api/twookie/pixel-uploads/") and parsed.path.endswith("/complete"):
                    upload_id = parsed.path.split("/")[4]
                    body = self.json_body()
                    idem_key = body.get("idempotency_key") or self.headers.get("Idempotency-Key") or ""
                    result = hub.with_idempotency(
                        f"twookie-pixel-upload-complete:{upload_id}",
                        idem_key,
                        body,
                        lambda: {"ok": True, **hub.complete_pixel_upload(upload_id, body)},
                    )
                    self.send_json(200, result)
                    return
                if parsed.path == "/api/twookie/messages":
                    body = self.json_body()
                    validated = hub.validate_twookie_message_request(body)

                    def create_message() -> dict:
                        message_kind = validated["kind"]
                        message_body = validated["body"]
                        message_meta = {}
                        if message_kind == "voice":
                            duration_ms = int(body.get("duration_ms") or 0)
                            if duration_ms > 10000:
                                raise ValueError("voice memo max is 10 seconds")
                            voice = hub.add_voice_memo(
                                validated["from"],
                                validated["to"],
                                body.get("audio_base64") or "",
                                body.get("mime_type") or "audio/webm",
                                duration_ms,
                            )
                            message_body = "[voice memo]"
                            message_meta = {"kind": "voice_memo", "voice_id": voice["id"], "mime_type": voice["mime_type"], "duration_ms": voice["duration_ms"]}
                        if message_kind == "pixel_voice":
                            duration_ms = int(body.get("duration_ms") or 0)
                            if duration_ms > 10000:
                                raise ValueError("voice memo max is 10 seconds")
                            pixel = hub.add_pixel_payload(
                                validated["from"],
                                validated["to"],
                                body.get("mime_type") or "audio/webm",
                                duration_ms,
                                body.get("chunks") or [],
                                body.get("payload_hash") or "",
                                {"cipher": body.get("cipher") or "aes-256-gcm", "nonce_b64": body.get("nonce_b64") or "", "width": body.get("width") or 96, "height": body.get("height") or 96},
                            )
                            message_body = "[pixel voice memo]"
                            message_meta = {"kind": "pixel_voice_memo", "pixel_id": pixel["id"], "mime_type": pixel["mime_type"], "duration_ms": pixel["duration_ms"], "chunk_count": pixel["chunk_count"], "payload_hash": pixel["payload_hash"], "algorithm": pixel["algorithm"], "cipher": body.get("cipher") or "aes-256-gcm", "nonce_b64": body.get("nonce_b64") or ""}
                        payload_hash = sha256_hex_text({key: value for key, value in idempotency_request_shape(body).items() if key not in {"idempotency_key"}})
                        node = hub.add_image_node(payload_hash, None, {"kind": "message_intent", "from": validated["from"], "to": validated["to"]})
                        bridge_direction = "web_to_discord" if validated["to"].startswith("discord-") else "web_local"
                        bridge_source = "twookie_web" if bridge_direction == "web_to_discord" else "local_web"
                        bridge = hub.add_bridge_message(
                            bridge_direction,
                            validated["from"],
                            validated["to"],
                            message_body,
                            bridge_source,
                            {"node_id": node["id"], **message_meta},
                        )
                        if bridge_direction == "web_local":
                            bridge = hub.mark_bridge_delivered(bridge["id"], "delivered")
                        auto_reply = None
                        sender_id = validated["from"]
                        target_id = validated["to"]
                        if message_kind == "text" and target_id == "codex-local" and sender_id and sender_id != "codex-local":
                            reply_body = codex_local_reply(validated["body"])
                            reply_hash = sha256_hex_text({"from": "codex-local", "to": sender_id, "body": reply_body})
                            reply_node = hub.add_image_node(
                                reply_hash,
                                None,
                                {"kind": "codex_auto_reply", "from": "codex-local", "to": sender_id, "reply_to": bridge["id"]},
                            )
                            auto_reply = hub.add_bridge_message(
                                "web_local",
                                "codex-local",
                                sender_id,
                                reply_body,
                                "codex_local",
                                {"node_id": reply_node["id"], "reply_to": bridge["id"]},
                            )
                            auto_reply = hub.mark_bridge_delivered(auto_reply["id"], "delivered")
                        return {
                            "ok": True,
                            "intent": {
                                "from": validated["from"],
                                "to": validated["to"],
                                "node": node,
                                "bridge": bridge,
                                "auto_reply": auto_reply,
                            },
                        }

                    idem_key = body.get("idempotency_key") or self.headers.get("Idempotency-Key") or ""
                    result = hub.with_idempotency(f"twookie-message:{validated['from']}:{validated['to']}:{validated['kind']}", idem_key, body, create_message)
                    self.send_json(200, result)
                    # Push APNs and WebSocket notification to recipient
                    to_id = validated["to"]
                    from_id = validated["from"]
                    if not to_id.startswith("discord-"):
                        _ws_mgr.broadcast(to_id, {"type": "message", "from_id": from_id})
                        threading.Thread(
                            target=hub.push_apns,
                            args=(to_id,),
                            daemon=True,
                        ).start()
                    return
                if parsed.path == "/api/twookie/bridge/inbound":
                    if not self.bridge_authorized():
                        self.send_json(403, {"ok": False, "error": "bridge token required"})
                        return
                    body = self.json_body()
                    discord_id = clean_text(body.get("discord_user_id") or "", 64)
                    account = hub.ensure_discord_account(discord_id, body.get("display_name") or "")
                    target_id = safe_id(body.get("to_id") or os.getenv("TBACON_WEB_DEFAULT_USER", "nate"))
                    message = hub.add_bridge_message(
                        "discord_to_web",
                        account["id"],
                        target_id,
                        body.get("body") or "",
                        "discord_bot",
                        {"discord_message_id": body.get("discord_message_id"), "username": body.get("display_name")},
                    )
                    self.send_json(200, {"ok": True, "account": account, "message": message})
                    return
                if parsed.path == "/api/twookie/bridge/delivered":
                    if not self.bridge_authorized():
                        self.send_json(403, {"ok": False, "error": "bridge token required"})
                        return
                    body = self.json_body()
                    message = hub.mark_bridge_delivered(int(body.get("id") or 0), body.get("status") or "delivered")
                    self.send_json(200, {"ok": True, "message": message})
                    return
                if parsed.path == "/api/twookie/bridge/purge":
                    body = self.json_body()
                    result = hub.purge_bridge_conversation(body.get("owner") or "", body.get("peer") or "")
                    self.send_json(200, {"ok": True, **result})
                    return
                if parsed.path in {"/api/v1/server/purge", "/api/twookie/server/purge"}:
                    body = self.json_body()
                    dry_run = bool(body.get("dry_run", True))
                    result = hub.purge_server(dry_run=dry_run)
                    self.send_json(200, {"ok": True, **result})
                    return
                if parsed.path in {"/api/v1/server/migrate-pixel-chunks", "/api/twookie/server/migrate-pixel-chunks"}:
                    body = self.json_body()
                    dry_run = bool(body.get("dry_run", True))
                    limit = int(body.get("limit") or 500)
                    result = hub.migrate_legacy_pixel_chunks(dry_run=dry_run, limit=limit)
                    self.send_json(200, {"ok": True, **result})
                    return
                if parsed.path in {"/api/v1/server/vacuum", "/api/twookie/server/vacuum"}:
                    result = hub.vacuum_database()
                    self.send_json(200, {"ok": True, **result})
                    return
                if parsed.path == "/api/twookie/calls":
                    body = self.json_body()
                    call = hub.add_twookie_call(body.get("from") or "", body.get("to") or "", body.get("mode") or "signal")
                    self.send_json(200, {"ok": True, "call": call})
                    return
                if parsed.path == "/api/twookie/image-nodes":
                    body = self.json_body()
                    node = hub.add_image_node(body["payload_hash"], body.get("image_path"), body.get("meta") or {})
                    self.send_json(200, {"ok": True, "node": node})
                    return
                # ── iOS native API ──────────────────────────────────────
                if parsed.path.startswith("/api/twookie/accounts/") and parsed.path.endswith("/token"):
                    account_id = parsed.path.split("/")[4]
                    body = self.json_body()
                    token = hub.claim_auth_token(account_id, body.get("seed_hash") or "")
                    self.send_json(200, {"ok": True, "auth_token": token})
                    return
                if parsed.path.startswith("/api/twookie/accounts/") and parsed.path.endswith("/contacts"):
                    account_id = parsed.path.split("/")[4]
                    body = self.json_body()
                    hub.add_account_contact(account_id, body.get("contact_id") or "")
                    self.send_json(200, {"ok": True})
                    return
                if parsed.path == "/api/twookie/messages/seen":
                    body = self.json_body()
                    hub.mark_seen(
                        body.get("account_id") or "",
                        body.get("peer_id") or "",
                    )
                    self.send_json(200, {"ok": True})
                    return
                if parsed.path == "/api/twookie/apns/register":
                    body = self.json_body()
                    hub.register_device_token(
                        body.get("account_id") or "",
                        body.get("device_token") or "",
                        body.get("bundle_id") or "com.trustos.twookie",
                    )
                    self.send_json(200, {"ok": True})
                    return
                if parsed.path == "/api/twookie/pixel-payloads":
                    body = self.json_body()
                    hub.add_flat_chunk(body)
                    self.send_json(200, {"ok": True})
                    return
                # ───────────────────────────────────────────────────────
                if parsed.path.startswith("/api/users/") and parsed.path.endswith("/rotate"):
                    user_id = parsed.path.split("/")[3]
                    user = hub.rotate_user(user_id)
                    self.send_json(200, {"ok": True, "user": user})
                    return
                if parsed.path == "/api/rooms":
                    body = self.json_body()
                    room = hub.create_room(body["id"], body.get("name") or body["id"])
                    self.send_json(200, {"ok": True, "room": room})
                    return
                if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/members"):
                    body = self.json_body()
                    room_id = parsed.path.split("/")[3]
                    hub.add_member(room_id, body["user_id"])
                    self.send_json(200, {"ok": True})
                    return
                if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/messages"):
                    body = self.json_body()
                    room_id = parsed.path.split("/")[3]
                    message = hub.add_message(room_id, body["user_id"], body["body"])
                    self.send_json(200, {"ok": True, "message": message})
                    return
                if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/pixel-messages"):
                    body = self.json_body()
                    room_id = parsed.path.split("/")[3]
                    message = hub.add_pixel_message(room_id, body["user_id"], body["encodedPng"])
                    self.send_json(200, {"ok": True, "message": message})
                    return
                if parsed.path.startswith("/api/rooms/") and parsed.path.endswith("/carrier"):
                    body = self.json_body()
                    room_id = parsed.path.split("/")[3]
                    carrier = hub.set_room_carrier(room_id, body["mask"], body["carrier_frame_path"])
                    self.send_json(200, {"ok": True, "carrier": carrier})
                    return
                self.send_json(404, {"ok": False, "error": "not found"})
            except Exception as exc:
                self.send_json(400, {"ok": False, "error": str(exc)})

        def public_user(self, user: dict) -> dict:
            return dict(user)

        def secret_for_dashboard(self, user_id: str) -> str:
            with hub.connect() as conn:
                row = conn.execute("SELECT secret FROM users WHERE id = ?", (user_id,)).fetchone()
                return row["secret"] if row else ""

        # ------------------------------------------------------------------
        # Security helpers
        # ------------------------------------------------------------------

        def _real_ip(self) -> str:
            xff = self.headers.get("X-Forwarded-For", "")
            if xff:
                return xff.split(",")[0].strip()
            return self.client_address[0]

        def _sec_check(self, method: str) -> bool:
            """Log request, check ban, detect recon. Returns True if banned (send 403)."""
            ip = self._real_ip()
            path = urlparse(self.path).path
            ua = self.headers.get("User-Agent", "")
            xff = self.headers.get("X-Forwarded-For", "")
            flag = "recon" if _RECON_RE.search(path) else None

            # Store on handler so _sec_done can log final status
            self._sec_ip = ip
            self._sec_method = method
            self._sec_path = path
            self._sec_ua = ua
            self._sec_xff = xff
            self._sec_flag = flag

            ban_reason = hub.is_banned(ip)
            if ban_reason:
                hub.log_request(ip, method, path, 403, ua, xff, "banned")
                self.send_json(403, {"ok": False, "error": "forbidden"})
                return True
            return False

        def _sec_done(self, status: int) -> None:
            """Log completed request and trigger auto-ban check."""
            ip = getattr(self, "_sec_ip", self.client_address[0])
            method = getattr(self, "_sec_method", "?")
            path = getattr(self, "_sec_path", self.path)
            ua = getattr(self, "_sec_ua", "")
            xff = getattr(self, "_sec_xff", "")
            flag = getattr(self, "_sec_flag", None)
            if status == 404 and not flag:
                flag = "404"
            hub.log_request(ip, method, path, status, ua, xff, flag)
            if not hub.is_banned(ip):
                reason = hub.check_auto_ban(ip)
                if reason:
                    hub.ban_ip(ip, reason, auto=True)

        def send_response(self, code, message=None):
            self._last_status = code
            super().send_response(code, message)

        def log_message(self, fmt: str, *args) -> None:
            status = getattr(self, "_last_status", 200)
            self._sec_done(status)

    return Handler


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8894)
    parser.add_argument("--db", type=Path, default=Path("reports/tbacon/latest/tbacon.sqlite3"))
    args = parser.parse_args()
    hub = Hub(args.db)
    server = ThreadingHTTPServer((args.host, args.port), make_handler(hub))
    print(f"T-Bacon Hub: http://{args.host}:{args.port}/")
    print(f"DB: {args.db}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
