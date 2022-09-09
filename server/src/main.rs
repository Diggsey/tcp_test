use std::{
    io::{self, BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    addr: SocketAddr,
}

fn client_thread(
    stream: Result<TcpStream, io::Error>,
    tx: mpsc::SyncSender<Command>,
) -> anyhow::Result<()> {
    let stream = stream?;

    tx.send(Command::Add(stream.try_clone()?))?;

    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        tx.send(Command::Broadcast(line))?;
    }

    Ok(())
}

#[derive(Debug)]
enum Command {
    Add(TcpStream),
    Broadcast(String),
}

fn broadcast_thread(rx: mpsc::Receiver<Command>) {
    let mut clients = Vec::new();
    for cmd in rx {
        println!("Command: {:?}", cmd);
        match cmd {
            Command::Add(stream) => clients.push(stream),
            Command::Broadcast(message) => {
                clients.retain_mut(|stream| match writeln!(stream, "{}", message) {
                    Ok(()) => true,
                    Err(e) => {
                        eprintln!("Error writing to client: {:?}", e);
                        false
                    }
                })
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let listener = TcpListener::bind(opt.addr)?;
    let (tx, rx) = mpsc::sync_channel(64);

    thread::spawn(move || broadcast_thread(rx));

    for stream in listener.incoming() {
        let tx = tx.clone();
        thread::spawn(move || {
            if let Err(e) = client_thread(stream, tx) {
                eprintln!("Client error: {:?}", e);
            }
        });
    }

    Ok(())
}
