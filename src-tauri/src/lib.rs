use tauri::{AppHandle, Manager, Emitter};
use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU32, Ordering};
use tauri_plugin_store::StoreExt;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct PopupData {
    source: String,
    content: String,
    raw_message: String,
    reasons: Vec<String>,
    images: Vec<String>,
    time: String,
    group_id: String,
    group_name: String,
    msg_time: f64,
    local_id: u64,
    is_highlight: bool,
    match_reason: String,
    content_dup_info: Option<serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize)]
struct WindowState {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

static POPUP_COUNTER: AtomicU32 = AtomicU32::new(0);

fn clean_text_for_copy(text: String) -> String {
    let mut result = text;
    
    // 第一步：删除零宽字符
    result = result.replace('\u{200B}', "") // 零宽空格
                 .replace('\u{200C}', "") // 零宽非连接符
                 .replace('\u{200D}', "") // 零宽连接符
                 .replace('\u{FEFF}', ""); // 字节顺序标记
    
    // 第二步：删除开头的所有空行和空白字符（只删除开头的，保留中间和结尾的）
    let mut chars = result.chars().peekable();
    let mut skip_count = 0;
    
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            skip_count += 1;
            chars.next();
        } else {
            break;
        }
    }
    
    if skip_count > 0 {
        result = result.chars().skip(skip_count).collect();
    }
    
    // 第三步：删除结尾的所有空白字符
    result = result.trim_end().to_string();
    
    result
}

use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

struct PopupDataStore {
    data: HashMap<String, serde_json::Value>,
}

impl PopupDataStore {
    fn new() -> Self {
        Self { data: HashMap::new() }
    }
}

static POPUP_STORE: OnceLock<Mutex<PopupDataStore>> = OnceLock::new();

fn get_popup_store() -> &'static Mutex<PopupDataStore> {
    POPUP_STORE.get_or_init(|| Mutex::new(PopupDataStore::new()))
}

#[tauri::command]
async fn debug_log(message: String) {
    println!("[前端] {}", message);
}

#[tauri::command]
async fn get_popup_data(popup_id: String) -> Option<serde_json::Value> {
    let store = get_popup_store().lock().unwrap();
    store.data.get(&popup_id).cloned()
}

#[tauri::command]
async fn clear_popup_data(popup_id: String) {
    let mut store = get_popup_store().lock().unwrap();
    store.data.remove(&popup_id);
}

#[tauri::command]
async fn copy_to_clipboard(_app: AppHandle, text: String, image_url: Option<String>) -> Result<(), String> {
    let text = clean_text_for_copy(text);
    
    if let Some(url) = image_url {
        let full_url = if url.starts_with("http") {
            url
        } else if url.starts_with("proxy.php?url=") {
            let encoded = url.strip_prefix("proxy.php?url=").unwrap_or(&url);
            urlencoding_decode(encoded)
        } else {
            format!("https://{}", url)
        };
        
        let response = reqwest::get(&full_url)
            .await
            .map_err(|e| format!("下载图片失败: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP错误: {}", response.status()));
        }
        
        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("读取数据失败: {}", e))?;
        
        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);
        
        let html = format!(
            "<html><body><div style=\"white-space: pre-wrap; font-family: sans-serif; margin: 0; padding: 0;\">{}</div><img src=\"data:image/jpeg;base64,{}\"></body></html>",
            escape_html(&text),
            base64_data
        );
        
        use clipboard_rs::{Clipboard, ClipboardContext};
        let ctx = ClipboardContext::new().map_err(|e| format!("创建剪贴板上下文失败: {:?}", e))?;
        
        ctx.set_text(text.clone()).map_err(|e| format!("设置文本失败: {:?}", e))?;
        ctx.set_html(html).map_err(|e| format!("设置 HTML 失败: {:?}", e))?;
    } else {
        use clipboard_rs::{Clipboard, ClipboardContext};
        let ctx = ClipboardContext::new().map_err(|e| format!("创建剪贴板上下文失败: {:?}", e))?;
        ctx.set_text(text).map_err(|e| format!("设置文本失败: {:?}", e))?;
    }
    
    Ok(())
}

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    
    result
}

#[tauri::command]
async fn fetch_image(url: String) -> Result<String, String> {
    let full_url = if url.starts_with("http") {
        url
    } else if url.starts_with("proxy.php?url=") {
        let encoded = url.strip_prefix("proxy.php?url=").unwrap_or(&url);
        urlencoding_decode(encoded)
    } else {
        format!("https://{}", url)
    };
    
    let response = reqwest::get(&full_url)
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP错误: {}", response.status()));
    }
    
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取数据失败: {}", e))?;
    
    use base64::Engine;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", content_type, base64_data);
    
    Ok(data_url)
}

