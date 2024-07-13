use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

use crate::{
    focus::InteractionFocusBundle,
    ui_box::{BlockBundle, SpawnUIBox},
    utils::{Language, LanguageData},
    ErrorEvent,
};

impl ErrorEvent {
    pub fn take_js_error(val: JsValue) -> Self {
        Self(
            val.as_string()
                .unwrap_or("A Javascript error occured".into()),
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum Command {
    SpawnBlock(String),
    RunCode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum Message {
    Code(String),
    LanguageList(Vec<LanguageData>),
    Error(String),
    Diagnostics(String),
    Command(Command),
}

pub struct WS(pub Option<WebSocket>);

impl Default for WS {
    fn default() -> Self {
        let ws = match WebSocket::new("ws://localhost:3000/websocket?type=server") {
            Ok(ws) => ws,
            Err(error) => {
                error!("{error:?}");
                return WS(None);
            }
        };
        WS(Some(ws))
    }
}

impl WS {
    fn send<T: Serialize>(&self, value: &T) -> bool {
        let value = match serde_json::to_string(value) {
            Ok(val) => val,
            Err(error) => {
                error!("{error}");
                String::default()
            }
        };
        match self.0.as_ref() {
            Some(val) => match val.send_with_str(value.as_str()) {
                Ok(_) => {
                    info!("Sent value");
                    true
                }
                Err(error) => {
                    error!("{error:?}");
                    false
                }
            },
            None => {
                info!("There's no websocket connection open right now");
                false
            }
        }
    }
    fn get_socket(&self) -> Option<&WebSocket> {
        self.0.as_ref()
    }
}

#[derive(Debug, Event, Clone)]
pub struct WASMRequest(pub Message);

#[derive(Debug, Serialize, Deserialize)]
pub struct WrappedMessage {
    sender: &'static str,
    message: Message,
}

#[derive(Debug, Resource, Default)]
struct FailedRequest {
    requests: Vec<Message>,
}

#[derive(Debug, Resource, Default)]
pub struct SocketSender(pub Arc<Mutex<Vec<Message>>>);

// #[derive(Debug)]
// pub struct SocketReciever(pub Receiver<Message>);

pub struct WASMPlugin;

impl WASMPlugin {
    fn set_wasm_handles(ws: NonSend<WS>, resource: Res<SocketSender>) {
        let Some(ws) = ws.get_socket() else {
            info!("Websocket is not yet opened");
            return;
        };
        let sender = resource.0.clone();
        let onmessage = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| match e
            .data()
            .dyn_into::<js_sys::JsString>(
        ) {
            Ok(data) => {
                info!("Recieved text");
                let Some(data) = data.as_string() else {
                    error!("Couldn't convert JsString to a Rust String");
                    return;
                };
                let Ok(message) = serde_json::from_str::<Message>(&data) else {
                    error!("Couldn't parse recieved message");
                    return;
                };
                if let Ok(mut array) = sender.lock() {
                    info!("Pushed the message to the array");
                    array.push(message);
                } else {
                    error!(
                        "There was an error sending this message down the channel\nMessage: {:?}",
                        message
                    );
                }
            }
            Err(error) => {
                error!("{error:?}");
            }
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
        let onopen_callback = Closure::<dyn Fn()>::new(move || {
            info!("socket opened");
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    }
    fn handle_wasm_request(
        ws: NonSend<WS>,
        mut request_reader: EventReader<WASMRequest>,
        mut failed_request: ResMut<FailedRequest>,
    ) {
        for event in request_reader.read().map(ToOwned::to_owned) {
            let message = WrappedMessage {
                sender: "server",
                message: event.0,
            };
            if !ws.send(&message) {
                info!("failed to send the message");
                failed_request.requests.push(message.message)
            }
        }
    }

    fn handle_channel(
        reciever: Res<SocketSender>,
        mut event_writer: EventWriter<SpawnUIBox>,
        mut error_writer: EventWriter<ErrorEvent>,
        language: Res<Language>,
    ) {
        if let Ok(mut message) = reciever.0.lock() {
            for message in message.drain(..) {
                #[allow(clippy::single_match)]
                match message {
                    Message::Command(Command::SpawnBlock(block)) => {
                        if let Some(block) = language.get_block(&block) {
                            event_writer.send(SpawnUIBox {
                                bundle: BlockBundle::new(
                                    0.,
                                    0.,
                                    40.,
                                    40.,
                                    InteractionFocusBundle::default(),
                                    block,
                                ),
                                marker: None,
                            });
                        } else {
                            error_writer.send(ErrorEvent(format!("Couldn't spawn block {block}")));
                        }
                    }
                    _ => {}
                }
            }
        }
    } //

    fn resend_failed_requests(
        mut failed_request: ResMut<FailedRequest>,
        mut request_writer: EventWriter<WASMRequest>,
    ) {
        if !failed_request.requests.is_empty() {
            request_writer.send_batch(failed_request.requests.drain(..).map(WASMRequest));
        }
    }
}

impl Plugin for WASMPlugin {
    fn build(&self, app: &mut App) {
        // let (sender, reciever) = channel::<Message>();
        app.init_non_send_resource::<WS>()
            // .insert_non_send_resource(SocketReciever(reciever))
            .insert_resource(SocketSender::default())
            .init_resource::<FailedRequest>()
            .add_event::<WASMRequest>()
            .add_systems(Startup, Self::set_wasm_handles)
            .add_systems(
                Update,
                (
                    Self::set_wasm_handles,
                    apply_deferred,
                    Self::handle_wasm_request,
                    Self::resend_failed_requests.run_if(on_timer(Duration::from_secs(1))),
                    Self::handle_channel,
                )
                    .chain(),
            );
    }
}
