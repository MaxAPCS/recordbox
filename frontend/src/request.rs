use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

pub async fn request(url: &str, method: &str, body: Option<&str>) -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_body_opt_str(body);

    let request = Request::new_with_str_and_init(url, &opts)?;

    request.headers().set("Accept", "application/json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    Ok(json)
}
