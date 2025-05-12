use std::net::TcpStream;
use std::io::Read;
use std::sync::{Arc, Mutex};

use eframe::{egui, epaint::ColorImage, NativeOptions};
use image::load_from_memory;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Robot {
    pub id: u32,
    pub temperature: f32,
    pub robot_pos: Vec<f32>,
}

struct VideoClient {
    current_frame: Arc<Mutex<Option<Arc<ColorImage>>>>,
    texture_handle: Option<egui::TextureHandle>,
    robot_info: Arc<Mutex<Option<Robot>>>,
}

impl VideoClient {
    fn new() -> Self {
        Self {
            current_frame: Arc::new(Mutex::new(None)),
            texture_handle: None,
            robot_info: Arc::new(Mutex::new(None)),
        }
    }

    fn start_receiving(&self) {
        let frame_arc = self.current_frame.clone();
        let robot_info_arc = self.robot_info.clone();

        std::thread::spawn(move || {
            loop {
                match TcpStream::connect("127.0.0.1:7878") {
                    Ok(mut stream) => {
                        println!("Подключено к серверу");
                        // data
                        Self::receive_frames(&mut stream, &frame_arc, &robot_info_arc);
                    }
                    Err(e) => {
                        eprintln!("Ошибка подключения: {}", e);
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    }
                }
            }
        });
    }

    fn receive_frames(stream: &mut TcpStream, frame_arc: &Arc<Mutex<Option<Arc<ColorImage>>>>, robot_info_arc: &Arc<Mutex<Option<Robot>>>) {
        loop {
            // Получаем кадр
            if let Some(image) = Self::recv_image_frame(stream) {
                *frame_arc.lock().unwrap() = Some(Arc::new(image));
            } else {
                break;
            }

            // Получаем данные робота
            if let Some(robot) = Self::recv_robot_info(stream) {
                *robot_info_arc.lock().unwrap() = Some(robot);
            } else {
                break;
            }
        }
    }

    fn recv_image_frame(stream: &mut TcpStream) -> Option<ColorImage> {
        let size = read_stream(stream)?;
        let mut buf = vec![0u8; size as _];
        stream.read_exact(&mut buf).ok()?;
        
        let img = load_from_memory(&buf).ok()?;
        let rgb = img.to_rgb8();
        let dimensions = [rgb.width() as _, rgb.height() as _];
        Some(ColorImage::from_rgb(dimensions, rgb.as_raw()))
    }

    fn recv_robot_info(stream: &mut TcpStream) -> Option<Robot> {
        let size = read_stream(stream)?;
        let mut buf = vec![0u8; size as _];
        stream.read_exact(&mut buf).ok()?;
        
        let json = String::from_utf8(buf).ok()?;
        serde_json::from_str(&json).ok()
    }
}


fn read_stream(stream: &mut TcpStream) -> Option<u32> {
    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf).ok().map(|_| u32::from_be_bytes(size_buf))
}

impl eframe::App for VideoClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_video_frame(ui);
            ui.separator();
            self.show_robot_info(ui);
        });
        ctx.request_repaint();
    }
}

impl VideoClient {
    fn show_video_frame(&mut self, ui: &mut egui::Ui) {
        let frame_lock = self.current_frame.lock().unwrap();
        if let Some(image) = &*frame_lock {
            if self.texture_handle.is_none() {
                self.texture_handle = Some(ui.ctx().load_texture(
                    "video_frame",
                    (**image).clone(),
                    Default::default(),
                ));
            }

            if let Some(texture) = &mut self.texture_handle {
                texture.set((**image).clone(), Default::default());
            }

            if let Some(texture) = &self.texture_handle {
                ui.image(texture);
            }
        } else {
            ui.label("Чилим...");
        }
    }

    fn show_robot_info(&self, ui: &mut egui::Ui) {
        let robot_info = self.robot_info.lock().unwrap();
        if let Some(ref robot) = *robot_info {
            ui.label(format!("ID робота: {}", robot.id));
            ui.label(format!("Температура: {:}°C", robot.temperature));
            ui.label(format!(
                "Позиция: [{:}, {:}, {:}]",
                robot.robot_pos[0], robot.robot_pos[1], robot.robot_pos[2]
            ));
        } else {
            ui.label("Данные дрона не получены");
        }
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };

    let app = VideoClient::new();
    app.start_receiving();

    eframe::run_native(
        "Видео клиент",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ).unwrap();
}