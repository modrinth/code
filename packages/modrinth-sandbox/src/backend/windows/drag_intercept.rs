//! Intercepts files dragged onto windows of a sandboxed process, exposing them
//! to the sandbox before forwarding the drop.
//!
//! A drag is detected by the creation of the shell's `SysDragImage` window, and
//! lasts until all mouse buttons are released. The window can't be used to
//! detect the end, since it's destroyed when hovering over a drop target.

use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, ensure};
use tracing::{debug, error, warn};
use windows::{
    Win32::{
        Foundation::{
            COLORREF, DV_E_TYMED, ERROR_CLASS_ALREADY_EXISTS, GlobalFree,
            HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, POINT, POINTL, RECT, S_OK,
            WPARAM,
        },
        Graphics::Gdi::{
            ANTIALIASED_QUALITY, BeginPaint, CLIP_DEFAULT_PRECIS, CreateFontW,
            DEFAULT_CHARSET, DEFAULT_PITCH, DRAW_TEXT_FORMAT, DT_CALCRECT,
            DT_CENTER, DT_NOPREFIX, DT_WORDBREAK, DeleteObject, DrawTextW,
            EndPaint, FW_BOLD, GetMonitorInfoW, GetStockObject, HBRUSH,
            HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
            OUT_DEFAULT_PRECIS, PAINTSTRUCT, ScreenToClient, SelectObject,
            SetBkMode, SetTextColor, TRANSPARENT, WHITE_BRUSH,
        },
        Security::{
            GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation,
            SECURITY_MAX_SID_SIZE, TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
            TokenIntegrityLevel,
        },
        System::{
            Com::{
                CoImpersonateClient, CoRevertToSelf, DVASPECT_CONTENT,
                FORMATETC, IDataObject, TYMED_HGLOBAL,
            },
            LibraryLoader::GetModuleHandleW,
            Memory::{GHND, GlobalAlloc, GlobalLock, GlobalUnlock},
            Ole::{
                CF_HDROP, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
                IDropTarget, IDropTarget_Impl, OleInitialize, OleUninitialize,
                RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop,
            },
            SystemServices::{
                MODIFIERKEYS_FLAGS, SECURITY_MANDATORY_MEDIUM_RID,
            },
            Threading::{
                GetCurrentThread, GetCurrentThreadId, OpenProcess,
                OpenProcessToken, OpenThreadToken,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
                SetThreadDpiAwarenessContext,
            },
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
                VK_XBUTTON1, VK_XBUTTON2,
            },
            Shell::{DROPFILES, DragQueryFileW, HDROP},
            WindowsAndMessaging::{
                CHILDID_SELF, CreateWindowExW, DefWindowProcW, DestroyWindow,
                DispatchMessageW, EVENT_OBJECT_CREATE, GA_ROOT, GetAncestor,
                GetClassNameW, GetClientRect, GetCursorPos, GetMessageW,
                GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
                HTTRANSPARENT, KillTimer, LWA_ALPHA, MSG, OBJID_WINDOW,
                PM_NOREMOVE, PeekMessageW, PostMessageW, PostThreadMessageW,
                RegisterClassW, SW_SHOWNOACTIVATE, SetLayeredWindowAttributes,
                SetTimer, ShowWindow, WINEVENT_OUTOFCONTEXT, WM_APP,
                WM_DROPFILES, WM_NCHITTEST, WM_PAINT, WM_QUIT, WM_TIMER,
                WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
                WS_EX_TOPMOST, WS_POPUP, WindowFromPoint,
            },
        },
    },
    core::{BOOL, HRESULT, HSTRING, Owned, PCWSTR, Ref, implement, w},
};

use super::WindowsSandboxHandle;
use crate::{
    backend::{ExposedPath, SandboxHandleOp},
    path_access::{PathPermission, has_executable_extension},
};

const DRAG_IMAGE_CLASS: &str = "SysDragImage";

const POLL_INTERVAL_MS: u32 = 15;

/// How long to wait for a drop after the mouse buttons are released.
const DROP_TIMEOUT: Duration = Duration::from_secs(1);

/// The `wParam` is the ID of the process that created the drag image.
const WM_DRAG_IMAGE_CREATED: u32 = WM_APP;

