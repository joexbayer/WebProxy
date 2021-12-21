extern crate threadpool;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, IpAddr};
use std::str::FromStr;
use std::time::Instant;
use std::{time};
use threadpool::ThreadPool;
use std::sync::{Mutex, Arc};

/// Proxy server struct containing list of clients, socket and threadpool
pub struct ProxyServer  {
    clients_list: Vec<Arc<ProxyConnection>>,
    whitelist: Vec<IpAddr>,
    server_socket: TcpListener,
    pool: ThreadPool
}

impl ProxyServer {
    /// Returns a proxy server with instantiated struct memebers
    ///
    /// # Arguments
    ///
    /// * `addr` - A string representing IP and port to listen too.
    ///
    /// # Examples
    ///
    /// ```
    /// use proxy::ProxyServer;
    /// let mut proxyserver = ProxyServer::new("0.0.0.0:12345");
    /// ```
    pub fn new(addr: &str) -> Self {
        Self {
            server_socket: TcpListener::bind(addr).unwrap(),
            whitelist: Vec::new(),
            pool: ThreadPool::new(20),
            clients_list: Vec::<Arc<ProxyConnection>>::new()
        }
    }

    /// Allow certain IPs to connect
    ///
    /// # Arguments
    ///
    /// * `self` - The proxy server itself
    /// * `str` - &str of IP to allow 
    ///
    /// # Examples
    ///
    /// ```
    /// proxyserver.allow("127.0.0.1");
    /// ```
    pub fn allow(&mut self, str: &str){
        let ip = std::net::IpAddr::from_str(str).unwrap();
        self.whitelist.push(ip);
    }

    /// Registers a incomming connection if its not already registered.
    ///
    /// # Arguments
    ///
    /// * `self` - The proxy server itself
    /// * `addr` - The address of the incomming connection.
    ///
    /// # Examples
    ///
    /// ```
    /// self.register_connection(addr);
    /// ```
    fn register_connection(&mut self, addr: SocketAddr) -> bool{
        for i in &self.clients_list {
            if i.ipaddr == addr.ip() {
                return true;
            }
        }

        let conn = Arc::new(ProxyConnection::new(addr.ip()));
        let conn2 = conn.clone();

        /* Start new thread from pool for client. */
        self.pool.execute( move || {
            let _res = conn2.run().clone();
        });

        println!("Connection request from {}", addr.ip());
        self.clients_list.push(conn);
        
        return false;
    }
    /// Main loop function for proxy server
    ///
    /// # Arguments
    ///
    /// * `self` - The proxy server itself
    ///
    /// # Examples
    ///
    /// ```
    /// let mut proxyserver = ProxyServer::new("0.0.0.0:12345");
    /// proxyserver.run();
    /// ```
    pub fn run(&mut self) {
        loop {
            match self.server_socket.accept() {
                Ok((stream, addr)) => {

                    /* Check if account is in whitelist */
                    if self.whitelist.len() != 0 && !self.whitelist.contains(&addr.ip()) {
                        continue;
                    }

                    self.register_connection(addr);
                    for i in &self.clients_list {
                        if i.ipaddr == addr.ip() {
                            i.connect(stream);
                            break;
                        }
                    }
                },
                Err(e) => {println!("[error thread] {}", e)}
            }
        }
    }
}
/// Struct for a single connection defined by a IpAddr with tunnels
struct ProxyConnection {
    tunnels: Mutex<Vec<ProxyConnectionTunnel>>,
    ipaddr: IpAddr
}

impl ProxyConnection {
    /// Returns a proxy connection with instantiated struct memebers
    /// Usually only called inside of a proxy server.
    /// 
    /// # Arguments
    ///
    /// * `addr` - A string representing IP from connection.
    ///
    /// # Examples
    ///
    /// ```
    /// let conn = ProxyConnection::new(addr.ip();
    /// ```
    fn new(addr: IpAddr) -> Self {
        Self {
            tunnels: Mutex::new(Vec::new()),
            ipaddr: addr
        }
    }

