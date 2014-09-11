use std::io::{IoResult, IoError, IoErrorKind, TcpStream, ConnectionRefused,
							ConnectionFailed, OtherIoError};

pub struct Socks4a {
	sockshost: String,
	socksport: u16,
	host: String,
	port: u16
}

impl Socks4a {
	pub fn new(host: String, port: u16) -> Socks4a {
		let tmphost = "";
		let tmpport = 0;
		Socks4a { sockshost: host, socksport: port, host: tmphost.to_string(), 
			port: tmpport }
	}
	pub fn connect(&mut self, host: String, port: u16) {
		self.host = host;
		self.port = port;
	}
	pub fn build(&mut self) -> IoResult<TcpStream> {
		let mut stream = try!(TcpStream::connect(self.sockshost.as_slice(),
																						 self.socksport));
		try!(stream.write([0x04, 0x01]));
		try!(stream.write_be_u16(self.port));
		try!(stream.write([0x00, 0x00, 0x00, 0x01]));
		try!(stream.write([0x00]));
		try!(stream.write_str(self.host.as_slice()));
		try!(stream.write([0x00]));

		// read null byte
		if 0 != try!(stream.read_u8()) {
			return io_err(OtherIoError, "Expected null byte not found");
		}

		// read status
		match try!(stream.read_u8()) {
			// request granted
			0x5a => {
				let _port = try!(stream.read_be_u16());
				let _ip = try!(stream.read_be_u32());

				Ok(stream)
			}
			// request rejected or failed
			0x5b => io_err(ConnectionRefused, "Request rejected or failed"),
			// request failed because client is not running identd (or unreachable)
			0x5c => io_err(ConnectionFailed, "Client unreachable"),
			// request failed because client's identd could not confirm the user ID
			// string in the request
			0x5d => io_err(ConnectionRefused, "Unknown user"),
			x => fail!("Unexpected status byte: {}", x)
		}
	}
}

fn io_err<T>(kind: IoErrorKind, desc: &'static str) -> IoResult<T> {
	Err(IoError { kind: kind, desc: desc, detail: None })
}

