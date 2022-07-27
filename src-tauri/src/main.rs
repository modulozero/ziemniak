#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant}, fmt, error::Error,
};

use chrono::{DateTime, FixedOffset, Local};
use tauri::{State, Window, async_runtime::spawn};
use tokio::time::interval;
use uuid::Uuid;

#[derive(serde::Serialize, Debug)]
enum TimerError {
    NotFound,
    NotStarted,
}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimerError::NotFound => write!(f, "timer not found"),
            TimerError::NotStarted => write!(f, "timer not started"),
        }
    }
}

impl Error for TimerError {}

#[derive(Clone, Default, serde::Serialize)]
struct Timer {
    id: Uuid,
    started: Option<DateTime<FixedOffset>>,
    duration: Duration,
    elapsed: Option<Duration>,
    message: String,
    #[serde(skip)]
    checked: Option<Instant>,
}

impl Timer {
    fn new(message: &str, duration: Duration) -> Self {
        Self {
            id: Uuid::new_v4(),
            duration,
            message: message.to_string(),
            ..Default::default()
        }
    }

    fn complete(&self) -> bool {
        self.elapsed.map_or(false, |e| e >= self.duration)
    }

    fn start(self) -> Self {
        Timer {
            started: Some(Local::now().into()),
            elapsed: Some(Duration::from_secs(0)),
            checked: Some(Instant::now()),
            ..self
        }
    }

    fn tick(self) -> Result<Self, TimerError> {
        let now = Instant::now();
        let elapsed = now - match self.checked {
            None => return Err(TimerError::NotStarted),
            Some(checked) => checked,
        };

        Ok(Timer {
            elapsed: self.elapsed.map(|e| e + elapsed),
            checked: Some(now),
            ..self
        })
    }
}

#[derive(Default)]
struct Timers(Arc<Mutex<HashMap<Uuid, Timer>>>);

impl Timers {
    fn make_timer(&self, duration: Duration, message: &str) -> Timer {
        let timer = Timer::new(message, duration);

        self.0.lock().unwrap().insert(timer.id, timer.clone());

        timer
    }

    fn delete_timer(&self, timer_id: Uuid) -> Option<Timer> {
        self.0.lock().unwrap().get(&timer_id).cloned()
    }

    fn start_timer(&self, timer_id: Uuid) -> Result<Timer, TimerError> {
        let mut timers = self.0.lock().unwrap();
        match timers.get(&timer_id).cloned() {
            None => Err(TimerError::NotFound),
            Some(t) => {
                let started = t.start();
                timers.insert(started.id, started.clone());

                Ok(started)
            }
        }
    }

    fn tick_timer(&self, timer_id: Uuid) -> Result<Timer, TimerError> {
        let mut timers = self.0.lock().unwrap();
        match timers.get(&timer_id).cloned() {
            None => Err(TimerError::NotFound),
            Some(t) => {
                let started = t.tick()?;
                timers.insert(started.id, started.clone());

                Ok(started)
            }
        }
    }

}

#[tauri::command]
fn make_timer(timers: State<'_, Timers>, duration: Duration, message: &str) -> Timer {
    timers.make_timer(duration, message)
}

#[tauri::command]
fn delete_timer(timers: State<'_, Timers>, timer_id: Uuid) -> Option<Timer> {
    timers.delete_timer(timer_id)
}

#[tauri::command]
fn start_timer(window: Window, timers: State<'_, Timers>, timer_id: Uuid) -> Result<Timer, TimerError> {
    let timers = Timers(timers.0.to_owned());
    let timer = timers.start_timer(timer_id)?;
    let res = timer.clone();

    spawn(async move {
        let mut interval = interval(Duration::from_secs(1) / 60);
        
        loop {
            interval.tick().await;
            match timers.tick_timer(timer_id) {
                Err(_) => break,
                Ok(timer) => {
                    if timer.complete() || window.emit("timer-tick", timer).is_err() {
                        break;
                    }
                }
            }
        }

        window.emit("timer-done", timer).ok();
    });

    Ok(res)
}

fn main() {
    tauri::Builder::default()
        .manage(Timers(Default::default()))
        .invoke_handler(tauri::generate_handler![
            delete_timer,
            make_timer,
            start_timer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
