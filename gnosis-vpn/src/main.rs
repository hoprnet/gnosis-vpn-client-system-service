use clap::Parser;
use std::io;
use std::io::Read;
use std::os::unix::net;
use std::path::Path;

/// Gnosis VPN system service - offers interaction commands on Gnosis VPN to other applications.
#[derive(Parser)]
struct Cli {
    /// communication socket name - will be created by installer and should be /run/gnosisvpn/service.sock
    socket: String,
}

fn incoming(mut stream: net::UnixStream) -> Result<(), io::Error> {
    let mut buffer = [0; 128];
    let size = stream.read(&mut buffer)?;
    log::info!("incoming: {}", String::from_utf8_lossy(&buffer[..size]));
    Ok(())
}

fn daemon(socket: &String) -> Result<(), io::Error> {
    let res = Path::try_exists(Path::new(socket));

    let receiver = match res {
        Ok(true) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Daemon already running",
        )),
        Ok(false) => net::UnixListener::bind(socket),
        Err(x) => Err(x),
    }?;

    for stream in receiver.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("incoming stream {:?}", stream);
                incoming(stream).expect("FOOBAR");
            }
            Err(x) => {
                log::error!("Error waiting for incoming message: {:?}", x)
            }
        }
    }

    Ok(())
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let res = daemon(&args.socket);
    log::info!("patthern: {:?}, result: {:?}", args.socket, res);
}
