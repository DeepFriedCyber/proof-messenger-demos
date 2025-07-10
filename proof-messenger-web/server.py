#!/usr/bin/env python3
"""
Simple HTTP server with proper MIME types for WASM files
"""
import http.server
import socketserver
import mimetypes
import os

# Add WASM MIME type
mimetypes.add_type('application/wasm', '.wasm')

class WASMHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add CORS headers
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        # Add security headers for WASM
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

if __name__ == "__main__":
    PORT = 8000
    
    # Change to the directory containing the web files
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    with socketserver.TCPServer(("", PORT), WASMHTTPRequestHandler) as httpd:
        print(f"Serving at http://localhost:{PORT}/")
        print("WASM MIME type configured correctly")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nServer stopped.")