const OVERLAY_CLASS: PCWSTR = w!("ModrinthSandboxDragOverlay");
const OVERLAY_OPACITY: u8 = 128;
const OVERLAY_HIDDEN_OPACITY: u8 = 1;
const OVERLAY_FONT: PCWSTR = w!("Segoe UI");
const OVERLAY_FONT_SCALE: i32 = 1920 / 48;

const DRAG_CONTENT_STATE_UNKNOWN: u8 = 0;
const DRAG_CONTENT_STATE_FILES: u8 = 1;
const DRAG_CONTENT_STATE_OTHER: u8 = 2;

const DROPPED_FILES_FORMAT: FORMATETC = FORMATETC {
    cfFormat: CF_HDROP.0,
    ptd: std::ptr::null_mut(),
    dwAspect: DVASPECT_CONTENT.0,
    lindex: -1,
    tymed: TYMED_HGLOBAL.0 as u32,
};

/// Stops the drag intercept thread when dropped, without waiting for it.
#[derive(Debug)]
pub struct DragInterceptHandle {
    thread_id: u32,
    /// The system can't reuse `thread_id` while this is open.
    thread: JoinHandle<()>,
}

impl DragInterceptHandle {
    pub fn start(
        process_id: u32,
        name: &str,
        sandbox: WindowsSandboxHandle,
    ) -> Result<Self> {
        let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);

        let overlay_text =
            HSTRING::from(format!("Drop to grant {name} access to this file"));

        let thread = std::thread::Builder::new()
            .name("Modrinth Sandbox Drag Intercept".to_string())
            .spawn(move || {
                let (_ole, _event_hook) = match setup_intercept_thread() {
                    Ok(setup) => setup,
                    Err(err) => {
                        _ = ready_sender.send(Err(err));
                        return;
                    }
                };
                _ = ready_sender.send(Ok(unsafe { GetCurrentThreadId() }));

                DragIntercept {
                    process_id,
                    sandbox,
                    overlay_text,
                    tracking: None,
                }
                .run_message_loop();
                debug!(process_id, "drag intercept thread stopped");
            })
            .context("spawning drag intercept thread")?;

        let thread_id = ready_receiver
            .recv()
            .context("drag intercept thread exited during setup")??;
        debug!(process_id, "drag intercept thread started");

        Ok(Self { thread_id, thread })
    }
}

impl Drop for DragInterceptHandle {
    fn drop(&mut self) {
        if self.thread.is_finished() {
            return;
        }
        if let Err(err) = unsafe {
            PostThreadMessageW(
                self.thread_id,
                WM_QUIT,
                WPARAM::default(),
                LPARAM::default(),
            )
        } {
            error!("unable to stop drag intercept thread: {err}");
        }
    }
}

struct OleGuard;

impl OleGuard {
    fn initialize() -> windows::core::Result<Self> {
        unsafe { OleInitialize(None)? };
        Ok(Self)
    }
}

impl Drop for OleGuard {
    fn drop(&mut self) {
        unsafe { OleUninitialize() };
    }
}

struct WinEventHook(HWINEVENTHOOK);

impl Drop for WinEventHook {
    fn drop(&mut self) {
        if !unsafe { UnhookWinEvent(self.0) }.as_bool() {
            error!("unable to remove drag image event hook");
        }
    }
}

struct ThreadTimer(usize);

impl ThreadTimer {
    fn start(interval_ms: u32) -> windows::core::Result<Self> {
        match unsafe { SetTimer(None, 0, interval_ms, None) } {
            0 => Err(windows::core::Error::from_win32()),
            id => Ok(Self(id)),
        }
    }
}

impl Drop for ThreadTimer {
    fn drop(&mut self) {
        if let Err(err) = unsafe { KillTimer(None, self.0) } {
            error!("unable to remove drag timer: {err}");
        }
    }
}

struct DragIntercept {
    process_id: u32,
    sandbox: WindowsSandboxHandle,
    overlay_text: HSTRING,
    tracking: Option<DragTracking>,
}

struct DragTracking {
    timer: ThreadTimer,
    released_at: Option<Instant>,
    overlay: Option<DragOverlay>,
    content_state: Arc<AtomicU8>,
}

