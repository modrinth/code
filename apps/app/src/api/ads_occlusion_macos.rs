use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::CGDisplay;
use core_graphics::geometry::CGRect;
use core_graphics::window::{
    CGWindowID, kCGWindowAlpha, kCGWindowBounds, kCGWindowLayer,
    kCGWindowListExcludeDesktopElements,
    kCGWindowListOptionOnScreenAboveWindow, kCGWindowListOptionOnScreenOnly,
    kCGWindowNumber, kCGWindowOwnerName, kCGWindowOwnerPID,
};
use objc2_app_kit::NSWindow;

pub fn main_window_id(ns_window: *mut std::ffi::c_void) -> Option<CGWindowID> {
    if ns_window.is_null() {
        return None;
    }

    let window_number =
        unsafe { (ns_window as *mut NSWindow).as_ref()?.windowNumber() };

    (window_number > 0).then_some(window_number as CGWindowID)
}

pub fn log_windows_above_main_overlaps<F>(
    main_window_id: CGWindowID,
    mut log_to_js_console: F,
) -> bool
where
    F: FnMut(&str, serde_json::Value),
{
    let Some(window_infos) = CGDisplay::window_list_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        None,
    ) else {
        tracing::debug!("macOS window list query returned no window info");
        log_to_js_console(
            "macOS window list query returned no window info",
            serde_json::json!({}),
        );

        return false;
    };

    let window_infos = unsafe {
        CFArray::<CFDictionary<CFString, CFType>>::wrap_under_get_rule(
            window_infos.as_concrete_TypeRef(),
        )
    };
    let Some(main_window_rect) = window_infos
        .iter()
        .find(|window_info| window_id(window_info) == Some(main_window_id))
        .and_then(|window_info| window_rect(&window_info))
    else {
        tracing::debug!(
            main_window_id,
            window_count = window_infos.len(),
            "Unable to find main app window in macOS window list"
        );
        log_to_js_console(
            "Unable to find main app window in macOS window list",
            serde_json::json!({
                "main_window_id": main_window_id,
                "window_count": window_infos.len(),
            }),
        );

        return false;
    };
    let app_process_id = std::process::id() as i32;
    let Some(windows_above_main) = CGDisplay::window_list_info(
        kCGWindowListOptionOnScreenAboveWindow
            | kCGWindowListExcludeDesktopElements,
        Some(main_window_id),
    ) else {
        tracing::debug!(
            main_window_id,
            "macOS windows-above-main query returned no window info"
        );
        log_to_js_console(
            "macOS windows-above-main query returned no window info",
            serde_json::json!({
                "main_window_id": main_window_id,
            }),
        );

        return false;
    };
    let windows_above_main = unsafe {
        CFArray::<CFDictionary<CFString, CFType>>::wrap_under_get_rule(
            windows_above_main.as_concrete_TypeRef(),
        )
    };
    let mut checked_windows = 0u32;
    let mut skipped_own_process = 0u32;
    let mut skipped_system_owner = 0u32;
    let mut skipped_non_normal_layer = 0u32;
    let mut skipped_transparent = 0u32;
    let mut overlapping_windows = 0u32;

    tracing::debug!(
        main_window_id,
        app_process_id,
        main_x = main_window_rect.origin.x,
        main_y = main_window_rect.origin.y,
        main_width = main_window_rect.size.width,
        main_height = main_window_rect.size.height,
        window_count = windows_above_main.len(),
        "Checking macOS windows above main app window"
    );
    log_to_js_console(
        "Checking macOS windows above main app window",
        serde_json::json!({
            "main_window_id": main_window_id,
            "app_process_id": app_process_id,
            "main_x": main_window_rect.origin.x,
            "main_y": main_window_rect.origin.y,
            "main_width": main_window_rect.size.width,
            "main_height": main_window_rect.size.height,
            "window_count": windows_above_main.len(),
        }),
    );

    for window_info in windows_above_main.iter() {
        checked_windows += 1;

        if window_id(&window_info) == Some(main_window_id) {
            continue;
        }

        if window_process_id(&window_info) == Some(app_process_id) {
            skipped_own_process += 1;

            continue;
        }

        let owner_name = window_owner_name(&window_info);

        if owner_name.as_deref().is_some_and(is_system_window_owner) {
            skipped_system_owner += 1;

            tracing::debug!(
                checked_windows,
                window_id = ?window_id(&window_info),
                owner_pid = ?window_process_id(&window_info),
                owner_name = ?owner_name,
                "Skipping macOS system/window-manager surface"
            );

            continue;
        }

        let layer = window_layer(&window_info);

        if layer != Some(0) {
            skipped_non_normal_layer += 1;

            tracing::debug!(
                checked_windows,
                window_id = ?window_id(&window_info),
                owner_pid = ?window_process_id(&window_info),
                owner_name = ?owner_name,
                layer = ?layer,
                "Skipping macOS window because it is not a normal app window layer"
            );

            continue;
        }

        let alpha = window_alpha(&window_info);

        if alpha.is_some_and(|alpha| alpha <= 0.0) {
            skipped_transparent += 1;

            continue;
        }

        let Some(rect) = window_rect(&window_info) else {
            continue;
        };

        if !intersects(&main_window_rect, &rect) {
            continue;
        }

        overlapping_windows += 1;
        let window_id = window_id(&window_info);
        let owner_pid = window_process_id(&window_info);

        tracing::debug!(
            checked_windows,
            window_id = ?window_id,
            owner_pid = ?owner_pid,
            owner_name = ?owner_name,
            layer = ?layer,
            alpha = ?alpha,
            x = rect.origin.x,
            y = rect.origin.y,
            width = rect.size.width,
            height = rect.size.height,
            "macOS window above main app window overlaps it"
        );
        log_to_js_console(
            "macOS window above main app window overlaps it",
            serde_json::json!({
                "checked_windows": checked_windows,
                "window_id": window_id,
                "owner_pid": owner_pid,
                "owner_name": owner_name,
                "layer": layer,
                "alpha": alpha,
                "x": rect.origin.x,
                "y": rect.origin.y,
                "width": rect.size.width,
                "height": rect.size.height,
            }),
        );
    }

    tracing::debug!(
        checked_windows,
        skipped_own_process,
        skipped_system_owner,
        skipped_non_normal_layer,
        skipped_transparent,
        overlapping_windows,
        "Finished checking macOS windows above main app window"
    );
    log_to_js_console(
        "Finished checking macOS windows above main app window",
        serde_json::json!({
            "checked_windows": checked_windows,
            "skipped_own_process": skipped_own_process,
            "skipped_system_owner": skipped_system_owner,
            "skipped_non_normal_layer": skipped_non_normal_layer,
            "skipped_transparent": skipped_transparent,
            "overlapping_windows": overlapping_windows,
        }),
    );

    overlapping_windows > 0
}

