# WebProxy
## A Standard Web Forward Proxy
# Extension
Connecting to the proxy is done by using the provided Chrome Extensions.
## Google Chrome Extension
Load the extension in your chrome browser.
After that you can click the icon and configure the proxy.

**Note that you can connect to any proxy that uses the HTTP scheme.**
# Server
Inside ProxyServer/

How to run: 
```bash
cargo run
```

The main function simply wraps the proxy server library.
```rust
mod proxy;
use proxy::ProxyServer;

fn main() {
    let mut proxyserver = ProxyServer::new("0.0.0.0:12345");
    proxyserver.allow("127.0.0.1"); // Default all IP's are allowed.
    proxyserver.run();
}
```