impl DragIntercept {
    fn run_message_loop(mut self) {
        let mut message = MSG::default();
        loop {
            match unsafe { GetMessageW(&raw mut message, None, 0, 0) }.0 {
                0 => return,
                -1 => {
                    error!(
                        "drag intercept message loop failed: {}",
                        windows::core::Error::from_win32()
                    );
                    return;
                }
                _ => {}
            }

            let is_thread_message = message.hwnd.is_invalid();
            match message.message {
                WM_DRAG_IMAGE_CREATED if is_thread_message => {
                    self.start_tracking(message.wParam.0 as u32);
                }
                WM_TIMER
                    if is_thread_message
                        && self.tracking.as_ref().is_some_and(|tracking| {
                            tracking.timer.0 == message.wParam.0
                        }) =>
                {
                    self.update_tracking();
                }
                _ => _ = unsafe { DispatchMessageW(&raw const message) },
            }
        }
    }

    fn start_tracking(&mut self, drag_image_owner: u32) {
        if self.tracking.is_some() || !is_any_mouse_button_down() {
            return;
        }
        if let Err(err) = ensure_process_medium_integrity(drag_image_owner) {
            debug!("ignoring drag image from untrusted process: {err:#}");
            return;
        }

        match ThreadTimer::start(POLL_INTERVAL_MS) {
            Ok(timer) => {
                debug!("drag started");
                self.tracking = Some(DragTracking {
                    timer,
                    released_at: None,
                    overlay: None,
                    content_state: Arc::new(AtomicU8::new(
                        DRAG_CONTENT_STATE_UNKNOWN,
                    )),
                });
                self.update_tracking();
            }
            Err(err) => error!("unable to track drag: {err}"),
        }
    }

    fn stop_tracking(&mut self) {
        if self.tracking.take().is_some() {
            debug!("drag finished");
        }
    }

    fn update_tracking(&mut self) {
        let Some(tracking) = &mut self.tracking else {
            return;
        };

        if tracking.released_at.is_none() && is_any_mouse_button_down() {
            // Keep tracking until release, since another drag image can be
            // created during the same drag
            if tracking.content_state.load(Ordering::SeqCst)
                == DRAG_CONTENT_STATE_OTHER
            {
                if tracking.overlay.take().is_some() {
                    debug!("ignoring drag without files");
                }
                return;
            }
            if let Err(err) = tracking.update_overlay(
                self.process_id,
                &self.sandbox,
                &self.overlay_text,
            ) {
                error!("unable to show drag overlay: {err:#}");
                self.stop_tracking();
            }
        } else {
            let released_at =
                tracking.released_at.get_or_insert_with(Instant::now);
            let expects_drop = tracking
                .overlay
                .as_ref()
                .is_some_and(DragOverlay::is_dragged_over);
            if !expects_drop || released_at.elapsed() >= DROP_TIMEOUT {
                self.stop_tracking();
            }
        }
    }
}

impl DragTracking {
    fn update_overlay(
        &mut self,
        process_id: u32,
        sandbox: &WindowsSandboxHandle,
        overlay_text: &HSTRING,
    ) -> Result<()> {
        let mut cursor = POINT::default();
        unsafe { GetCursorPos(&raw mut cursor) }
            .context("getting cursor position")?;
        let monitor =
            unsafe { MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST) };
        let hovered = top_level_window_at(cursor)
            .filter(|window| get_window_process_id(*window) == process_id)
            .map(|window| (window, monitor));
        if hovered == self.overlay.as_ref().map(DragOverlay::hovered) {
            return Ok(());
        }

        debug!(process_id, ?hovered, "hovered sandboxed window changed");
        self.overlay = hovered
            .map(|(window, monitor)| {
                DragOverlay::show(
                    monitor,
                    overlay_text,
                    self.content_state.clone(),
                    DropDestination {
                        sandbox: sandbox.clone(),
                        process_id,
                        window,
                    },
                )
            })
            .transpose()?;
        Ok(())
    }
}

