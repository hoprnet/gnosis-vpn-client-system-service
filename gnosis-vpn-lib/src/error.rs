use std::io;

pub enum Error {
    ServiceNotRunning,
    SocketPathIO(io::Error),
    ConnectSocketIO(io::Error),
    CommandSerialization(serde_json::Error),
    WriteSocketIO(io::Error),
    ReadSocketIO(io::Error),
}
