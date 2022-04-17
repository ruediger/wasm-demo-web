use futures::TryFutureExt;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;
use web_sys::{ErrorEvent, MessageEvent, Request, RequestInit, RequestMode, Response};

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");

    body.append_child(&val)?;

    let canvas = document.get_element_by_id("canvas").expect("should have canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let image = web_sys::HtmlImageElement::new()?;
    let image2 = image.clone();
    let context2 = context.clone();
    let draw_image = Closure::wrap(Box::new(move || {
        for x in 0..10 {
            for y in 0..10 {
                context2.draw_image_with_html_image_element(&image2, (image2.width() * x).into(), (image2.height() * y).into()).expect("drawing image failed");
            }
        }
    }) as Box<dyn Fn()>);
    image.set_onload(Some(draw_image.as_ref().unchecked_ref()));
    draw_image.forget();
    image.set_src("webdata/example128.jpg");

    let context2 = context.clone();
    let mousedown = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        console::log_2(&"Mousedown: ".into(), &event.button().into());
        context2.begin_path();
        context2.ellipse(event.offset_x().into(), event.offset_y().into(), 10.0, 10.0, 0.0, 0.0, std::f64::consts::TAU).expect("ellipse failed");
        context2.set_fill_style(&"red".into());
        context2.fill();
    }) as Box<dyn Fn(_)>);
    canvas.set_onmousedown(Some(mousedown.as_ref().unchecked_ref()));
    mousedown.forget();

    chat_init()
}

#[wasm_bindgen]
pub fn scroll_test() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let div = document.get_element_by_id("canvas-div").expect("should have canvas-div element");
    div.scroll_to_with_x_and_y(100.0, 100.0);
    console::log_1(&"did it".into());
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

////////////////////////////////////////////////////////////

// TODO: have structs in common lib for server+client!
#[derive(Serialize, Debug)]
struct RegisterRequest {
    name: String,
}

#[derive(Deserialize, Debug)]
struct RegisterResponse {
    uuid: String,  // Uuid
}

#[wasm_bindgen]
pub fn chat_init() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    // Connect actions
    let registration_div = document.get_element_by_id("registration").expect(
        "should have `registration` div element");
    let registration_div: web_sys::HtmlDivElement = registration_div
        .dyn_into::<web_sys::HtmlDivElement>()
        .expect("div element `registration`");

    let name_input = document.get_element_by_id("name").expect("should have `name` input element");
    let name_input: web_sys::HtmlInputElement = name_input
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input element `name`");

    let connect_input = document.get_element_by_id("connect").expect("should have `connect` input element");
    let connect_input: web_sys::HtmlInputElement = connect_input
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input element `connect`");

    let live_div = document.get_element_by_id("live").expect("should have `live` div element");
    let live_div: web_sys::HtmlDivElement = live_div
        .dyn_into::<web_sys::HtmlDivElement>()
        .expect("div element `live`");

    let chatmessages_area = document.get_element_by_id("chatmessages").expect(
        "should have `chatmessages` textarea element");
    let chatmessages_area: web_sys::HtmlTextAreaElement = chatmessages_area
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .expect("textarea element `chatmessages`");

    let message_input = document.get_element_by_id("message").expect("should have `message` input element");
    let message_input: web_sys::HtmlInputElement = message_input
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input element `message`");

    let send_button = document.get_element_by_id("send").expect("should have `send` input element");
    let send_button: web_sys::HtmlInputElement = send_button
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input element `send`");

    let connect_action = Closure::wrap(Box::new(move || {
        console::log_2(&"Action action action: ".into(), &name_input.value().into());

        if name_input.value() == "" {
            // TODO: validate name (also on server)
            return;
        }

        let fut = register_chat(name_input.value())
            .map_ok(|uuid: String| {
                console::log_2(&"UUID: ".into(), &JsValue::from(&uuid));
                uuid
            });
        wasm_bindgen_futures::spawn_local(async move {
            match fut.await {
                Err(e) => console::log_2(&"Failed to register: ".into(), &e),
                Ok(uuid) => { connect_chat(uuid); },
            };
        });


        live_div.style().set_property("display", "block").expect("display block of live div failed");
        registration_div.style().set_property("display", "none").expect("display none of registration div failed");
    }) as Box<dyn Fn()>);
    connect_input.set_onclick(Some(connect_action.as_ref().unchecked_ref()));
    connect_action.forget();

    Ok(())
}

pub fn register_chat(name: String) -> impl futures::Future<Output = std::result::Result<String, JsValue>> {
    // TODO: too many expect. Better error handling!

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let register_request = RegisterRequest {
        name: name,
    };
    let register_request = serde_json::to_string(&register_request).expect("failed conversion from RegisterRequest to JSON");
    let register_request = JsValue::from(register_request);
    opts.body(Some(&register_request));

    let request = Request::new_with_str_and_init(&"/register", &opts).expect(
        "request to be created");
    request.headers().set("Content-Type", "application/json; charset=utf-8").expect("content-type");

    let window = web_sys::window().expect("no global `window` exists");
    JsFuture::from(window.fetch_with_request(&request)).map_ok(|resp_value| {
        // `resp_value` is a `Response` object.
        let resp: Response = resp_value.dyn_into().expect("resp_value should be a Response object");
        resp.json().expect("response to be json")
    }).and_then(|json_value: Promise| {
        JsFuture::from(json_value)
    }).map_ok(|json| {
        let register_response : RegisterResponse = json.into_serde().expect(
            "expected RegisterResponse response type");
        register_response.uuid
    })
}

pub fn connect_chat(uuid: String) -> std::result::Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let chatmessages_area = document.get_element_by_id("chatmessages").expect(
        "should have `chatmessages` textarea element");
    let chatmessages_area: web_sys::HtmlTextAreaElement = chatmessages_area
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .expect("textarea element `chatmessages`");

    let ws = web_sys::WebSocket::new(&format!("ws://localhost:3030/chat/{}", uuid).as_str())?;

    // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            console::log_2(&"message event, received Text: ".into(), &txt);
            let t = chatmessages_area.inner_text();
            let t = String::from(t) + &String::from(txt);
            chatmessages_area.set_inner_text(&t);
        } else {
            console::log_2(&"message event, received Unknown: ".into(), &e.data());
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        console::log_2(&"error event: ".into(), &e);
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        console::log_1(&"socket opened".into());
        match cloned_ws.send_with_str("ping") {
            Ok(_) => console::log_1(&"message successfully sent".into()),
            Err(err) => console::log_2(&"error sending message: {:?}".into(), &err),
        }
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}