fn setup_intercept_thread() -> Result<(OleGuard, WinEventHook)> {
    let ole = OleGuard::initialize().context("initializing OLE")?;

    unsafe {
        // Use physical pixels for all coordinates
        ensure!(
            !SetThreadDpiAwarenessContext(
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
            )
            .is_invalid(),
            "unable to make drag intercept thread DPI aware"
        );

        // Create the message queue, so that `WM_QUIT` can be posted to it
        let mut message = MSG::default();
        _ = PeekMessageW(&raw mut message, None, 0, 0, PM_NOREMOVE);

        let hook = SetWinEventHook(
            EVENT_OBJECT_CREATE,
            EVENT_OBJECT_CREATE,
            None,
            Some(on_window_created),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        if hook.is_invalid() {
            return Err(anyhow!("unable to install drag image event hook"));
        }
        Ok((ole, WinEventHook(hook)))
    }
}

unsafe extern "system" fn on_window_created(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    object_id: i32,
    child_id: i32,
    _thread_id: u32,
    _time: u32,
) {
    if object_id != OBJID_WINDOW.0
        || child_id != CHILDID_SELF as i32
        || !is_drag_image_window(window)
    {
        return;
    }
    if let Err(err) = unsafe {
        PostMessageW(
            None,
            WM_DRAG_IMAGE_CREATED,
            WPARAM(get_window_process_id(window) as usize),
            LPARAM::default(),
        )
    } {
        error!("unable to post drag image: {err}");
    }
}

fn is_any_mouse_button_down() -> bool {
    [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2]
        .into_iter()
        .any(|button| unsafe { GetAsyncKeyState(button.0.into()) } < 0)
}

fn is_drag_image_window(window: HWND) -> bool {
    let mut buffer = [0; DRAG_IMAGE_CLASS.len() + 2];
    let length = unsafe { GetClassNameW(window, &mut buffer) };
    buffer[..length as usize]
        .iter()
        .copied()
        .eq(DRAG_IMAGE_CLASS.encode_utf16())
}

/// Must be dropped on the thread that created it.
#[derive(Debug)]
struct DragOverlay {
    window: HWND,
    hovered: (HWND, HMONITOR),
    dragged_over: Arc<AtomicBool>,
}

impl DragOverlay {
    fn show(
        monitor: HMONITOR,
        overlay_text: &HSTRING,
        content_state: Arc<AtomicU8>,
        destination: DropDestination,
    ) -> Result<Self> {
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        ensure!(
            unsafe { GetMonitorInfoW(monitor, &raw mut info) }.as_bool(),
            "unable to get bounds of monitor"
        );
        let bounds = info.rcMonitor;

        unsafe {
            let instance = GetModuleHandleW(None)?.into();
            let class = WNDCLASSW {
                lpfnWndProc: Some(overlay_window_proc),
                hInstance: instance,
                hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
                lpszClassName: OVERLAY_CLASS,
                ..Default::default()
            };
            if RegisterClassW(&raw const class) == 0 {
                let err = windows::core::Error::from_win32();
                if err.code()
                    != HRESULT::from_win32(ERROR_CLASS_ALREADY_EXISTS.0)
                {
                    return Err(err).context("registering overlay class");
                }
            }

            let window = CreateWindowExW(
                WS_EX_LAYERED
                    | WS_EX_NOACTIVATE
                    | WS_EX_TOOLWINDOW
                    | WS_EX_TOPMOST,
                OVERLAY_CLASS,
                overlay_text,
                WS_POPUP,
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
                None,
                None,
                Some(instance),
                None,
            )
            .context("creating overlay window")?;
            let dragged_over = Arc::<AtomicBool>::default();
            let overlay = Self {
                window,
                hovered: (destination.window, monitor),
                dragged_over: dragged_over.clone(),
            };

            set_overlay_opacity(window, OVERLAY_HIDDEN_OPACITY)
                .context("setting overlay opacity")?;

            let target: IDropTarget = DropTarget {
                overlay_window: window,
                destination,
                content_state,
                dragged_over,
            }
            .into();
            RegisterDragDrop(window, &target)
                .context("registering overlay as a drop target")?;

            _ = ShowWindow(window, SW_SHOWNOACTIVATE);

            Ok(overlay)
        }
    }

    fn hovered(&self) -> (HWND, HMONITOR) {
        self.hovered
    }

    fn is_dragged_over(&self) -> bool {
        self.dragged_over.load(Ordering::SeqCst)
    }
}

impl Drop for DragOverlay {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = RevokeDragDrop(self.window) {
                debug!("unable to revoke drag overlay drop target: {err}");
            }
            if let Err(err) = DestroyWindow(self.window) {
                error!("unable to destroy drag overlay: {err}");
            }
        }
    }
}

/// Not agile, so that COM always calls it on the thread of the overlay.
#[implement(IDropTarget, Agile = false)]
struct DropTarget {
    overlay_window: HWND,
    destination: DropDestination,
    content_state: Arc<AtomicU8>,
    dragged_over: Arc<AtomicBool>,
}

