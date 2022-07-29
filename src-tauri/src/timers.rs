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
    AppHandle, Manager, Runtime, State,
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
    #[serde(skip)]
    checked: Option<Instant>,
    version: u64,
}

impl Timer {
    fn new(duration: Duration) -> Self {
        Self {
            id: Uuid::new_v4(),
            duration,
            ..Default::default()
        }
    }

    fn is_complete(&self) -> bool {
        self.elapsed.map_or(false, |e| e >= self.duration)
    }

    fn start(&mut self) {
        let now = Local::now().into();
        self.started = Some(now);
        self.elapsed = Some(Duration::from_secs(0));
        self.checked = Some(Instant::now());
        self.version += 1;
    }

    /// Increment the timer, returning the time since last tick
    fn tick(&mut self) -> Result<(), TimerError> {
        let now = Instant::now();
        match self.checked {
            None => Err(TimerError::NotStarted),
            Some(checked) => {
                self.elapsed = Some(now - checked);
                self.checked = Some(checked);
                self.version += 1;

                Ok(())
            }
        }
    }

    fn reset(&mut self, duration: Duration) {
        self.duration = duration;
        self.started = None;
        self.elapsed = None;
        self.checked = None;
        self.version += 1;
    }
}

#[derive(Default)]
struct Timers(Arc<Mutex<HashMap<Uuid, Timer>>>);

impl Timers {
    fn make(&self, duration: Duration) -> Timer {
        let timer = Timer::new(duration);

        self.0.lock().unwrap().insert(timer.id, timer.clone());

        timer
    }

    fn delete(&self, timer_id: Uuid) -> Option<Timer> {
        self.0.lock().unwrap().get(&timer_id).cloned()
    }

    fn start(&self, timer_id: Uuid) -> Result<Timer, TimerError> {
        let mut timers = self.0.lock().unwrap();
        match timers.get_mut(&timer_id) {
            None => Err(TimerError::NotFound),
            Some(t) => {
                t.start();

                Ok(t.clone())
            }
        }
    }

    fn tick(&self, timer_id: Uuid) -> Result<Timer, TimerError> {
        let mut timers = self.0.lock().unwrap();
        match timers.get_mut(&timer_id) {
            None => Err(TimerError::NotFound),
            Some(t) => t.tick().and(Ok(t.clone())),
        }
    }

    fn reset(&self, timer_id: Uuid, duration: Duration) -> Result<Timer, TimerError> {
        let mut timers = self.0.lock().unwrap();
        match timers.get_mut(&timer_id) {
            None => Err(TimerError::NotFound),
            Some(t) => {
                t.reset(duration);

                Ok(t.clone())
            }
        }
    }
}

#[tauri::command]
fn make(timers: State<'_, Timers>, duration: Duration) -> Timer {
    timers.make(duration)
}

#[tauri::command]
fn delete(timers: State<'_, Timers>, timer_id: Uuid) -> Option<Timer> {
    timers.delete(timer_id)
}

#[tauri::command]
fn reset<R: Runtime>(
    _app: AppHandle<R>,
    timers: State<'_, Timers>,
    timer_id: Uuid,
    duration: Duration,
) -> Result<Timer, TimerError> {
    let res = timers.reset(timer_id, duration);

    if let Ok(timer) = &res {
        _app.emit_all("timer-update", timer).ok();
    }

    res
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
            match &timers.tick(timer_id) {
                Err(_) => break, // Timer is gone or no longer running, we're done
                Ok(timer) => {
                    _app.emit_all("timer-update", timer).ok();
                    if timer.is_complete() {
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
        .invoke_handler(tauri::generate_handler![delete, make, reset, start,])
        .setup(|app_handle| {
            app_handle.manage(Timers::default());
            Ok(())
        })
        .build()
}
