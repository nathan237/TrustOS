#!/usr/bin/env python3
"""Simple HTTP server for TrustOS package distribution."""

import http.server
import socketserver
import os

PORT = 8080
DIRECTORY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "packages")

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)
    
    def log_message(self, format, *args):
        print(f"[{self.address_string()}] {format % args}")

if __name__ == "__main__":
    os.chdir(DIRECTORY)
    print(f"TrustOS Package Server")
    print(f"======================")
    print(f"Serving: {DIRECTORY}")
    print(f"URL: http://0.0.0.0:{PORT}")
    print(f"VM access: http://10.0.2.2:{PORT}")
    print(f"Press Ctrl+C to stop\n")
    
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nServer stopped.")