impl DropTarget {
    fn has_files(&self) -> bool {
        self.content_state.load(Ordering::SeqCst) == DRAG_CONTENT_STATE_FILES
    }

    /// Only allows copying, as a move may cause the source to delete the files.
    fn choose_drop_effect(
        &self,
        point: &POINTL,
        allowed: DROPEFFECT,
    ) -> DROPEFFECT {
        let inside = top_level_window_at(pointl_to_point(point))
            == Some(self.destination.window);
        if inside && self.has_files() {
            allowed & DROPEFFECT_COPY
        } else {
            DROPEFFECT_NONE
        }
    }
}

impl IDropTarget_Impl for DropTarget_Impl {
    fn DragEnter(
        &self,
        data: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        if let Err(err) = ensure_com_caller_medium_integrity() {
            warn!("ignoring drag from untrusted source: {err:#}");
            unsafe { *effect = DROPEFFECT_NONE };
            return Ok(());
        }

        self.dragged_over.store(true, Ordering::SeqCst);
        if self.content_state.load(Ordering::SeqCst)
            == DRAG_CONTENT_STATE_UNKNOWN
        {
            let format = DROPPED_FILES_FORMAT;
            let has_files = data.as_ref().is_some_and(|data| unsafe {
                data.QueryGetData(&raw const format) == S_OK
            });
            let content_state = if has_files {
                DRAG_CONTENT_STATE_FILES
            } else {
                DRAG_CONTENT_STATE_OTHER
            };
            self.content_state.store(content_state, Ordering::SeqCst);
        }
        if self.has_files()
            && let Err(err) =
                set_overlay_opacity(self.overlay_window, OVERLAY_OPACITY)
        {
            error!("unable to reveal drag overlay: {err}");
        }
        unsafe { *effect = self.choose_drop_effect(point, *effect) };
        Ok(())
    }

    fn DragOver(
        &self,
        _keys: MODIFIERKEYS_FLAGS,
        point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        unsafe { *effect = self.choose_drop_effect(point, *effect) };
        Ok(())
    }

