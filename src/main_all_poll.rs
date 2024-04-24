use core::fmt;
use std::os::fd::AsRawFd;
use std::time::Duration;
use std::net::TcpStream;
use std::io::{self, Error, Read, Write};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};

const TOKEN_STDIN: Token = Token(0);
const TOKEN_SOCKET: Token = Token(1);

#[derive(Debug, Clone)]
struct EndStreamError;

impl fmt::Display for EndStreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "stream ended")
    }
}

fn process_message(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buf = [0; 1056];
    let n = stream.read(&mut buf).unwrap();
    if n == 0 {
        return Err(Error::new(io::ErrorKind::ConnectionReset, "remote end ended connection"));
    }
    let message = String::from_utf8_lossy(&buf[..n]);
    if message.eq("choose_nickname") {
        println!("Enter your nickname: ");
        return Ok(());
    }
    if message.eq("nickname_too_long") {
        println!("Nickname too long");
        return Err(Error::new(io::ErrorKind::Other, "nickname too long"));
    }
    if message.eq("nickname_taken") {
        println!("Nickname already taken");
        return Err(Error::new(io::ErrorKind::Other, "nickname already taken"));
    }
    let nick_end = message.find("\n");
    match nick_end {
        Some(nick_end) => {
            let nickname = &message[..nick_end];
            println!("{}: {}", nickname, &message[nick_end+1..]);
        },
        None => {
            println!("{}", message);
        }
    
    }
    Ok(())
}

pub fn app_all_poll() -> std::io::Result<()> {
    // get host and port from args
    let args: Vec<String> = std::env::args().collect();
    let host = &args[1];
    let port = &args[2];

    let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;

    let stdin = io::stdin();
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
    poll.registry().register(&mut SourceFd(&stdin.as_raw_fd()), TOKEN_STDIN, Interest::READABLE)?;
    poll.registry().register(&mut SourceFd(&stream.as_raw_fd()), TOKEN_SOCKET, Interest::READABLE)?;
    'outer_loop:
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(0)))?;
        for event in &events {
            match event.token() {
                TOKEN_STDIN => {
                    let mut message = String::new();
                    std::io::stdin().read_line(&mut message).unwrap();
                    if message.starts_with("/") {
                        if message.starts_with("/exit") {
                            break 'outer_loop;
                        } else if message.starts_with("/help") {
                            println!("Commands: /exit, /help");
                        } else {
                            println!("Unknown command - try /help");
                        }
                    } else {
                        stream.write(message.as_bytes()).unwrap();
                    }
                },
                TOKEN_SOCKET => {
                    if process_message(&mut stream).is_err() {
                        break 'outer_loop;
                    }
                },
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    println!("Exiting...");
    Ok(())

}