fn window_id(
    window_info: &CFDictionary<CFString, CFType>,
) -> Option<CGWindowID> {
    number_value(window_info, unsafe { kCGWindowNumber })
        .and_then(|value| u32::try_from(value).ok())
}

fn window_process_id(
    window_info: &CFDictionary<CFString, CFType>,
) -> Option<i32> {
    number_value(window_info, unsafe { kCGWindowOwnerPID })
        .and_then(|value| i32::try_from(value).ok())
}

fn window_layer(window_info: &CFDictionary<CFString, CFType>) -> Option<i32> {
    number_value(window_info, unsafe { kCGWindowLayer })
        .and_then(|value| i32::try_from(value).ok())
}

fn window_alpha(window_info: &CFDictionary<CFString, CFType>) -> Option<f64> {
    let key = unsafe { CFString::wrap_under_get_rule(kCGWindowAlpha) };

    window_info.find(&key)?.downcast::<CFNumber>()?.to_f64()
}

fn window_owner_name(
    window_info: &CFDictionary<CFString, CFType>,
) -> Option<String> {
    let key = unsafe { CFString::wrap_under_get_rule(kCGWindowOwnerName) };

    Some(window_info.find(&key)?.downcast::<CFString>()?.to_string())
}

fn is_system_window_owner(owner_name: &str) -> bool {
    matches!(owner_name, "WindowManager")
}

fn number_value(
    window_info: &CFDictionary<CFString, CFType>,
    key: core_foundation::string::CFStringRef,
) -> Option<i64> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };

    window_info.find(&key)?.downcast::<CFNumber>()?.to_i64()
}

fn window_rect(window_info: &CFDictionary<CFString, CFType>) -> Option<CGRect> {
    let key = unsafe { CFString::wrap_under_get_rule(kCGWindowBounds) };
    let bounds = window_info.find(&key)?.downcast::<CFDictionary>()?;
    let rect = CGRect::from_dict_representation(&bounds)?;

    (!is_empty_rect(&rect)).then_some(rect)
}

fn is_empty_rect(rect: &CGRect) -> bool {
    rect.size.width <= 0.0 || rect.size.height <= 0.0
}

fn intersects(a: &CGRect, b: &CGRect) -> bool {
    a.origin.x < b.origin.x + b.size.width
        && a.origin.x + a.size.width > b.origin.x
        && a.origin.y < b.origin.y + b.size.height
        && a.origin.y + a.size.height > b.origin.y
}
