mod proxy;

use proxy::ProxyServer;

fn main() {
    let mut proxyserver = ProxyServer::new("0.0.0.0:12345");
    proxyserver.run();
}
