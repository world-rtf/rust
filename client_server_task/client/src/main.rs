
use prost::Message;
use prost_types::Timestamp;
use std::io::Result;
use std::{io::Write, net::TcpStream, time::Duration, thread};
use std::time::{SystemTime, UNIX_EPOCH};


pub struct DHT {
    humidity: f32,
    temperature: f32,
}

impl DHT {


    pub fn new() -> Self {
        Self {
            humidity: rand::random_range(0.0..120.0),
            temperature: rand::random_range(-40.0..40.0),  
        }
    }

    pub fn get_humidity(&mut self) -> f32 {
        self.humidity += rand::random_range(-10.0..10.0);
        self.humidity = self.humidity.clamp(0.0, 120.0);
        self.humidity
    }


    pub fn get_temperature(&mut self) -> f32 {
        self.temperature += rand::random_range(-10.0..10.0);
        self.temperature = self.temperature.clamp(-40.0, 40.0);
        self.temperature
    }

    
}




mod data {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

pub struct Config {
    device_id: u32,
    address: String,
    port: String
}

impl Config {

    pub fn new(device_id: u32, address: impl Into<String>, port: impl Into<String>) -> Self {
        Config {
            device_id,
            address: address.into(),
            port: port.into(),
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

}



pub struct SERVER {
    config: Config,
    event_id: u64,
    dht: DHT
}

impl SERVER {
    pub fn new(config: Config) -> Self {
        SERVER {
            config,
            event_id: 0,
            dht: DHT::new()
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            thread::sleep(Duration::from_secs(1));
            let data = data::Data {
                device_id: self.config.device_id,
                event_id: self.event_id,
                humidity: self.dht.get_humidity(),
                temperature: self.dht.get_temperature(),
                read_time: Some(current_timestamp()),
                ..Default::default()
            };
            self.event_id += 1;

            let mut stream = match TcpStream::connect(self.config.addr()) {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                    continue;
                }
            };

            let proto_data = data.encode_to_vec();
            let len_bytes = (proto_data.len() as u32).to_le_bytes();
            
            stream.write_all(&len_bytes)?;
            stream.write_all(&proto_data)?;
            stream.flush()?;
        }
    }
}


fn current_timestamp() -> Timestamp {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    
    Timestamp {
        seconds: duration.as_secs() as i64,
        nanos: duration.subsec_nanos() as i32,
    }
}


fn main() -> Result<()> {
    let config = Config::new( 121, "127.0.0.1","7878");
    SERVER::new(config).run()
}