    fn DragLeave(&self) -> windows::core::Result<()> {
        if let Err(err) = ensure_com_caller_medium_integrity() {
            warn!("ignoring drag leave from untrusted source: {err:#}");
            return Ok(());
        }

        self.dragged_over.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn Drop(
        &self,
        data: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        if let Err(err) = ensure_com_caller_medium_integrity() {
            warn!("rejecting drop from untrusted source: {err:#}");
            unsafe { *effect = DROPEFFECT_NONE };
            return Ok(());
        }

        self.dragged_over.store(false, Ordering::SeqCst);

        let chosen = self.choose_drop_effect(point, unsafe { *effect });
        unsafe { *effect = DROPEFFECT_NONE };
        if chosen == DROPEFFECT_NONE {
            return Ok(());
        }

        let files = read_dropped_files(data.ok()?).inspect_err(|err| {
            error!("unable to read dropped files: {err}");
        })?;
        debug!(?files, "files dropped on sandboxed window");
        self.destination
            .expose_and_forward(files, pointl_to_point(point));

        unsafe { *effect = chosen };
        Ok(())
    }
}

fn pointl_to_point(point: &POINTL) -> POINT {
    POINT {
        x: point.x,
        y: point.y,
    }
}

fn read_dropped_files(
    data: &IDataObject,
) -> windows::core::Result<Vec<PathBuf>> {
    let format = DROPPED_FILES_FORMAT;
    let medium = scopeguard::guard(
        unsafe { data.GetData(&raw const format)? },
        |mut medium| unsafe { ReleaseStgMedium(&raw mut medium) },
    );
    if medium.tymed != format.tymed {
        return Err(DV_E_TYMED.into());
    }

    unsafe {
        let drop = HDROP(medium.u.hGlobal.0);
        let count = DragQueryFileW(drop, u32::MAX, None);
        Ok((0..count)
            .map(|index| {
                let length = DragQueryFileW(drop, index, None);
                let mut buffer = vec![0; length as usize + 1];
                let copied =
                    DragQueryFileW(drop, index, Some(buffer.as_mut_slice()));
                OsString::from_wide(&buffer[..copied as usize]).into()
            })
            .collect())
    }
}

/// Ignores our overlay, as hit tests from its own thread respect
/// [`HTTRANSPARENT`].
fn top_level_window_at(point: POINT) -> Option<HWND> {
    let window = unsafe { WindowFromPoint(point) };
    if window.is_invalid() {
        return None;
    }
    let root = unsafe { GetAncestor(window, GA_ROOT) };
    (!root.is_invalid()).then_some(root)
}

unsafe extern "system" fn overlay_window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_PAINT => {
            paint_overlay(window);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn set_overlay_opacity(window: HWND, opacity: u8) -> windows::core::Result<()> {
    unsafe {
        SetLayeredWindowAttributes(window, COLORREF(0), opacity, LWA_ALPHA)
    }
}

fn paint_overlay(window: HWND) {
    const FORMAT: DRAW_TEXT_FORMAT =
        DRAW_TEXT_FORMAT(DT_CENTER.0 | DT_NOPREFIX.0 | DT_WORDBREAK.0);

    unsafe {
        let mut text = vec![0; GetWindowTextLengthW(window) as usize + 1];
        let length = GetWindowTextW(window, &mut text) as usize;
        let text = &mut text[..length];

        let mut paint = PAINTSTRUCT::default();
        let dc = BeginPaint(window, &raw mut paint);

        let mut bounds = RECT::default();
        _ = GetClientRect(window, &raw mut bounds);

        let font = CreateFontW(
            -bounds.right / OVERLAY_FONT_SCALE,
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            u32::from(DEFAULT_PITCH.0),
            OVERLAY_FONT,
        );
        let previous_font = SelectObject(dc, font.into());
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(0));

        let mut measured = bounds;
        let height =
            DrawTextW(dc, text, &raw mut measured, FORMAT | DT_CALCRECT);
        bounds.top = (bounds.bottom - height) / 2;
        DrawTextW(dc, text, &raw mut bounds, FORMAT);

        SelectObject(dc, previous_font);
        _ = DeleteObject(font.into());
        _ = EndPaint(window, &raw const paint);
    }
}

#[derive(Clone)]
struct DropDestination {
    sandbox: WindowsSandboxHandle,
    process_id: u32,
    window: HWND,
}

impl DropDestination {
    /// Runs on another thread, as exposing files can take a while and would block
    /// the source of the drop.
    fn expose_and_forward(&self, files: Vec<PathBuf>, mut point: POINT) {
        if !unsafe { ScreenToClient(self.window, &raw mut point) }.as_bool() {
            error!("unable to convert drop position to window coordinates");
            return;
        }

        let sandbox = self.sandbox.clone();
        let process_id = self.process_id;
        let window = self.window.0 as usize;
        let thread = std::thread::Builder::new()
            .name("Modrinth Sandbox Drop Forward".to_string())
            .spawn(move || {
                let files = expose_files(&sandbox, files);
                if files.is_empty() {
                    return;
                }
                let window = HWND(window as *mut _);
                match post_drop_message(process_id, window, &files, point) {
                    Ok(()) => {
                        debug!(process_id, ?files, "forwarded dropped files");
                    }
                    Err(err) => {
                        error!("unable to forward dropped files: {err:#}");
                    }
                }
            });
        if let Err(err) = thread {
            error!("unable to spawn drop forward thread: {err}");
        }
    }
}

fn get_window_process_id(window: HWND) -> u32 {
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, Some(&raw mut process_id)) };
    process_id
}

fn expose_files(
    sandbox: &WindowsSandboxHandle,
    files: Vec<PathBuf>,
) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter_map(|file| {
            let resolved = file.canonicalize().ok()?;
            match sandbox.expose_path(&file, get_drop_permission(&resolved)) {
                Ok(ExposedPath::Direct) => Some(file),
                Ok(ExposedPath::Proxied(proxied)) => Some(proxied),
                Err(err) => {
                    error!(
                        file = %file.display(),
                        "unable to expose dropped file to sandbox: {err:#}"
                    );
                    None
                }
            }
        })
        .collect()
}

/// Prevents the sandboxed process from modifying anything which could later be
/// executed outside of the sandbox.
fn get_drop_permission(path: &Path) -> PathPermission {
    if path.is_dir() || has_executable_extension(path) {
        PathPermission::ReadOnly
    } else {
        PathPermission::ReadWrite
    }
}