    /// Connect function for proxy connection
    /// Connects to parsed domain and port.
    ///
    /// # Arguments
    ///
    /// * `self` - The proxy server itself
    /// * `stream` - Stream to read from.
    ///
    /// # Examples
    ///
    /// ```
    /// connection.connect(stream);
    /// ```
    fn connect(&self, mut stream: TcpStream){
        /* Parse connection HTTP request */
        let mut buffer = [0 as u8; 2000];
        let connect: &str = match stream.read(&mut buffer){
           Ok(usize) => {
               if usize == 0 {
                   println!("[error read] Client sent empty connection request.");
                   return;
               }

               /* Parse CONNECT <domain:port> ... */
               let string = std::str::from_utf8(&mut buffer[0..usize]).unwrap();
               let string_split: Vec<&str> = string.split(" ").collect();

               /* Send OK response */
               let response = "HTTP/1.1 200 OK\r\n\r\n";
               stream.write(response.as_bytes()).unwrap();
               stream.flush().unwrap();

               /* Return response */
               string_split[1]
           },
           Err(e) => {println!("[error parse] {}", e); ""}
        };

       /* Connect to the given domain */
        let stream2: TcpStream = match TcpStream::connect(connect) {
            Ok(stream) => {
                stream
            },
            Err(e) => {println!("[error connect] {}", e);return;}
        };

        println!("Connection request from {} too {}", self.ipaddr, connect);

        self.add_tunnel(stream, stream2);
    }

    /// Adds a tunnel to proxy connection
    ///
    /// # Arguments
    ///
    /// * `self` - The proxy server itself
    /// * `s1` - First stream for tunnel
    /// * `s1` - Second stream for tunnel
    /// # Examples
    ///
    /// ```
    /// self.add_tunnel(stream, stream2);
    /// ```
    fn add_tunnel(&self, s1: TcpStream, s2: TcpStream) {
        s1.set_nonblocking(true).unwrap();
        s2.set_nonblocking(true).unwrap();
        let tunnel = ProxyConnectionTunnel::new(s1, s2);
        match self.tunnels.lock(){
            Ok(mut e) => e.push(tunnel),
            Err(_) => {}
        }
    }

    /// Main loop for connection (runs in thread.)
    ///
    /// # Arguments
    ///
    /// * `self` - The proxy server itself
    /// # Examples
    ///
    /// ```
    /// /* Proxy server perspective */
    /// let conn = Arc::new(ProxyConnection::new(addr.ip()));
    ///
    /// self.pool.execute( move || {
    ///    let _res = conn2.run().clone();
    /// });
    /// 
    /// ```
    fn run(&self) { 
        loop {
            let mut connections =  self.tunnels.lock().unwrap();

            for i in (0..connections.len()).rev() {
                if connections[i].alive {
                    connections[i].run();
                } else {
                    println!("Closed a connection for {}", self.ipaddr);
                    connections.remove(i);
                }
            }
        }
    }

}

/// A tunnel in a proxy connectin, "connecting" to streams.
struct ProxyConnectionTunnel {
    stream_client: TcpStream,
    stream_endpoint: TcpStream,
    timer: Instant,
    alive: bool
}

impl ProxyConnectionTunnel {
    /// Returns a proxy connection tunnel
    /// 
    /// # Arguments
    ///
    /// * `s1` - First stream for tunnel
    /// * `s1` - Second stream for tunnel
    ///
    /// # Examples
    ///
    /// ```
    /// let tunnel = ProxyConnectionTunnel::new(s1, s2);
    /// ```
    fn new(s1: TcpStream, s2: TcpStream) -> Self {
        Self {
            stream_client: s1,
            stream_endpoint: s2,
            timer: time::Instant::now(),
            alive: true
        }
    }

    /// Main loop for tunnel, reads and forwards requests.
    ///
    /// # Arguments
    ///
    /// * `self` - The proxy server itself
    /// # Examples
    ///
    /// ```
    /// tunnel.run();
    /// ```
    fn run(&mut self){
        /* Forward traffic from client to endpoint */
        let mut buffer = [0 as u8; 2000];
        match self.stream_client.read(&mut buffer){
            Ok(usize) => {
                if usize == 0 {
                    println!("[log] Client has disconnected.");
                    self.alive = false;
                    return;
                }
                let res_usize = match self.stream_endpoint.write(&buffer[0..usize]) {
                    Ok(size) => size,
                    Err(_) => {0}
                };
                self.stream_endpoint.flush().unwrap();
                if res_usize != usize {
                    println!("[warning] Sent size is different from received size!");
                }

                self.timer = time::Instant::now();
            },
            Err(_) => {/*println!("[error client] {}", e)*/}
        }
        
         /* Forward traffic from endpoint to client */
        match self.stream_endpoint.read(&mut buffer){
            Ok(usize) => {
                if usize == 0 {
                    println!("[log] Client has disconnected.");
                    self.alive = false;
                    return;
                }
                let _res_usize = match self.stream_client.write(&buffer[0..usize]){
                    Ok(size) => size,
                    Err(_) => {0}
                };
                self.stream_client.flush().unwrap();

                self.timer = time::Instant::now();
            },
            Err(_) => {/*println!("[error endpoint] {}", e)*/}
        }
        if self.timer.elapsed().as_secs() > 2 {
            self.alive = false;
        }
    }
}