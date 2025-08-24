@echo off
echo Testing gstreamer-mcp MCP server...
echo.

if not exist target\release\gstreamer-mcp.exe (
    echo Building gstreamer-mcp...
    cargo build --release
)

echo.
echo Running MCP server tests...
(
    echo {"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test-client","version":"1.0.0"}}}
    echo {"jsonrpc":"2.0","method":"notifications/initialized"}
    timeout /t 1 >nul
    echo {"jsonrpc":"2.0","id":"tools","method":"tools/list","params":{}}
    timeout /t 1 >nul
    echo {"jsonrpc":"2.0","id":"list_elements","method":"tools/call","params":{"name":"gst_list_elements","arguments":{}}}
    timeout /t 2 >nul
    echo {"jsonrpc":"2.0","id":"list_plugins","method":"tools/call","params":{"name":"gst_list_plugins","arguments":{}}}
    timeout /t 2 >nul
    echo {"jsonrpc":"2.0","id":"inspect","method":"tools/call","params":{"name":"gst_inspect_element","arguments":{"element_name":"videotestsrc"}}}
    timeout /t 1 >nul
    echo {"jsonrpc":"2.0","id":"search","method":"tools/call","params":{"name":"gst_search_elements","arguments":{"query":"video"}}}
    timeout /t 1 >nul
) | target\release\gstreamer-mcp.exe 2>server.log

echo.
echo Server output saved to server.log
echo Tests completed! Check server.log for detailed output.
echo.
type server.log
echo.