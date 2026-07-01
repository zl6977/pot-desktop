use crate::clipboard::*;
use crate::config::{get, set};
use crate::window::config_window;
use crate::window::input_translate;
use crate::window::ocr_recognize;
use crate::window::ocr_translate;
use crate::window::updater_window;
use log::info;
use tauri::Emitter;
use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;

pub fn create_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    let menu = build_tray_menu(&app_handle, "", "");

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip(&format!("pot {}", app_handle.package_info().version))
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "input_translate" => input_translate(),
                "clipboard_monitor" => on_clipboard_monitor_click(app),
                "copy_source" => on_auto_copy_click(app, "source"),
                "copy_target" => on_auto_copy_click(app, "target"),
                "copy_source_target" => on_auto_copy_click(app, "source_target"),
                "copy_disable" => on_auto_copy_click(app, "disable"),
                "ocr_recognize" => on_ocr_recognize_click(),
                "ocr_translate" => on_ocr_translate_click(),
                "config" => on_config_click(),
                "check_update" => on_check_update_click(),
                "view_log" => on_view_log_click(app),
                "restart" => on_restart_click(app),
                "quit" => on_quit_click(app),
                _ => {}
            }
        })
        .on_tray_icon_event(|_tray, _event| {
            #[cfg(target_os = "windows")]
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                on_tray_click();
            }
        })
        .build(app)?;

    Ok(())
}

#[tauri::command]
pub fn update_tray(app_handle: tauri::AppHandle, language: String, copy_mode: String) {
    let menu = build_tray_menu(&app_handle, &language, &copy_mode);
    if let Some(tray) = app_handle.tray_by_id("main") {
        let _ = tray.set_menu(Some(menu));
        #[cfg(not(target_os = "linux"))]
        let _ = tray.set_tooltip(Some(&format!("pot {}", app_handle.package_info().version)));
    }
}

fn build_tray_menu(app_handle: &tauri::AppHandle, language: &str, copy_mode: &str) -> tauri::menu::Menu<tauri::Wry> {
    let lang = if language.is_empty() {
        match get("app_language") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                set("app_language", "en");
                "en".to_string()
            }
        }
    } else {
        language.to_string()
    };

    let _copy = if copy_mode.is_empty() {
        match get("translate_auto_copy") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                set("translate_auto_copy", "disable");
                "disable".to_string()
            }
        }
    } else {
        copy_mode.to_string()
    };

    let _enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };

    let input_translate = MenuItemBuilder::with_id("input_translate", get_tray_label(&lang, "input_translate")).build(app_handle).unwrap();
    let clipboard_monitor = MenuItemBuilder::with_id("clipboard_monitor", get_tray_label(&lang, "clipboard_monitor")).build(app_handle).unwrap();
    let copy_source = MenuItemBuilder::with_id("copy_source", get_tray_label(&lang, "copy_source")).build(app_handle).unwrap();
    let copy_target = MenuItemBuilder::with_id("copy_target", get_tray_label(&lang, "copy_target")).build(app_handle).unwrap();
    let copy_source_target = MenuItemBuilder::with_id("copy_source_target", get_tray_label(&lang, "copy_source_target")).build(app_handle).unwrap();
    let copy_disable = MenuItemBuilder::with_id("copy_disable", get_tray_label(&lang, "copy_disable")).build(app_handle).unwrap();
    let ocr_recognize = MenuItemBuilder::with_id("ocr_recognize", get_tray_label(&lang, "ocr_recognize")).build(app_handle).unwrap();
    let ocr_translate = MenuItemBuilder::with_id("ocr_translate", get_tray_label(&lang, "ocr_translate")).build(app_handle).unwrap();
    let config = MenuItemBuilder::with_id("config", get_tray_label(&lang, "config")).build(app_handle).unwrap();
    let check_update = MenuItemBuilder::with_id("check_update", get_tray_label(&lang, "check_update")).build(app_handle).unwrap();
    let view_log = MenuItemBuilder::with_id("view_log", get_tray_label(&lang, "view_log")).build(app_handle).unwrap();
    let restart = MenuItemBuilder::with_id("restart", get_tray_label(&lang, "restart")).build(app_handle).unwrap();
    let quit = MenuItemBuilder::with_id("quit", get_tray_label(&lang, "quit")).build(app_handle).unwrap();

    let auto_copy_submenu = SubmenuBuilder::new(app_handle, get_tray_label(&lang, "auto_copy"))
        .item(&copy_source)
        .item(&copy_target)
        .item(&copy_source_target)
        .separator()
        .item(&copy_disable)
        .build()
        .unwrap();

    MenuBuilder::new(app_handle)
        .item(&input_translate)
        .item(&clipboard_monitor)
        .item(&auto_copy_submenu)
        .separator()
        .item(&ocr_recognize)
        .item(&ocr_translate)
        .separator()
        .item(&config)
        .item(&check_update)
        .item(&view_log)
        .separator()
        .item(&restart)
        .item(&quit)
        .build()
        .unwrap()
}