#[tauri::command]
async fn show_highlight_popup(app: AppHandle, data: PopupData) -> Result<String, String> {
    let popup_id = format!("highlight-popup-{}", POPUP_COUNTER.fetch_add(1, Ordering::SeqCst));
    
    let default_width = 400.0;
    let default_height = 300.0;

    let saved_state = app.store("window-state.json")
        .ok()
        .and_then(|s| s.get("popup_state"))
        .and_then(|v| {
            let x = v.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
            let y = v.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
            let width = v.get("width").and_then(|v| v.as_u64()).map(|v| v as u32);
            let height = v.get("height").and_then(|v| v.as_u64()).map(|v| v as u32);
            if let (Some(x), Some(y), Some(w), Some(h)) = (x, y, width, height) {
                Some((x, y, w, h))
            } else {
                None
            }
        });

    let (x, y, width, height) = if let Some((sx, sy, sw, sh)) = saved_state {
        (sx, sy, sw, sh)
    } else {
        let monitor = app.primary_monitor()
            .map_err(|e| format!("获取显示器失败: {}", e))?;

        if let Some(m) = monitor {
            let screen_size = m.size();
            let scale_factor = m.scale_factor();
            let screen_width = screen_size.width as f64 / scale_factor;
            let screen_height = screen_size.height as f64 / scale_factor;

            let existing_popups = get_existing_popup_count(&app);
            let offset = (existing_popups * 30) as f64;

            let x = (screen_width - default_width) / 2.0 + offset;
            let y = (screen_height - default_height) / 2.0 + offset;
            (x as i32, y as i32, default_width as u32, default_height as u32)
        } else {
            (100, 100, default_width as u32, default_height as u32)
        }
    };
    
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        &popup_id,
        tauri::WebviewUrl::App(format!("popup.html#{}", popup_id).into()),
    )
    .title("重点消息")
    .inner_size(width as f64, height as f64)
    .min_inner_size(300.0, 150.0)
    .max_inner_size(800.0, 2000.0)
    .always_on_top(true)
    .decorations(true)
    .transparent(false)
    .skip_taskbar(false)
    .resizable(true)
    .focused(true)
    .position(x as f64, y as f64)
    .visible(true)
    .build()
    .map_err(|e| format!("创建弹窗失败: {}", e))?;
    
    let _ = window.show();
    let window_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = window_clone.set_focus();
    });

    {
        let mut store = get_popup_store().lock().unwrap();
        store.data.insert(popup_id.clone(), serde_json::to_value(&data).unwrap_or(serde_json::Value::Null));
    }

    Ok(popup_id)
}

fn get_existing_popup_count(app: &AppHandle) -> u32 {
    let windows = app.webview_windows();
    windows.keys()
        .filter(|k| k.starts_with("highlight-popup-"))
        .count() as u32
}

#[tauri::command]
async fn close_popup(app: AppHandle, popup_id: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&popup_id) {
        println!("[Rust] 关闭弹窗: {}", popup_id);
        match window.destroy() {
            Ok(_) => println!("[Rust] 弹窗 {} 已销毁", popup_id),
            Err(e) => {
                println!("[Rust] destroy 失败: {}, 尝试 close", e);
                let _ = window.close();
            }
        }
    } else {
        println!("[Rust] 弹窗 {} 不存在", popup_id);
    }
    Ok(())
}

#[tauri::command]
async fn show_context_window(
    app: AppHandle,
    group_id: String,
    current_local_id: u64,
    msg_time: f64,
) -> Result<(), String> {
    let win_id = format!("context-{}", group_id);
    
    if let Some(window) = app.get_webview_window(&win_id) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        &win_id,
        tauri::WebviewUrl::App("context.html".into()),
    )
    .title("消息上下文")
    .inner_size(500.0, 600.0)
    .min_inner_size(400.0, 400.0)
    .max_inner_size(1200.0, 1200.0)
    .always_on_top(false)
    .decorations(true)
    .transparent(false)
    .skip_taskbar(false)
    .resizable(true)
    .focused(true)
    .visible(true)
    .build()
    .map_err(|e| format!("创建上下文窗口失败: {}", e))?;
    
    let _ = window.show();
    let _ = window.set_focus();
    
    let window_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = window_clone.emit("context-data", serde_json::json!({
            "groupId": group_id,
            "currentLocalId": current_local_id,
            "msgTime": msg_time
        }));
    });
    
    Ok(())
}

