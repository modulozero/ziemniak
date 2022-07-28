// Copyright 2022 ModZero.
// SPDX-License-Identifier: 	AGPL-3.0-or-later
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod timers;

fn main() {
    tauri::Builder::default()
        .plugin(timers::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
