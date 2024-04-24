use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{net::TcpStream, thread};
use std::io::{self, Read, Write};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};

const TOKEN: Token = Token(0);

pub fn app_thread_and_poll() -> std::io::Result<()> {
    // get host and port from args
    let args: Vec<String> = std::env::args().collect();
    let host = &args[1];
    let port = &args[2];

    let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;
    let mut stream2 = stream.try_clone()?;

    let working = Arc::new(Mutex::new(true));
    let working_clone = Arc::clone(&working);
    thread::spawn(move || {
        let mut buf = [0; 1056];
        loop {
            let n = stream2.read(&mut buf).unwrap();
            if n == 0 {
                *working_clone.lock().unwrap() = false;
                break;
            }
            let message = String::from_utf8_lossy(&buf[..n]);
            if message.eq("choose_nickname") {
                println!("Enter your nickname: ");
                continue;
            }
            if message.eq("nickname_too_long") {
                println!("Nickname too long");
                *working_clone.lock().unwrap() = false;
                break;
            }
            if message.eq("nickname_taken") {
                println!("Nickname already taken");
                *working_clone.lock().unwrap() = false;
                break;
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
        }
    });
    let stdin = io::stdin();
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
    poll.registry().register(&mut SourceFd(&stdin.as_raw_fd()), TOKEN, Interest::READABLE)?;
    while *working.lock().unwrap() {
        poll.poll(&mut events, Some(Duration::from_millis(0)))?;
        for event in &events {
            match event.token() {
                TOKEN => {
                    let mut message = String::new();
                    std::io::stdin().read_line(&mut message).unwrap();
                    if message.starts_with("/") {
                        if message.starts_with("/exit") {
                            *working.lock().unwrap() = false;
                            break;
                        } else if message.starts_with("/help") {
                            println!("Commands: /exit, /help");
                        } else {
                            println!("Unknown command - try /help");
                        }
                    } else {
                        stream.write(message.as_bytes()).unwrap();
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
