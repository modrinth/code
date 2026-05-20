use std::ptr::NonNull;

use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::CGDisplay;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_graphics::window::{
    CGWindowID, kCGWindowAlpha, kCGWindowBounds, kCGWindowLayer,
    kCGWindowListExcludeDesktopElements,
    kCGWindowListOptionOnScreenAboveWindow, kCGWindowListOptionOnScreenOnly,
    kCGWindowNumber, kCGWindowOwnerName, kCGWindowOwnerPID,
};
use objc2_app_kit::NSWindow;

pub fn is_ads_webview_occluded<R: tauri::Runtime>(
    main_window: &tauri::Window<R>,
    ad_x: i32,
    ad_y: i32,
    ad_width: u32,
    ad_height: u32,
) -> Option<bool> {
    let scale_factor = main_window.scale_factor().ok()?;
    let maybe_ns_window = main_window.ns_window().ok()?;
    let ns_window = NonNull::new(maybe_ns_window)?.cast::<NSWindow>();
    let ns_window = unsafe { ns_window.as_ref() };
    let window_number = ns_window.windowNumber();
    let main_window_id = if window_number > 0 {
        window_number as CGWindowID
    } else {
        return None;
    };

    let window_infos = CGDisplay::window_list_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        None,
    )?;

    let window_infos = unsafe {
        CFArray::<CFDictionary<CFString, CFType>>::wrap_under_get_rule(
            window_infos.as_concrete_TypeRef(),
        )
    };
    let main_window_rect = window_infos
        .iter()
        .find(|window_info| window_id(window_info) == Some(main_window_id))
        .and_then(|window_info| window_rect(&window_info))?;
    let ad_rect = ad_rect_from_main_window(
        &main_window_rect,
        ad_x,
        ad_y,
        ad_width,
        ad_height,
        scale_factor,
    );

    if is_empty_rect(&ad_rect) {
        return None;
    }

    let ad_area = rect_area(&ad_rect);

    if ad_area == 0.0 {
        return None;
    }

    let app_process_id = std::process::id() as i32;
    let windows_above_main = CGDisplay::window_list_info(
        kCGWindowListOptionOnScreenAboveWindow
            | kCGWindowListExcludeDesktopElements,
        Some(main_window_id),
    )?;
    let windows_above_main = unsafe {
        CFArray::<CFDictionary<CFString, CFType>>::wrap_under_get_rule(
            windows_above_main.as_concrete_TypeRef(),
        )
    };
    let mut occluded_area = 0.0;

    for window_info in windows_above_main.iter() {
        if window_id(&window_info) == Some(main_window_id) {
            continue;
        }

        if window_process_id(&window_info) == Some(app_process_id) {
            continue;
        }

        let owner_name = window_owner_name(&window_info);

        if owner_name.as_deref().is_some_and(is_system_window_owner) {
            continue;
        }

        let layer = window_layer(&window_info);

        if layer != Some(0) {
            continue;
        }

        let alpha = window_alpha(&window_info);

        if alpha.is_some_and(|alpha| alpha <= 0.0) {
            continue;
        }

        let Some(rect) = window_rect(&window_info) else {
            continue;
        };

        let Some(intersection) = intersect_rects(&ad_rect, &rect) else {
            continue;
        };

        occluded_area += rect_area(&intersection);
        let occluded_ratio = occluded_area / ad_area;

        if occluded_ratio >= super::ads::OCCLUDED_AREA_THRESHOLD {
            return Some(true);
        }
    }

    Some(false)
}

fn ad_rect_from_main_window(
    main_window_rect: &CGRect,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale_factor: f64,
) -> CGRect {
    let scale_factor = if scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };

    CGRect::new(
        &CGPoint::new(
            main_window_rect.origin.x + x as f64 / scale_factor,
            main_window_rect.origin.y + y as f64 / scale_factor,
        ),
        &CGSize::new(width as f64 / scale_factor, height as f64 / scale_factor),
    )
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

    if is_empty_rect(&rect) {
        None
    } else {
        Some(rect)
    }
}

fn is_empty_rect(rect: &CGRect) -> bool {
    rect.size.width <= 0.0 || rect.size.height <= 0.0
}

fn rect_area(rect: &CGRect) -> f64 {
    if is_empty_rect(rect) {
        return 0.0;
    }

    rect.size.width * rect.size.height
}

fn intersect_rects(a: &CGRect, b: &CGRect) -> Option<CGRect> {
    let left = a.origin.x.max(b.origin.x);
    let top = a.origin.y.max(b.origin.y);
    let right = (a.origin.x + a.size.width).min(b.origin.x + b.size.width);
    let bottom = (a.origin.y + a.size.height).min(b.origin.y + b.size.height);
    let rect = CGRect::new(
        &CGPoint::new(left, top),
        &CGSize::new(right - left, bottom - top),
    );

    if is_empty_rect(&rect) {
        None
    } else {
        Some(rect)
    }
}
