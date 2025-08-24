@echo off
echo Testing basic GStreamer MCP functionality...
echo.

(
    echo {"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}
    echo {"jsonrpc":"2.0","method":"notifications/initialized"}
    echo {"jsonrpc":"2.0","id":"search","method":"tools/call","params":{"name":"gst_search_elements","arguments":{"query":"test"}}}
) | target\release\gstreamer-mcp.exe 2>nul | findstr /C:"videotestsrc" /C:"audiotestsrc" /C:"result"

echo.
echo If you see test sources above, the server is working!