fn get_tray_label(lang: &str, key: &str) -> &'static str {
    match (lang, key) {
        // English
        ("en", "input_translate") => "Input Translate",
        ("en", "clipboard_monitor") => "Clipboard Monitor",
        ("en", "auto_copy") => "Auto Copy",
        ("en", "copy_source") => "Source",
        ("en", "copy_target") => "Target",
        ("en", "copy_source_target") => "Source+Target",
        ("en", "copy_disable") => "Disable",
        ("en", "ocr_recognize") => "OCR Recognize",
        ("en", "ocr_translate") => "OCR Translate",
        ("en", "config") => "Config",
        ("en", "check_update") => "Check Update",
        ("en", "view_log") => "View Log",
        ("en", "restart") => "Restart",
        ("en", "quit") => "Quit",
        // Chinese Simplified
        ("zh_cn", "input_translate") => "输入翻译",
        ("zh_cn", "clipboard_monitor") => "监听剪切板",
        ("zh_cn", "auto_copy") => "自动复制",
        ("zh_cn", "copy_source") => "原文",
        ("zh_cn", "copy_target") => "译文",
        ("zh_cn", "copy_source_target") => "原文+译文",
        ("zh_cn", "copy_disable") => "关闭",
        ("zh_cn", "ocr_recognize") => "文字识别",
        ("zh_cn", "ocr_translate") => "截图翻译",
        ("zh_cn", "config") => "偏好设置",
        ("zh_cn", "check_update") => "检查更新",
        ("zh_cn", "view_log") => "查看日志",
        ("zh_cn", "restart") => "重启应用",
        ("zh_cn", "quit") => "退出",
        // Chinese Traditional
        ("zh_tw", "input_translate") => "輸入翻譯",
        ("zh_tw", "clipboard_monitor") => "偵聽剪貼簿",
        ("zh_tw", "auto_copy") => "自動複製",
        ("zh_tw", "copy_source") => "原文",
        ("zh_tw", "copy_target") => "譯文",
        ("zh_tw", "copy_source_target") => "原文+譯文",
        ("zh_tw", "copy_disable") => "關閉",
        ("zh_tw", "ocr_recognize") => "文字識別",
        ("zh_tw", "ocr_translate") => "截圖翻譯",
        ("zh_tw", "config") => "偏好設定",
        ("zh_tw", "check_update") => "檢查更新",
        ("zh_tw", "view_log") => "查看日誌",
        ("zh_tw", "restart") => "重啓程式",
        ("zh_tw", "quit") => "退出",
        // Japanese
        ("ja", "input_translate") => "翻訳を入力",
        ("ja", "clipboard_monitor") => "クリップボードを監視する",
        ("ja", "auto_copy") => "自動コピー",
        ("ja", "copy_source") => "原文",
        ("ja", "copy_target") => "訳文",
        ("ja", "copy_source_target") => "原文+訳文",
        ("ja", "copy_disable") => "閉じる",
        ("ja", "ocr_recognize") => "テキスト認識",
        ("ja", "ocr_translate") => "スクリーンショットの翻訳",
        ("ja", "config") => "プリファレンス設定",
        ("ja", "check_update") => "更新を確認する",
        ("ja", "view_log") => "ログを見る",
        ("ja", "restart") => "アプリの再起動",
        ("ja", "quit") => "退出する",
        // Korean
        ("ko", "input_translate") => "입력 번역",
        ("ko", "clipboard_monitor") => "감청 전단판",
        ("ko", "auto_copy") => "자동 복사",
        ("ko", "copy_source") => "원문",
        ("ko", "copy_target") => "번역문",
        ("ko", "copy_source_target") => "원문+번역문",
        ("ko", "copy_disable") => "닫기",
        ("ko", "ocr_recognize") => "문자인식",
        ("ko", "ocr_translate") => "스크린샷 번역",
        ("ko", "config") => "기본 설정",
        ("ko", "check_update") => "업데이트 확인",
        ("ko", "view_log") => "로그 보기",
        ("ko", "restart") => "응용 프로그램 다시 시작",
        ("ko", "quit") => "퇴출",
        // French
        ("fr", "input_translate") => "Traduction d'entrée",
        ("fr", "clipboard_monitor") => "Surveiller le presse-papiers",
        ("fr", "auto_copy") => "Copier automatiquement",
        ("fr", "copy_source") => "Source",
        ("fr", "copy_target") => "Cible",
        ("fr", "copy_source_target") => "Source+Cible",
        ("fr", "copy_disable") => "Désactiver",
        ("fr", "ocr_recognize") => "Reconnaissance de texte",
        ("fr", "ocr_translate") => "Traduction d'image",
        ("fr", "config") => "Paramètres",
        ("fr", "check_update") => "Vérifier les mises à jour",
        ("fr", "view_log") => "Voir le journal",
        ("fr", "restart") => "Redémarrer l'application",
        ("fr", "quit") => "Quitter",
        // German
        ("de", "input_translate") => "Eingabeübersetzung",
        ("de", "clipboard_monitor") => "Zwischenablage überwachen",
        ("de", "auto_copy") => "Automatisch kopieren",
        ("de", "copy_source") => "Quelle",
        ("de", "copy_target") => "Ziel",
        ("de", "copy_source_target") => "Quelle+Ziel",
        ("de", "copy_disable") => "Deaktivieren",
        ("de", "ocr_recognize") => "Texterkennung",
        ("de", "ocr_translate") => "Bildübersetzung",
        ("de", "config") => "Einstellungen",
        ("de", "check_update") => "Auf Updates prüfen",
        ("de", "view_log") => "Protokoll anzeigen",
        ("de", "restart") => "Anwendung neu starten",
        ("de", "quit") => "Beenden",
        // Russian
        ("ru", "input_translate") => "\u{0412}\u{0432}\u{043e}\u{0434} \u{043f}\u{0435}\u{0440}\u{0435}\u{0432}\u{043e}\u{0434}\u{0430}",
        ("ru", "clipboard_monitor") => "\u{0421}\u{043b}\u{0435}\u{0434}\u{0438}\u{0442}\u{044c} \u{0437}\u{0430} \u{0431}\u{0443}\u{0444}\u{0435}\u{0440}\u{043e}\u{043c} \u{043e}\u{0431}\u{043c}\u{0435}\u{043d}\u{0430}",
        ("ru", "auto_copy") => "\u{0410}\u{0432}\u{0442}\u{043e}\u{043c}\u{0430}\u{0442}\u{0438}\u{0447}\u{0435}\u{0441}\u{043a}\u{043e}\u{0435} \u{043a}\u{043e}\u{043f}\u{0438}\u{0440}\u{043e}\u{0432}\u{0430}\u{043d}\u{0438}\u{0435}",
        ("ru", "copy_source") => "\u{0418}\u{0441}\u{0442}\u{043e}\u{0447}\u{043d}\u{0438}\u{043a}",
        ("ru", "copy_target") => "\u{0426}\u{0435}\u{043b}\u{044c}",
        ("ru", "copy_source_target") => "\u{0418}\u{0441}\u{0442}\u{043e}\u{0447}\u{043d}\u{0438}\u{043a}+\u{0426}\u{0435}\u{043b}\u{044c}",
        ("ru", "copy_disable") => "\u{041e}\u{0442}\u{043a}\u{043b}\u{044e}\u{0447}\u{0438}\u{0442}\u{044c}",
        ("ru", "ocr_recognize") => "\u{0420}\u{0430}\u{0441}\u{043f}\u{043e}\u{0437}\u{043d}\u{0430}\u{0432}\u{0430}\u{043d}\u{0438}\u{0435} \u{0442}\u{0435}\u{043a}\u{0441}\u{0442}\u{0430}",
        ("ru", "ocr_translate") => "\u{041f}\u{0435}\u{0440}\u{0435}\u{0432}\u{043e}\u{0434} \u{0438}\u{0437}\u{043e}\u{0431}\u{0440}\u{0430}\u{0436}\u{0435}\u{043d}\u{0438}\u{044f}",
        ("ru", "config") => "\u{041d}\u{0430}\u{0441}\u{0442}\u{0440}\u{043e}\u{0439}\u{043a}\u{0438}",
        ("ru", "check_update") => "\u{041f}\u{0440}\u{043e}\u{0432}\u{0435}\u{0440}\u{0438}\u{0442}\u{044c} \u{043e}\u{0431}\u{043d}\u{043e}\u{0432}\u{043b}\u{0435}\u{043d}\u{0438}\u{044f}",
        ("ru", "view_log") => "\u{041f}\u{0440}\u{043e}\u{0441}\u{043c}\u{043e}\u{0442}\u{0440} \u{0436}\u{0443}\u{0440}\u{043d}\u{0430}\u{043b}\u{0430}",
        ("ru", "restart") => "\u{041f}\u{0435}\u{0440}\u{0435}\u{0437}\u{0430}\u{043f}\u{0443}\u{0441}\u{0442}\u{0438}\u{0442}\u{044c} \u{043f}\u{0440}\u{0438}\u{043b}\u{043e}\u{0436}\u{0435}\u{043d}\u{0438}\u{0435}",
        ("ru", "quit") => "\u{0412}\u{044b}\u{0445}\u{043e}\u{0434}",
        // Portuguese (Brazil)
        ("pt_br", "input_translate") => "Traduzir Entrada",
        ("pt_br", "clipboard_monitor") => "Monitorando a área de transferência",
        ("pt_br", "auto_copy") => "Copiar Automaticamente",
        ("pt_br", "copy_source") => "Origem",
        ("pt_br", "copy_target") => "Destino",
        ("pt_br", "copy_source_target") => "Origem+Destino",
        ("pt_br", "copy_disable") => "Desabilitar",
        ("pt_br", "ocr_recognize") => "Reconhecimento de Texto",
        ("pt_br", "ocr_translate") => "Tradução de Imagem",
        ("pt_br", "config") => "Configurações",
        ("pt_br", "check_update") => "Checar por Atualização",
        ("pt_br", "view_log") => "Exibir Registro",
        ("pt_br", "restart") => "Reiniciar aplicativo",
        ("pt_br", "quit") => "Sair",
        // Persian
        ("fa", "input_translate") => "\u{0645}\u{062a}\u{0646}",
        ("fa", "clipboard_monitor") => "\u{06af}\u{0648}\u{0634} \u{062f}\u{0627}\u{062f}\u{0646} \u{0628}\u{0647} \u{062a}\u{062e}\u{062a}\u{0647} \u{0628}\u{0631}\u{0634}",
        ("fa", "auto_copy") => "\u{06a}\u{0645}\u{067} \u{06a}\u{0634}\u{0648}\u{0627}\u{0631}",
        ("fa", "copy_source") => "\u{0645}\u{0646}\u{0628}\u{0639}",
        ("fa", "copy_target") => "\u{0647}\u{062f}\u{0641}",
        ("fa", "copy_source_target") => "\u{0645}\u{0646}\u{0628}\u{0639} + \u{0647}\u{062f}\u{0641}",
        ("fa", "copy_disable") => "\u{0645}\u{062a}\u{0646}",
        ("fa", "ocr_recognize") => "\u{062a}\u{0634}\u{062e}\u{06cc}\u{0635} \u{0645}\u{062a}\u{0646}",
        ("fa", "ocr_translate") => "\u{062a}\u{0631}\u{062c}\u{0645}\u{0647} \u{0639}\u{06a}\u{0634}",
        ("fa", "config") => "\u{062a}\u{0646}\u{0638}\u{06cc}\u{0645}\u{0627}\u{062a} \u{062a}\u{0631}\u{062c}\u{06cc}\u{062d}",
        ("fa", "check_update") => "\u{0628}\u{0631}\u{0633}\u{06cc} \u{0628}\u{0631}\u{0648}\u{0632}\u{0631}\u{0633}\u{0627}\u{0646}\u{06cc}",
        ("fa", "view_log") => "\u{0645}\u{0634}\u{0627}\u{0647}\u{062f}\u{0647} \u{06af}\u{0632}\u{0627}\u{0631}\u{0634}\u{0627}\u{062a}",
        ("fa", "restart") => "\u{0631}\u{0627}\u{0647}\u{06d2}\u{0622}\u{0646}\u{0627}\u{06cc}\u{06cc} \u{0645}\u{062c}\u{062f}\u{062f} \u{0628}\u{0631}\u{0646}\u{0627}\u{0645}\u{0647}",
        ("fa", "quit") => "\u{062e}\u{0631}\u{0648}\u{062c}",
        // Ukrainian
        ("uk", "input_translate") => "\u{0412}\u{0432}\u{0456}\u{0434}\u{0435}\u{043d}\u{043d}\u{044f} \u{043f}\u{0435}\u{0440}\u{0435}\u{043a}\u{043b}\u{0430}\u{0434}\u{0443}",
        ("uk", "clipboard_monitor") => "\u{0421}\u{0442}\u{0435}\u{0436}\u{0438}\u{0442}\u{0438} \u{0437}\u{0430} \u{0431}\u{0443}\u{0444}\u{0435}\u{0440}\u{043e}\u{043c} \u{043e}\u{0431}\u{043c}\u{0456}\u{043d}\u{0443}",
        ("uk", "auto_copy") => "\u{0410}\u{0432}\u{0442}\u{043e}\u{043c}\u{0430}\u{0442}\u{0438}\u{0447}\u{043d}\u{0435} \u{043a}\u{043e}\u{043f}\u{0456}\u{044e}\u{0432}\u{0430}\u{043d}\u{043d}\u{044f}",
        ("uk", "copy_source") => "\u{0414}\u{0436}\u{0435}\u{0440}\u{0435}\u{043b}\u{043e}",
        ("uk", "copy_target") => "\u{041c}\u{0435}\u{0442}\u{0430}",
        ("uk", "copy_source_target") => "\u{0414}\u{0436}\u{0435}\u{0440}\u{0435}\u{043b}\u{043e}+\u{041c}\u{0435}\u{0442}\u{0430}",
        ("uk", "copy_disable") => "\u{0412}\u{0456}\u{0434}\u{043a}\u{043b}\u{044e}\u{0447}\u{0438}\u{0432}\u{0448}\u{0438}",
        ("uk", "ocr_recognize") => "\u{0420}\u{043e}\u{0437}\u{043f}\u{0456}\u{0437}\u{043d}\u{0430}\u{0432}\u{043b}\u{0435}\u{043d}\u{043d}\u{044f} \u{0442}\u{0435}\u{043a}\u{0441}\u{0442}\u{0443}",
        ("uk", "ocr_translate") => "\u{041f}\u{0435}\u{0440}\u{0435}\u{043a}\u{043b}\u{0430}\u{0434} \u{0437}\u{043e}\u{0432}\u{043d}\u{0456}\u{0441}\u{043d}\u{044f}",
        ("uk", "config") => "\u{041d}\u{0430}\u{043b}\u{0430}\u{0448}\u{0442}\u{0443}\u{0432}\u{0430}\u{043d}\u{043d}\u{044f}",
        ("uk", "check_update") => "\u{041f}\u{0435}\u{0440}\u{0435}\u{0432}\u{0456}\u{0440}\u{0438}\u{0442}\u{0438} \u{043e}\u{043d}\u{043e}\u{0432}\u{043b}\u{0435}\u{043d}\u{043d}\u{044f}",
        ("uk", "view_log") => "\u{041f}\u{0435}\u{0440}\u{0435}\u{0433}\u{043b}\u{044f}\u{0434} \u{0436}\u{0443}\u{0440}\u{043d}\u{0430}\u{043b}\u{0443}",
        ("uk", "restart") => "\u{041f}\u{0435}\u{0440}\u{0435}\u{0437}\u{0430}\u{043f}\u{0443}\u{0441}\u{0442}\u{0438}\u{0442}\u{0438} \u{0434}\u{043e}\u{0434}\u{0430}\u{0442}\u{043e}\u{043a}",
        ("uk", "quit") => "\u{0412}\u{0438}\u{0445}\u{0456}\u{0434}",
        // Default to English
        _ => match key {
            "input_translate" => "Input Translate",
            "clipboard_monitor" => "Clipboard Monitor",
            "auto_copy" => "Auto Copy",
            "copy_source" => "Source",
            "copy_target" => "Target",
            "copy_source_target" => "Source+Target",
            "copy_disable" => "Disable",
            "ocr_recognize" => "OCR Recognize",
            "ocr_translate" => "OCR Translate",
            "config" => "Config",
            "check_update" => "Check Update",
            "view_log" => "View Log",
            "restart" => "Restart",
            "quit" => "Quit",
            _ => "",
        },
    }
}

