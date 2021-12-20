extern crate threadpool;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, IpAddr};
use std::time::Instant;
use std::{time};
use threadpool::ThreadPool;
use std::sync::{Mutex, Arc};

pub struct ProxyServer  {
    clients_list: Vec<Arc<ProxyConnection>>,
    server_socket: TcpListener,
    pool: ThreadPool
}

impl ProxyServer {

    pub fn new(addr: &str) -> Self {
        Self {
            server_socket: TcpListener::bind(addr).unwrap(),
            pool: ThreadPool::new(20),
            clients_list: Vec::<Arc<ProxyConnection>>::new()
        }
    }

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

    pub fn run(&mut self) {
        loop {
            match self.server_socket.accept() {
                Ok((stream, addr)) => {

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
struct ProxyConnection {
    tunnels: Mutex<Vec<ProxyConnectionTunnel>>,
    ipaddr: IpAddr
}

impl ProxyConnection {
    fn new(addr: IpAddr) -> Self {
        Self {
            tunnels: Mutex::new(Vec::new()),
            ipaddr: addr
        }
    }

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

    fn add_tunnel(&self, s1: TcpStream, s2: TcpStream) {
        s1.set_nonblocking(true).unwrap();
        s2.set_nonblocking(true).unwrap();
        let tunnel = ProxyConnectionTunnel::new(s1, s2);
        self.tunnels.lock().unwrap().push(tunnel);
    }

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

struct ProxyConnectionTunnel {
    stream_client: TcpStream,
    stream_endpoint: TcpStream,
    timer: Instant,
    alive: bool
}

impl ProxyConnectionTunnel {

    fn new(s1: TcpStream, s2: TcpStream) -> Self {
        Self {
            stream_client: s1,
            stream_endpoint: s2,
            timer: time::Instant::now(),
            alive: true
        }
    }

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
                let res_usize = self.stream_endpoint.write(&buffer[0..usize]).unwrap();
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
                let _res_usize = self.stream_client.write(&buffer[0..usize]).unwrap();
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