#!/usr/bin/env python3
"""Serve Swagger UI for Hyperliquid API OpenAPI documentation

Usage:
    uv run python scripts/serve_swagger.py
    # Or
    uv run python scripts/serve_swagger.py --port 8080
"""

import argparse
import json
import os
from pathlib import Path
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs


class SwaggerUIHandler(BaseHTTPRequestHandler):
    """HTTP handler for serving Swagger UI"""
    
    def __init__(self, *args, openapi_spec_path=None, **kwargs):
        self.openapi_spec_path = openapi_spec_path
        super().__init__(*args, **kwargs)
    
    def do_GET(self):
        """Handle GET requests"""
        parsed_path = urlparse(self.path)
        path = parsed_path.path
        
        # Serve Swagger UI HTML
        if path == "/" or path == "/docs":
            self.send_response(200)
            self.send_header("Content-type", "text/html")
            self.end_headers()
            
            # Read OpenAPI spec
            spec_path = Path(self.openapi_spec_path)
            if spec_path.exists():
                with open(spec_path, 'r') as f:
                    openapi_spec = json.load(f)
            else:
                openapi_spec = {}
            
            # Generate Swagger UI HTML
            html = self._generate_swagger_html(openapi_spec)
            self.wfile.write(html.encode('utf-8'))
        
        # Serve OpenAPI spec JSON
        elif path == "/openapi.json":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            
            spec_path = Path(self.openapi_spec_path)
            if spec_path.exists():
                with open(spec_path, 'rb') as f:
                    self.wfile.write(f.read())
            else:
                self.wfile.write(b'{"error": "OpenAPI spec not found"}')
        
        # Serve static assets (Swagger UI CSS/JS)
        elif path.startswith("/swagger-ui/"):
            self._serve_swagger_assets(path)
        
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b'Not Found')
    
    def _generate_swagger_html(self, openapi_spec: dict) -> str:
        """Generate Swagger UI HTML page"""
        spec_json = json.dumps(openapi_spec, indent=2)
        
        return f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hyperliquid API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui.css" />
    <style>
        html {{
            box-sizing: border-box;
            overflow: -moz-scrollbars-vertical;
            overflow-y: scroll;
        }}
        *, *:before, *:after {{
            box-sizing: inherit;
        }}
        body {{
            margin:0;
            background: #fafafa;
        }}
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {{
            const spec = {spec_json};
            
            const ui = SwaggerUIBundle({{
                spec: spec,
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                tryItOutEnabled: true,
                supportedSubmitMethods: ['get', 'post', 'put', 'delete', 'patch'],
                validatorUrl: null
            }});
        }};
    </script>
</body>
</html>"""
    
    def _serve_swagger_assets(self, path: str):
        """Serve Swagger UI static assets (fallback to CDN)"""
        # For simplicity, we redirect to CDN
        # In production, you might want to serve these locally
        self.send_response(302)
        self.send_header("Location", f"https://unpkg.com/swagger-ui-dist@5.10.5{path}")
        self.end_headers()
    
    def log_message(self, format, *args):
        """Override to customize logging"""
        print(f"[{self.address_string()}] {format % args}")


def create_handler_class(openapi_spec_path):
    """Create a handler class with the spec path"""
    class Handler(SwaggerUIHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, openapi_spec_path=openapi_spec_path, **kwargs)
    return Handler


def main():
    parser = argparse.ArgumentParser(description="Serve Swagger UI for Hyperliquid API")
    parser.add_argument(
        "--port",
        type=int,
        default=8080,
        help="Port to serve on (default: 8080)"
    )
    parser.add_argument(
        "--host",
        type=str,
        default="localhost",
        help="Host to bind to (default: localhost)"
    )
    parser.add_argument(
        "--spec",
        type=str,
        default=None,
        help="Path to OpenAPI spec JSON file (default: openapi/openapi.json)"
    )
    
    args = parser.parse_args()
    
    # Determine spec path
    script_dir = Path(__file__).parent.parent
    if args.spec:
        spec_path = Path(args.spec).resolve()
    else:
        spec_path = script_dir / "openapi" / "openapi.json"
    
    if not spec_path.exists():
        print(f"❌ Error: OpenAPI spec not found at {spec_path}")
        print(f"   Please ensure the spec file exists or use --spec to specify a different path")
        return 1
    
    print(f"📄 Loading OpenAPI spec from: {spec_path}")
    
    # Create handler class with spec path
    Handler = create_handler_class(str(spec_path))
    
    # Start server
    server_address = (args.host, args.port)
    httpd = HTTPServer(server_address, Handler)
    
    print(f"\n{'='*70}")
    print(f"  Swagger UI Server")
    print(f"{'='*70}")
    print(f"📍 URL: http://{args.host}:{args.port}/docs")
    print(f"📄 OpenAPI Spec: http://{args.host}:{args.port}/openapi.json")
    print(f"📁 Spec File: {spec_path}")
    print(f"{'='*70}")
    print(f"\n🚀 Server running... Press Ctrl+C to stop\n")
    
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n\n👋 Shutting down server...")
        httpd.shutdown()
        return 0


if __name__ == "__main__":
    exit(main())

