use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    thread,
};

use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    name: String,
    #[structopt(short, long)]
    addr: SocketAddr,
}

fn background_thread(stream: TcpStream) -> anyhow::Result<()> {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let mut stream = TcpStream::connect(opt.addr)?;
    let stream2 = stream.try_clone()?;
    thread::spawn(move || {
        if let Err(e) = background_thread(stream2) {
            eprintln!("Error: {:?}", e);
        }
    });

    for line in stdin().lines() {
        let line = line?;
        writeln!(stream, "{}: {}", opt.name, line)?;
    }

    Ok(())
}
