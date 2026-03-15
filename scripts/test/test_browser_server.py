"""Simple HTTP test server for TrustOS browser testing.

Serves test_server/ directory on port 8080.
Sets cookies on responses to test cookie handling.
Has a /submit endpoint to test form POST.

From QEMU user-mode networking, the host is accessible at 10.0.2.2.
So in TrustOS: browse http://10.0.2.2:8080/
"""

import http.server
import os
import sys
from urllib.parse import parse_qs

PORT = 8080
DIRECTORY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "test_server")


class TrustOSTestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def do_GET(self):
        # Serve files with Set-Cookie header to test cookie handling
        super().do_GET()

    def end_headers(self):
        # Add Set-Cookie headers to every response
        self.send_header("Set-Cookie", "session_id=trustos_test_123; Path=/")
        self.send_header("Set-Cookie", "theme=dark; Path=/")
        self.send_header("X-Powered-By", "TrustOS-TestServer/1.0")
        super().end_headers()

    def do_POST(self):
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length).decode("utf-8", errors="replace")

        # Parse form data
        params = parse_qs(body)

        # Build response
        response = f"""<!DOCTYPE html>
<html>
<head><title>Form Submitted</title></head>
<body style="background:#1a1a2e; color:#eaeaea;">
<h1 style="color:#00ff88;">Form Submission Received!</h1>
<h2>POST Data:</h2>
<pre>{body}</pre>
<h2>Parsed Fields:</h2>
<ul>
"""
        for key, values in params.items():
            for val in values:
                response += f"<li><b>{key}</b> = {val}</li>\n"

        response += """</ul>
<p><a href="/">Back to test page</a></p>
<h2>Cookie Test</h2>
<p>Cookies received from browser:</p>
<pre>"""
        cookie_header = self.headers.get("Cookie", "(none)")
        response += f"{cookie_header}"
        response += """</pre>
</body>
</html>"""

        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(response.encode())))
        self.send_header("Set-Cookie", "form_submitted=true; Path=/")
        self.end_headers()
        self.wfile.write(response.encode())

    def log_message(self, format, *args):
        # Color-coded logging
        msg = format % args
        if "POST" in msg:
            print(f"\033[33m[POST] {msg}\033[0m")
        elif "200" in msg:
            print(f"\033[32m[OK]   {msg}\033[0m")
        elif "404" in msg:
            print(f"\033[31m[404]  {msg}\033[0m")
        else:
            print(f"[LOG]  {msg}")


if __name__ == "__main__":
    os.chdir(DIRECTORY)
    print(f"=" * 60)
    print(f"TrustOS Browser Test Server")
    print(f"=" * 60)
    print(f"Serving: {DIRECTORY}")
    print(f"Port:    {PORT}")
    print(f"")
    print(f"From host:   http://localhost:{PORT}/")
    print(f"From QEMU:   http://10.0.2.2:{PORT}/")
    print(f"")
    print(f"In TrustOS shell, type:")
    print(f"  browse http://10.0.2.2:{PORT}/")
    print(f"")
    print(f"Or in desktop browser, navigate to:")
    print(f"  http://10.0.2.2:{PORT}/")
    print(f"=" * 60)

    with http.server.HTTPServer(("0.0.0.0", PORT), TrustOSTestHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nServer stopped.")
