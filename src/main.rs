use std::cmp::min;
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};


fn handle_connection(mut connection: TcpStream) -> Result<()> {
    let mut bytes_read = 0;
    let mut request = [0u8; 1024];

    // Read request
    println!("reading from connection");
    loop {
        let num_bytes = Read::by_ref(&mut connection).take(5).read(&mut request[bytes_read..])?;
        // let num_bytes = connection.read(&mut request[bytes_read..])?;
        println!("read {num_bytes} bytes");
        bytes_read += num_bytes;

        if num_bytes == 0 {
            println!("client disconnected unexpectedly while reading request");
            return Ok(());
        }

        if request.get(bytes_read - 4..bytes_read) == Some(b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8_lossy(&request[..bytes_read]);
    println!("finished reading request: {request}");

    // Write response
    println!("writing to connection");
    let response = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Content-Length: 12\n",
        "Connection: close\r\n\r\n",
        "Hello world!"
    ).as_bytes();
    let response_bytes = response.len();

    let mut bytes_written = 0;

    loop {
        let num_bytes = connection.write(&response[bytes_written..min(bytes_written + 5, response_bytes)])?;
        // let num_bytes = connection.write(&response[bytes_written..])?;
        bytes_written += num_bytes;
        println!("wrote {num_bytes} bytes, {bytes_written}/{response_bytes}");

        if num_bytes == 0 {
            println!("client disconnected unexpectedly while writing response");
            return Ok(());
        }

        if bytes_written == response_bytes {
            break;
        }
    }

    // Flush response
    println!("flushing response");
    connection.flush()
}

fn main() {
    // Blocking TcpListener
    let blocking_listener = TcpListener::bind("localhost:3000").unwrap();
    println!("waiting to accept connection");
    let (connection, socket_addr) = blocking_listener.accept().unwrap();
    println!("accepted connection to {socket_addr}");

    if let Err(e) = handle_connection(connection) {
        println!("failed to handle connection {e}")
    }
}
