
use eframe::egui::mutex::Mutex;
// lib
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::net::{TcpListener, TcpStream};
use std::io::Write;
use std::sync::Arc;

use opencv::videoio;
use opencv::prelude::{VideoCaptureTrait, VectorToVec};
use opencv::core::Mat;

#[derive(Deserialize, Clone)]
pub struct Camera {
    pub vid: String,
}


#[derive(Deserialize, Serialize, Clone)]
pub struct Robot {
    id: u32,
    temperature: f32,
    robot_pos: Vec<f32>,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub camera: Camera,
    pub robot: Robot,
}

pub fn gen_rand_info(mut robot: Robot) -> Robot {
    robot.temperature += rand::random_range(-0.5..0.5);
    for pos in &mut robot.robot_pos[..3] {
        *pos += rand::random_range(-1.0..1.0);
    }
    robot
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // считывание config.toml
    let str_var = std::fs::read_to_string("./config.toml")?;
    let config = toml::from_str(&str_var)?;
    Ok(config)
}

fn load_video_capturer(conf: &Camera) -> Result<videoio::VideoCapture, String> {
    let parts: Vec<&str> = conf.vid.split(":").collect();

    let capture = match parts.as_slice() {
        ["FILE", path] => {
            videoio::VideoCapture::from_file(*path, 0)
                .map_err(|e| e.to_string())?
        }
        ["DEVICE", idx_str] => {
            let idx = idx_str.parse::<i32>().map_err(|e| e.to_string())?;
            videoio::VideoCapture::new(idx, videoio::CAP_V4L2)
                .map_err(|e| e.to_string())?
        }
        _ => return Err("Invalid camera format. Expected 'FILE:/path' or 'DEVICE:/dev_index'".to_string()),
    };

    Ok(capture)
}

pub fn load_video_frame(capturer: &mut videoio::VideoCapture) -> Result<Mat, Box<dyn std::error::Error>> {
    let mut frame = Mat::default();
    if capturer.read(&mut frame)? {
        Ok(frame)
    } else {
        Err("Кадр не читается".into())
    }
}

pub fn send_frame(frame: Mat, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let mut buf = opencv::core::Vector::new();
    opencv::imgcodecs::imencode_def(".jpg", &frame, &mut buf).unwrap();
    let image_data = buf.to_vec();
    let data_size = image_data.len() as u32;
    stream.write_all(&data_size.to_be_bytes()).unwrap();
    stream.write_all(&image_data)
}



pub fn send_robot_info(robot_info: &Robot, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let string_json = serde_json::to_string_pretty(robot_info)?;
    let data_len = string_json.len() as u32;
    
    stream.write_all(&data_len.to_be_bytes())?;
    stream.write_all(string_json.as_bytes())?;
    
    Ok(())
}
fn main() {
        let conf = load_config().expect("Error getting config from TOML file");
    let mut video_capturer = load_video_capturer(&conf.camera).expect("loading video error");
    let robot = conf.robot;

    let listener = TcpListener::bind("127.0.0.1:7878").expect("Не удалось открыть порт");
    println!("Сервер запущен и ожидает подключения...");

    // let robot = conf.robot;
    let new_robot_info = Arc::new(Mutex::new(robot));

    let robot_logger = Arc::clone(&new_robot_info);

    std::thread::spawn(move || {
        loop {
            let log_entry = {
                let info = robot_logger.lock();
                format!(
                    "[{}] ID: {}, Температура: {:.2}°C, Позиция: [{:}, {:}, {:}]\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    info.id,
                    info.temperature,
                    info.robot_pos[0],
                    info.robot_pos[1],
                    info.robot_pos[2]
                )
            };

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("robot_state.log")
                .expect("Не удалось открыть файл логов");

            file.write_all(log_entry.as_bytes())
                .expect("Ошибка записи в файл");

            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Клиент подключён");
                loop {
                    // Отправка видео
                    match load_video_frame(&mut video_capturer) {
                        Ok(frame) => {
                            if let Err(e) = send_frame(frame, &mut stream) {
                                eprintln!("Ошибка отправки кадра: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Ошибка чтения кадра: {}", e);
                        }
                    }

                    // Обновление данных робота
                    {
                        let mut robot_info = new_robot_info.lock();
                        *robot_info = gen_rand_info(robot_info.clone());
                    }

                    // Отправка данных робота
                    let robot_data = new_robot_info.lock();
                    if let Err(e) = send_robot_info(&robot_data, &mut stream) {
                        eprintln!("Ошибка отправки данных дрона: {}", e);
                    }

                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
            }
            Err(e) => {
                eprintln!("Ошибка подключения: {}", e);
            }
        }
    }
}