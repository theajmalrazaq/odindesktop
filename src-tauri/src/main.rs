#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod operations;
use operations::{Files, Receipt};
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Manager};

#[derive(Default)]
struct Session {
    receipt: Option<Receipt>,
}
type Shared = Arc<Mutex<Session>>;

fn engine(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if !cfg!(target_os = "linux") {
        return Err("This release supports Linux only.".into());
    }
    // Only execute our bundled engine, never a PATH lookup or a frontend-supplied program.
    let name = if cfg!(target_arch = "aarch64") {
        "odin4-aarch64-unknown-linux-gnu"
    } else {
        "odin4-x86_64-unknown-linux-gnu"
    };
    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(name);
    let production = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .with_file_name("odin4");
    let path = if cfg!(debug_assertions) {
        development
    } else {
        production
    };
    if !path.is_file() {
        return Err(
            "Odin4 engine is missing. Run npm run engine:build before starting the desktop app."
                .into(),
        );
    }
    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(log_dir).map_err(|e| e.to_string())?;
    Ok(path)
}
fn command(app: &tauri::AppHandle) -> Result<Command, String> {
    let mut cmd = Command::new(engine(app)?);
    cmd.current_dir(app.path().app_log_dir().map_err(|e| e.to_string())?);
    Ok(cmd)
}
fn stream(app: &tauri::AppHandle, args: &[String]) -> Result<i32, String> {
    let mut child = command(app)?
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let out = child.stdout.take().ok_or("Missing stdout")?;
    let err = child.stderr.take().ok_or("Missing stderr")?;
    let a = app.clone();
    let b = app.clone();
    let stdout = std::thread::spawn(move || {
        for line in BufReader::new(out).lines() {
            match line {
                Ok(line) => {
                    let _ = a.emit("engine-log", line);
                }
                Err(_) => break,
            }
        }
    });
    let stderr = std::thread::spawn(move || {
        for line in BufReader::new(err).lines() {
            match line {
                Ok(line) => {
                    let _ = b.emit("engine-log", line);
                }
                Err(_) => break,
            }
        }
    });
    let status = child.wait().map_err(|e| e.to_string())?;
    let _ = stdout.join();
    let _ = stderr.join();
    Ok(status.code().unwrap_or(-1))
}
#[tauri::command]
async fn inspect_engine(
    app: tauri::AppHandle,
    state: tauri::State<'_, Shared>,
) -> Result<String, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = state
            .try_lock()
            .map_err(|_| "Another operation is running")?;
        let out = command(&app)?
            .arg("-v")
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("Engine version check failed".into());
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn scan_devices(
    app: tauri::AppHandle,
    state: tauri::State<'_, Shared>,
) -> Result<Vec<String>, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = state
            .try_lock()
            .map_err(|_| "Another operation is running")?;
        let out = command(&app)?
            .arg("-l")
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).into_owned());
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| operations::valid_device(l))
            .map(str::to_string)
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn run_operation(
    app: tauri::AppHandle,
    state: tauri::State<'_, Shared>,
    files: Files,
    device: String,
    flash: bool,
    confirmed: bool,
    reboot: bool,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session=state.try_lock().map_err(|_|"Another operation is running")?;
        if flash && !confirmed {return Err("Explicit flash confirmation is required".into());}
        let previous=session.receipt.take();
        let _=app.emit("engine-log","Checking firmware fingerprints…");
        let receipt=Receipt::capture(&files,&device)?;
        if flash && previous.as_ref()!=Some(&receipt) {return Err("Validate these exact files and this device successfully before flashing.".into());}
        let args=operations::arguments(&files,&device,flash,reboot)?;
        let code=stream(&app,&args)?;
        if code!=0 {return Err(format!("Odin4 exited with code {code}. Review the operation log. Validation has been cleared."));}
        if !flash {
            if Receipt::capture(&files,&device)?!=receipt {return Err("Firmware changed during validation. Validate again.".into());}
            session.receipt=Some(receipt);
        }
        Ok(())
    }).await.map_err(|e|e.to_string())?
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Shared::default())
        .setup(|app| {
            if std::env::args().any(|arg| arg == "--smoke-test") {
                // Exercise the installed GTK/WebKit app and bundled engine without writing to USB.
                let result = (|| -> Result<(), String> {
                    for flag in ["-v", "-l"] {
                        let status = command(app.handle())?
                            .arg(flag)
                            .status()
                            .map_err(|e| e.to_string())?;
                        if !status.success() {
                            return Err(format!("Engine smoke check failed: {flag}"));
                        }
                    }
                    Ok(())
                })();
                if let Err(error) = &result {
                    eprintln!("{error}");
                }
                app.handle().exit(if result.is_ok() { 0 } else { 1 });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Shared>();
                if state.try_lock().is_err() {
                    api.prevent_close();
                    let _ = window.emit(
                        "engine-log",
                        "An operation is still running. Keep the app open until it finishes.",
                    );
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            inspect_engine,
            scan_devices,
            run_operation
        ])
        .run(tauri::generate_context!())
        .expect("Failed to launch Odin Desktop");
}
