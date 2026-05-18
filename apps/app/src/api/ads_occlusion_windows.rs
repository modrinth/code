use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GW_HWNDNEXT, GetAncestor, GetTopWindow, GetWindow, GetWindowRect,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible,
};

const OCCLUDED_AREA_THRESHOLD: f64 = 1.0;

pub fn is_ads_webview_occluded(
    main_hwnd: HWND,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> bool {
    let Some(ad_rect) = ad_rect_in_screen(main_hwnd, x, y, width, height)
    else {
        return false;
    };

    if is_empty_rect(&ad_rect) {
        return false;
    }

    let ad_area = rect_area(&ad_rect);
    if ad_area == 0 {
        return false;
    }

    let mut occluded_area = 0u64;
    let app_root = unsafe { GetAncestor(main_hwnd, GA_ROOT) };
    let app_process_id = std::process::id();
    let mut hwnd = match unsafe { GetTopWindow(None) } {
        Ok(hwnd) => hwnd,
        Err(_) => return false,
    };

    while !hwnd.is_invalid() {
        let window_root = unsafe { GetAncestor(hwnd, GA_ROOT) };

        if window_root == app_root {
            return false;
        }

        if window_process_id(hwnd) == Some(app_process_id) {
            hwnd = match unsafe { GetWindow(hwnd, GW_HWNDNEXT) } {
                Ok(hwnd) => hwnd,
                Err(_) => break,
            };
            continue;
        }

        if window_counts_as_occluder(hwnd)
            && let Some(occluder_rect) = window_rect(hwnd)
            && let Some(intersection) =
                intersect_rects(&ad_rect, &occluder_rect)
        {
            occluded_area =
                occluded_area.saturating_add(rect_area(&intersection));

            if (occluded_area as f64 / ad_area as f64)
                >= OCCLUDED_AREA_THRESHOLD
            {
                return true;
            }
        }

        hwnd = match unsafe { GetWindow(hwnd, GW_HWNDNEXT) } {
            Ok(hwnd) => hwnd,
            Err(_) => break,
        };
    }

    false
}

fn ad_rect_in_screen(
    main_hwnd: HWND,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Option<RECT> {
    let mut origin = POINT { x: 0, y: 0 };

    if !unsafe { ClientToScreen(main_hwnd, &mut origin).as_bool() } {
        return None;
    }

    let left = origin.x.saturating_add(x);
    let top = origin.y.saturating_add(y);
    let right = left.saturating_add(width as i32);
    let bottom = top.saturating_add(height as i32);

    Some(RECT {
        left,
        top,
        right,
        bottom,
    })
}

fn window_counts_as_occluder(hwnd: HWND) -> bool {
    if !unsafe { IsWindowVisible(hwnd).as_bool() } {
        return false;
    }

    if unsafe { IsIconic(hwnd).as_bool() } {
        return false;
    }

    if is_dwm_cloaked(hwnd) {
        return false;
    }

    true
}

fn window_process_id(hwnd: HWND) -> Option<u32> {
    let mut process_id = 0u32;

    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }

    (process_id != 0).then_some(process_id)
}

fn is_dwm_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;

    unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut _,
            std::mem::size_of::<u32>() as u32,
        )
    }
    .is_ok()
        && cloaked != 0
}

fn window_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();

    if unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut RECT as *mut _,
            std::mem::size_of::<RECT>() as u32,
        )
    }
    .is_err()
        && unsafe { GetWindowRect(hwnd, &mut rect) }.is_err()
    {
        return None;
    }

    if is_empty_rect(&rect) {
        return None;
    }

    Some(rect)
}

fn is_empty_rect(rect: &RECT) -> bool {
    rect.right <= rect.left || rect.bottom <= rect.top
}

fn rect_area(rect: &RECT) -> u64 {
    if is_empty_rect(rect) {
        return 0;
    }

    (rect.right - rect.left) as u64 * (rect.bottom - rect.top) as u64
}

fn intersect_rects(a: &RECT, b: &RECT) -> Option<RECT> {
    let rect = RECT {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    };

    (!is_empty_rect(&rect)).then_some(rect)
}
