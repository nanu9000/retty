use log::{debug, info, error};

use std::cmp::min;
use std::io::{ErrorKind, Read, Result, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut connection: TcpStream) -> Result<()> {
    let mut bytes_read = 0;
    let mut request = [0u8; 1024];

    // Read request
    info!("reading from connection");
    loop {
        let num_bytes = Read::by_ref(&mut connection).take(5).read(&mut request[bytes_read..])?;
        // let num_bytes = connection.read(&mut request[bytes_read..])?;
        // println!("DEBUG read {num_bytes} bytes");
        bytes_read += num_bytes;

        if num_bytes == 0 {
            error!("client disconnected unexpectedly while reading request");
            return Ok(());
        }

        if request.get(bytes_read - 4..bytes_read) == Some(b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8_lossy(&request[..bytes_read]);
    debug!("finished reading request: {request}");

    // TODO Base the response off the input provided
    // Write response
    info!("writing to connection");
    let response = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Content-Length: 12\n",
        "Connection: close\r\n\r\n",
        "COMPLETED!\n\n"
    ).as_bytes();
    let response_bytes = response.len();

    let mut bytes_written = 0;

    loop {
        let num_bytes = connection.write(&response[bytes_written..min(bytes_written + 5, response_bytes)])?;
        // let num_bytes = connection.write(&response[bytes_written..])?;
        bytes_written += num_bytes;
        debug!("wrote {num_bytes} bytes, {bytes_written}/{response_bytes}");

        if num_bytes == 0 {
            error!("client disconnected unexpectedly while writing response");
            return Ok(());
        }

        if bytes_written == response_bytes {
            break;
        }
    }

    // Flush response
    info!("flushing response");
    connection.flush()
}

fn blocking_listener_primitive() {
    let listener = TcpListener::bind("localhost:3000").unwrap();

    loop {
        // Primitive blocking TcpListener
        info!("about to wait to accept connection");
        let (connection, _) = listener.accept().unwrap();
        info!("accepted connection");

        if let Err(e) = handle_connection(connection) {
            error!("failed to handle connection {e}")
        }
    }
}

fn blocking_listener_spawns_threads() {
    let listener = TcpListener::bind("localhost:3000").unwrap();

    loop {
        // Spawning threads for each connection
        info!("about to wait to accept connection");
        let (connection, _) = listener.accept().unwrap();
        info!("accepted connection");

        std::thread::spawn(|| {
            if let Err(e) = handle_connection(connection) {
                error!("failed to handle connection {e}")
            }
        });
    }
}

enum ConnectionState {
    Read {
        request: [u8; 1024],
        read: usize,
    },
    Write {
        response: &'static [u8],
        written: usize,
    },
    Flush,
}

fn non_blocking_listener() {
    let listener = TcpListener::bind("localhost:3000").unwrap();
    listener.set_nonblocking(true).unwrap(); // Unwrap to make sure call succeeded

    let mut connections = Vec::new();

    loop {
        match listener.accept() {
            Ok((connection, _)) => {
                connection.set_nonblocking(true).unwrap();
                let state = ConnectionState::Read { request: [0u8; 1024], read: 0 };
                connections.push((connection, state));
                info!("added new connection to queue");
            },
            Err(e) if e.kind() == ErrorKind::WouldBlock => {},
            Err(e) => panic!("{e}"),
        }

        let mut completed = Vec::new();

        'next: for (i, (connection, state)) in connections.iter_mut().enumerate() {
            if let ConnectionState::Read { request, read } = state {
                loop {
                    match connection.read(request) {
                        Ok(0) => {
                            error!("client disconnected unexpectedly while reading request {i}");
                            completed.push(i);
                            continue 'next;
                        }
                        Ok(num_bytes) => *read += num_bytes,
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {
                            debug!("would block during read");
                            continue 'next;
                        },
                        // Err(e) if e.kind() == ErrorKind::WouldBlock => continue 'next,
                        Err(e) => panic!("{e}"),
                    }

                    if request.get(*read - 4..*read) == Some(b"\r\n\r\n") {
                        break;
                    }
                }

                let request = String::from_utf8_lossy(&request[..*read]);
                info!("finished reading request: {request}");

                let response = concat!(
                    "HTTP/1.1 200 OK\r\n",
                    "Content-Length: 12\n",
                    "Connection: close\r\n\r\n",
                    "Hello world!"
                );
            
                *state = ConnectionState::Write { // 👈
                    response: response.as_bytes(),
                    written: 0,
                };
            }
            if let ConnectionState::Write { response, written } = state {
                loop {
                    match connection.write(response) {
                        Ok(0) => {
                            error!("client disconnected unexpectedly while writing request {i}");
                            completed.push(i);
                            continue 'next;
                        }
                        Ok(num_bytes) => *written += num_bytes,
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {
                            debug!("would block during write");
                            continue 'next;
                        },
                        // Err(e) if e.kind() == ErrorKind::WouldBlock => continue 'next,
                        Err(e) => panic!("{e}"),
                    }

                    if *written == response.len() {
                        break;
                    }
                }

                *state = ConnectionState::Flush;
            }
            if let ConnectionState::Flush = state {
                loop {
                    match connection.flush() {
                        Ok(_) => {
                            completed.push(i);
                            break;
                        }
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {
                            debug!("would block during flush");
                            continue 'next;
                        },
                        // Err(e) if e.kind() == ErrorKind::WouldBlock => continue 'next,
                        Err(e) => panic!("{e}"),
                    }
                }
            }
        }

        // iterate in reverse order to preserve the indices
        for i in completed.into_iter().rev() {
            info!("finished processing request {i}");
            connections.remove(i);
        }
    }
}

fn main() {
    env_logger::init();
    // TODO Use a CLI args parser to determine which listener impl to use
    // blocking_listener_primitive();
    // blocking_listener_spawns_threads();
    non_blocking_listener();
}
