#!/usr/bin/env python3
"""
Simple HTTP Proxy for TrustOS Browser
Receives HTTP requests from TrustOS and forwards them as HTTPS to real servers.
TrustOS connects to 10.0.2.2:8080 (QEMU host) and this proxy fetches the page.

Usage: python web_proxy.py
Then in TrustOS browser: http://10.0.2.2:8080/https://google.com
"""

import http.server
import urllib.request
import urllib.error
import ssl
import sys

PORT = 8080

class ProxyHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        # Path format: /https://google.com or /http://example.com
        path = self.path[1:]  # Remove leading /
        
        if not path:
            self.send_response(200)
            self.send_header('Content-Type', 'text/html')
            self.end_headers()
            self.wfile.write(b'''<!DOCTYPE html>
<html>
<head><title>TrustOS Web Proxy</title></head>
<body style="background:#1a1a2e;color:#0f0;font-family:monospace;padding:20px;">
<h1>TrustOS Web Proxy</h1>
<p>Usage: http://10.0.2.2:8080/https://google.com</p>
<p>Or: http://10.0.2.2:8080/https://example.com</p>
</body>
</html>''')
            return
        
        # Ensure URL has scheme
        if not path.startswith('http://') and not path.startswith('https://'):
            path = 'https://' + path
        
        print(f"[PROXY] Fetching: {path}")
        
        try:
            # Create SSL context that doesn't verify (for simplicity)
            ctx = ssl.create_default_context()
            ctx.check_hostname = False
            ctx.verify_mode = ssl.CERT_NONE
            
            req = urllib.request.Request(
                path,
                headers={
                    'User-Agent': 'Mozilla/5.0 (TrustOS; rv:1.0) Gecko/20100101',
                    'Accept': 'text/html,application/xhtml+xml,*/*',
                    'Accept-Language': 'en-US,en;q=0.5',
                }
            )
            
            with urllib.request.urlopen(req, timeout=10, context=ctx) as response:
                content = response.read()
                content_type = response.headers.get('Content-Type', 'text/html')
                
                self.send_response(200)
                self.send_header('Content-Type', content_type)
                self.send_header('Content-Length', len(content))
                self.send_header('Access-Control-Allow-Origin', '*')
                self.end_headers()
                self.wfile.write(content)
                
                print(f"[PROXY] OK: {len(content)} bytes")
                
        except urllib.error.HTTPError as e:
            print(f"[PROXY] HTTP Error: {e.code}")
            self.send_response(e.code)
            self.end_headers()
            self.wfile.write(f"HTTP Error: {e.code}".encode())
            
        except urllib.error.URLError as e:
            print(f"[PROXY] URL Error: {e.reason}")
            self.send_response(502)
            self.end_headers()
            self.wfile.write(f"URL Error: {e.reason}".encode())
            
        except Exception as e:
            print(f"[PROXY] Error: {e}")
            self.send_response(500)
            self.end_headers()
            self.wfile.write(f"Error: {e}".encode())
    
    def log_message(self, format, *args):
        print(f"[PROXY] {args[0]}")

def main():
    print(f"=" * 50)
    print(f"TrustOS Web Proxy Server")
    print(f"=" * 50)
    print(f"Listening on port {PORT}")
    print(f"")
    print(f"In TrustOS browser, use:")
    print(f"  http://10.0.2.2:{PORT}/https://google.com")
    print(f"  http://10.0.2.2:{PORT}/https://example.com")
    print(f"")
    print(f"Press Ctrl+C to stop")
    print(f"=" * 50)
    
    with http.server.HTTPServer(('0.0.0.0', PORT), ProxyHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n[PROXY] Shutting down...")

if __name__ == '__main__':
    main()