fn post_drop_message(
    process_id: u32,
    window: HWND,
    files: &[PathBuf],
    point: POINT,
) -> Result<()> {
    // The system may have reused the window handle since the drop
    ensure!(
        get_window_process_id(window) == process_id,
        "window no longer belongs to the sandboxed process"
    );

    let drop = create_drop_data(files, point).context("creating drop data")?;
    // The receiver frees the data if this succeeds
    if let Err(err) = unsafe {
        PostMessageW(
            Some(window),
            WM_DROPFILES,
            WPARAM(drop.0 as usize),
            LPARAM::default(),
        )
    } {
        unsafe { _ = GlobalFree(Some(drop)) };
        return Err(err).context("posting drop message");
    }
    Ok(())
}

fn create_drop_data(files: &[PathBuf], point: POINT) -> Result<HGLOBAL> {
    let paths = encode_path_list(files);
    let header = DROPFILES {
        pFiles: size_of::<DROPFILES>() as u32,
        pt: point,
        fNC: BOOL::from(false),
        fWide: BOOL::from(true),
    };

    unsafe {
        let drop = GlobalAlloc(
            GHND,
            size_of::<DROPFILES>() + size_of_val(paths.as_slice()),
        )?;
        let data = GlobalLock(drop).cast::<DROPFILES>();
        if data.is_null() {
            let err = windows::core::Error::from_win32();
            _ = GlobalFree(Some(drop));
            return Err(err.into());
        }
        data.write_unaligned(header);
        std::ptr::copy_nonoverlapping(
            paths.as_ptr().cast::<u8>(),
            data.add(1).cast::<u8>(),
            size_of_val(paths.as_slice()),
        );
        _ = GlobalUnlock(drop);
        Ok(drop)
    }
}

fn encode_path_list(paths: &[PathBuf]) -> Vec<u16> {
    paths
        .iter()
        .flat_map(|path| path.as_os_str().encode_wide().chain([0]))
        .chain([0])
        .collect()
}

fn ensure_process_medium_integrity(process_id: u32) -> Result<()> {
    let token = unsafe {
        let process = Owned::new(
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
                .context("opening process")?,
        );
        let mut token = HANDLE::default();
        OpenProcessToken(*process, TOKEN_QUERY, &raw mut token)
            .context("opening process token")?;
        Owned::new(token)
    };
    ensure_token_medium_integrity(*token)
}

fn ensure_com_caller_medium_integrity() -> Result<()> {
    let token = unsafe {
        CoImpersonateClient().context("impersonating COM caller")?;
        let mut token = HANDLE::default();
        let opened = OpenThreadToken(
            GetCurrentThread(),
            TOKEN_QUERY,
            true,
            &raw mut token,
        );
        let reverted = CoRevertToSelf();
        opened.context("opening token of COM caller")?;
        let token = Owned::new(token);
        reverted.context("reverting impersonation of COM caller")?;
        token
    };
    ensure_token_medium_integrity(*token)
}

/// Rejects sandboxed processes, as AppContainers have a low integrity level.
fn ensure_token_medium_integrity(token: HANDLE) -> Result<()> {
    let integrity_level = get_token_integrity_level(token)?;
    ensure!(
        integrity_level >= SECURITY_MANDATORY_MEDIUM_RID as u32,
        "integrity level {integrity_level:#x} is too low"
    );
    Ok(())
}

fn get_token_integrity_level(token: HANDLE) -> Result<u32> {
    #[repr(C)]
    struct LabelBuffer {
        label: TOKEN_MANDATORY_LABEL,
        _sid: [u8; SECURITY_MAX_SID_SIZE as usize],
    }

    let mut buffer = LabelBuffer {
        label: TOKEN_MANDATORY_LABEL::default(),
        _sid: [0; SECURITY_MAX_SID_SIZE as usize],
    };
    let mut returned = 0;

    unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            Some((&raw mut buffer).cast()),
            size_of::<LabelBuffer>() as u32,
            &raw mut returned,
        )
        .context("querying token integrity level")?;

        let sid = buffer.label.Label.Sid;
        let count = *GetSidSubAuthorityCount(sid);
        ensure!(count > 0, "token integrity level is empty");
        Ok(*GetSidSubAuthority(sid, u32::from(count) - 1))
    }
}
