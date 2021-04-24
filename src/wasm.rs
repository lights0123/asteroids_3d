use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::WindowMode;
use crossbeam_channel::{bounded, unbounded, Receiver};
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use winit::platform::web::WindowExtWebSys;

pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(resize_handler.system())
            .add_system_to_stage(CoreStage::PreUpdate, pointer_events.system())
            .insert_resource(WindowDescriptor {
                canvas: Some("#can".to_string()),
                mode: WindowMode::BorderlessFullscreen,
                ..Default::default()
            });
    }
}

fn resize_handler(
    mut bevy_windows: ResMut<Windows>,
    windows: Res<bevy::winit::WinitWindows>,
    mut resize_handler: Local<Option<Receiver<()>>>,
) {
    let handler = match &mut *resize_handler {
        Some(x) => x,
        None => {
            let (sender, receiver) = bounded(1);
            *resize_handler = Some(receiver);
            let observer = awsm_web::dom::resize::ResizeObserver::new_simple(move || {
                let _ = sender.try_send(());
            });
            for window in windows.windows.values() {
                if let Some(parent) = window.canvas().parent_element() {
                    observer.observe(&parent);
                }
            }
            std::mem::forget(observer);
            resize_handler.as_mut().unwrap()
        }
    };
    if handler.try_recv().is_ok() {
        for bevy_window in bevy_windows.iter_mut() {
            if let Some(window) = windows
                .window_id_to_winit
                .get(&bevy_window.id())
                .and_then(|id| windows.windows.get(id))
            {
                if let Some(parent) = window.canvas().parent_element() {
                    bevy_window.set_resolution(
                        parent.client_width() as f32,
                        parent.client_height() as f32,
                    );
                }
            }
        }
    }
}

fn pointer_events(
    windows: Res<bevy::winit::WinitWindows>,
    mut event_handler: Local<Option<Receiver<MouseMotion>>>,
    mut state: EventWriter<MouseMotion>,
) {
    let handler = match &mut *event_handler {
        Some(x) => x,
        None => {
            let (sender, receiver) = unbounded();
            *event_handler = Some(receiver);
            for window in windows.windows.values() {
                let sender = sender.clone();
                gloo_events::EventListener::new(&window.canvas(), "pointermove", move |event| {
                    let event = event.dyn_ref::<web_sys::PointerEvent>().unwrap_throw();
                    if let Err(err) = sender.try_send(MouseMotion {
                        delta: Vec2::new(event.movement_x() as f32, event.movement_y() as f32)
                            * web_sys::window().unwrap().device_pixel_ratio() as f32,
                    }) {
                        tracing::event!(tracing::Level::ERROR, "{:?}", err);
                    }
                })
                .forget();
            }
            event_handler.as_mut().unwrap()
        }
    };
    while let Ok(val) = handler.try_recv() {
        state.send(val);
    }
}

pub fn toggle_grab_cursor(bevy_window: &mut Window, windows: &bevy::winit::WinitWindows) {
    if let Some(window) = windows
        .window_id_to_winit
        .get(&bevy_window.id())
        .and_then(|id| windows.windows.get(id))
    {
        window.canvas().request_pointer_lock();
        bevy_window.set_cursor_lock_mode(true);
        bevy_window.set_cursor_visibility(false);
    }
}

pub fn cursor_locked(bevy_window: &Window, windows: &bevy::winit::WinitWindows) -> bool {
    if let Some(window) = windows
        .window_id_to_winit
        .get(&bevy_window.id())
        .and_then(|id| windows.windows.get(id))
    {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.pointer_lock_element())
            .map_or(false, |locked| {
                &locked == AsRef::<web_sys::Element>::as_ref(&window.canvas())
            })
    } else {
        false
    }
}
