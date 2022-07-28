// Copyright 2022 ModZero.
// SPDX-License-Identifier: 	AGPL-3.0-or-later

use std::{
    collections::HashMap,
    error::Error,
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::{DateTime, FixedOffset, Local};
use tauri::{
    async_runtime::spawn,
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime, State, Manager,
};
use tokio::time::interval;
use uuid::Uuid;

#[derive(serde::Serialize, Debug)]
pub enum TimerError {
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
pub struct Timer {
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

    fn is_complete(&self) -> bool {
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
        let elapsed = now
            - match self.checked {
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
    fn make(&self, duration: Duration, message: &str) -> Timer {
        let timer = Timer::new(message, duration);

        self.0.lock().unwrap().insert(timer.id, timer.clone());

        timer
    }

    fn delete(&self, timer_id: Uuid) -> Option<Timer> {
        self.0.lock().unwrap().get(&timer_id).cloned()
    }

    fn start(&self, timer_id: Uuid) -> Result<Timer, TimerError> {
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

    fn tick(&self, timer_id: Uuid) -> Result<Timer, TimerError> {
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
fn make(timers: State<'_, Timers>, duration: Duration, message: &str) -> Timer {
    timers.make(duration, message)
}

#[tauri::command]
fn delete(timers: State<'_, Timers>, timer_id: Uuid) -> Option<Timer> {
    timers.delete(timer_id)
}

#[tauri::command]
fn start<R: Runtime>(
    _app: AppHandle<R>,
    timers: State<'_, Timers>,
    timer_id: Uuid,
) -> Result<Timer, TimerError> {
    let timers = Timers(timers.0.to_owned());
    let timer = timers.start(timer_id)?;

    spawn(async move {
        let mut interval = interval(Duration::from_secs(1) / 60);

        loop {
            interval.tick().await;
            match timers.tick(timer_id) {
                Err(_) => break,
                Ok(timer) => {
                    if timer.is_complete() {
                        _app.emit_all("timer-done", timer).ok();
                        break;
                    }
                    if _app.emit_all("timer-tick", timer).is_err() {
                        break;
                    }
                }
            }
        }
    });

    Ok(timer)
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("timers")
        .invoke_handler(tauri::generate_handler![
            delete,
            make,
            start
        ])
        .setup(|app_handle| {
            app_handle.manage(Timers::default());
            Ok(())
        })
        .build()
}
