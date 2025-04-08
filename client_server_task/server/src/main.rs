    use prost::Message;
    use std::net::{TcpListener, TcpStream};
    use std::io::{BufReader, Read};
    use postgres::{Client, NoTls};
    use chrono::{TimeZone, Utc};
    use std::env;
    mod data {
        include!(concat!(env!("OUT_DIR"), "/_.rs"));
    }

    struct Database(Client);

    impl Database {
        fn new() -> Self {
            // let mut client = Client::connect("host=localhost user=postgres password=0330", NoTls).unwrap();

            let database_url = env::var("DATABASE_URL")
            .expect("nnn");
            let mut client = Client::connect(&database_url, NoTls)
                .unwrap();


            
            client.batch_execute(
                "CREATE TABLE IF NOT EXISTS sensor_data (
                    device_id BIGINT NOT NULL,
                    event_id BIGINT NOT NULL,
                    humidity REAL NOT NULL,
                    temperature REAL NOT NULL,
                    read_time TIMESTAMP NOT NULL
                )"
            ).unwrap();
            Self(client)
        }

        fn save(&mut self, data: &data::Data) {

            let read_time = data.read_time.as_ref().map(|ts| {
                Utc.timestamp_opt(ts.seconds, ts.nanos as u32)
                    .single()
                    .unwrap()
                    .naive_utc()
            }).unwrap();

            println!("DATA: {:?}", data);

            self.0.execute(
                "INSERT INTO sensor_data VALUES ($1, $2, $3, $4, $5)",
                &[
                    &(data.device_id as i64),
                    &(data.event_id as i64),
                    &data.humidity,
                    &data.temperature,
                    &read_time
                ]
            ).unwrap();
        }
    }

    fn handle_client(stream: TcpStream, db: &mut Database) {
        let mut reader = BufReader::new(stream);
        let mut len_buf = [0u8; 4];

        while reader.read_exact(&mut len_buf).is_ok() {
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut proto_data = vec![0u8; len];
            if reader.read_exact(&mut proto_data).is_err() {
                break;
            }

            if let Ok(data) = data::Data::decode(&proto_data[..]) {
                println!("Data from device {}", data.device_id);
                db.save(&data);
            }
        }
    }

    fn main() {
        let mut db = Database::new();
        let address = env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "7878".to_string());
        let listen_addr = format!("{}:{}", address, port);
        
        let listener = TcpListener::bind(&listen_addr).unwrap();
        println!("Server started on {}", listen_addr);
        // let mut db = Database::new();
        // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        // println!("Server started on 127.0.0.1:7878");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_client(stream, &mut db),
                Err(e) => eprintln!("Connection error: {}", e),
            }
        }
    }