fn on_tray_click() {
    let event = match get("tray_click_event") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("tray_click_event", "config");
            "config".to_string()
        }
    };
    match event.as_str() {
        "config" => config_window(),
        "translate" => input_translate(),
        "ocr_recognize" => ocr_recognize(),
        "ocr_translate" => ocr_translate(),
        "disable" => {}
        _ => config_window(),
    }
}

fn on_clipboard_monitor_click(app: &tauri::AppHandle) {
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let current = !enable_clipboard_monitor;
    set("clipboard_monitor", current);
    let state = app.state::<ClipboardMonitorEnableWrapper>();
    state
        .0
        .lock()
        .unwrap()
        .replace_range(.., &current.to_string());
    if current {
        start_clipboard_monitor(app.clone());
    }
}

fn on_auto_copy_click(app: &tauri::AppHandle, mode: &str) {
    info!("Set copy mode to: {}", mode);
    set("translate_auto_copy", mode);
    let _ = app.emit("translate_auto_copy_changed", mode);
    update_tray(app.clone(), "".to_string(), mode.to_string());
}

fn on_ocr_recognize_click() {
    ocr_recognize();
}

fn on_ocr_translate_click() {
    ocr_translate();
}

fn on_config_click() {
    config_window();
}

fn on_check_update_click() {
    updater_window();
}

fn on_view_log_click(_app: &tauri::AppHandle) {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    let log_dir = dirs::cache_dir()
        .unwrap()
        .join(crate::config::APP_ID)
        .join("log");
    #[cfg(target_os = "macos")]
    let log_dir = dirs::cache_dir()
        .unwrap()
        .join(crate::config::APP_ID)
        .join("log");
    let log_path = log_dir.join("log.txt");
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&log_path).spawn().ok();
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&log_path).spawn().ok();
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&log_path).spawn().ok();
}

fn on_restart_click(app: &tauri::AppHandle) {
    info!("============== Restart App ==============");
    app.restart();
}

fn on_quit_click(app: &tauri::AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app.global_shortcut().unregister_all();
    info!("============== Quit App ==============");
    app.exit(0);
}