#[tauri::command]
async fn get_context_messages(
    _app: AppHandle,
    _group_id: String,
    _current_local_id: u64,
    _msg_time: f64,
) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
async fn set_window_always_on_top(app: AppHandle, enabled: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(enabled)
            .map_err(|e| format!("设置置顶失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn save_window_state(app: AppHandle, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    if x < -10000 || y < -10000 || width < 200 || height < 300 {
        return Ok(());
    }
    
    let store = app.store("window-state.json")
        .map_err(|e| format!("获取存储失败: {}", e))?;

    let state = WindowState { x, y, width, height };
    store.set("main_window", serde_json::to_value(state).map_err(|e| format!("序列化失败: {}", e))?);

    if let Err(e) = store.save() {
        return Err(format!("保存失败: {}", e));
    }

    Ok(())
}

#[tauri::command]
async fn load_window_state(app: AppHandle) -> Result<Option<WindowState>, String> {
    let store = app.store("window-state.json")
        .map_err(|e| format!("获取存储失败: {}", e))?;

    if let Some(value) = store.get("main_window") {
        match serde_json::from_value::<WindowState>(value) {
            Ok(state) => {
                return Ok(Some(state));
            }
            Err(e) => {
                eprintln!("[窗口状态] 解析失败: {}", e);
            }
        }
    }

    Ok(None)
}

#[tauri::command]
async fn save_popup_state(app: AppHandle, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    let store = app.store("window-state.json")
        .map_err(|e| format!("获取存储失败: {}", e))?;
    
    let state = serde_json::json!({ "x": x, "y": y, "width": width, "height": height });
    store.set("popup_state", state);
    store.save().map_err(|e| format!("保存失败: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn load_popup_state(app: AppHandle) -> Result<Option<(i32, i32)>, String> {
    let store = app.store("window-state.json")
        .map_err(|e| format!("获取存储失败: {}", e))?;
    
    if let Some(value) = store.get("popup_position") {
        let x = value.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
        let y = value.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
        
        if let (Some(x), Some(y)) = (x, y) {
            return Ok(Some((x, y)));
        }
    }
    
    Ok(None)
}

#[tauri::command]
async fn minimize_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

#[tauri::command]
async fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
async fn request_show_context(
    app: AppHandle,
    group_id: String,
    group_name: String,
    local_id: u64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("show-context-request", serde_json::json!({
            "groupId": group_id,
            "groupName": group_name,
            "localId": local_id
        }));
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::Manager;
                use tauri::tray::TrayIconBuilder;
                use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
                
                let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
                let hide_item = MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?;
                let separator = PredefinedMenuItem::separator(app)?;
                let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                
                let menu = Menu::with_items(app, &[&show_item, &hide_item, &separator, &quit_item])?;
                
                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .icon(tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| {
                        match event.id.as_ref() {
                            "show" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "hide" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.hide();
                                }
                            }
                            "quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click { 
                            button: tauri::tray::MouseButton::Left,
                            button_state: tauri::tray::MouseButtonState::Up,
                            .. 
                        } = event {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let window_ref: tauri::WebviewWindow = window;
                                if let Ok(visible) = window_ref.is_visible() {
                                    if visible {
                                        let _ = window_ref.hide();
                                    } else {
                                        let _ = window_ref.show();
                                        let _ = window_ref.set_focus();
                                    }
                                }
                            }
                        }
                    })
                    .build(app)?;
                
                if let Some(window) = app.get_webview_window("main") {
                    #[cfg(target_os = "macos")]
                    window.set_shadow(true)?;
                    
                    if let Ok(store) = app.store("window-state.json") {
                        if let Some(value) = store.get("main_window") {
                            if let Ok(state) = serde_json::from_value::<WindowState>(value) {
                                if state.x > -10000 && state.y > -10000 && state.width >= 200 && state.height >= 300 {
                                    let _ = window.set_position(tauri::PhysicalPosition::new(state.x, state.y));
                                    let _ = window.set_size(tauri::PhysicalSize::new(state.width, state.height));
                                }
                            }
                        }
                    }
                    
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            debug_log,
            get_popup_data,
            clear_popup_data,
            copy_to_clipboard,
            fetch_image,
            show_highlight_popup,
            close_popup,
            show_context_window,
            get_context_messages,
            set_window_always_on_top,
            save_window_state,
            load_window_state,
            save_popup_state,
            load_popup_state,
            minimize_to_tray,
            show_window,
            request_show_context,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            _ => {}
        });
}
