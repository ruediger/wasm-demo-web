use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
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

    Ok(())
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
