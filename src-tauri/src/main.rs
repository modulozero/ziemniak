#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Local};
use tauri::{async_runtime::spawn, Window};
use tokio::time::interval;
use uuid::Uuid;

#[derive(Clone, Default, serde::Serialize)]
struct Timer {
    id: Uuid,
    started: Option<DateTime<FixedOffset>>,
    duration: Duration,
    elapsed: Option<Duration>,
    message: String,
}

impl Timer {
    fn new(message: &str, duration: Duration) -> Self {
        return Self {
            id: Uuid::new_v4(),
            duration: duration,
            message: message.to_string(),
            ..Default::default()
        };
    }

    fn complete(self: &Self) -> bool {
        return self.elapsed.map_or(false, |e| e >= self.duration);
    }

    async fn run(self: &mut Self, window: Window) {
        self.started = Some(Local::now().into());
        let mut elapsed = Duration::from_secs(0);
        self.elapsed = Some(elapsed);
        let mut last_checked = Instant::now();

        let mut interval = interval(Duration::from_secs(1) / 60);
        loop {
            interval.tick().await;
            let now = Instant::now();
            let duration = now - last_checked;

            elapsed = elapsed + duration;
            self.elapsed = Some(elapsed);

            if self.complete() {
                break;
            }

            if let Err(_) = window.emit("timer-tick", self.clone()) {
                break;
            }
            last_checked = now;
        }

        window
            .emit("timer-done", self.clone())
            .expect("Our window went away?");
    }
}

#[tauri::command]
fn start_timer(window: Window, duration: Duration, message: &str) -> Uuid {
    let mut timer = Timer::new(message, duration);
    let timer_id = timer.id.clone();

    spawn(async move {
        timer.run(window).await;
    });

    timer_id
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_timer])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
