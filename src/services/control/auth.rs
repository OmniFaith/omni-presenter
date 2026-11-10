use super::WebControl;
use serde_json::json;
use std::io::Read;

fn token(request: &tiny_http::Request) -> String {
    request
        .headers()
        .iter()
        .find_map(|header| {
            if header.field.equiv("Authorization") {
                header
                    .value
                    .as_str()
                    .strip_prefix("Bearer ")
                    .map(str::to_owned)
            } else if header.field.equiv("Cookie") {
                header.value.as_str().split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("omni_control=")
                        .or_else(|| part.strip_prefix("omni_chat="))
                        .map(str::to_owned)
                })
            } else {
                None
            }
        })
        .unwrap_or_default()
}

pub(super) fn authenticated(request: &tiny_http::Request, control: &WebControl) -> bool {
    control
        .auth
        .read()
        .unwrap()
        .as_ref()
        .is_some_and(|service| service.heartbeat(&token(request)).is_ok())
}

/// Returns the request only when it is an authenticated application endpoint.
pub(super) fn gate(
    mut request: tiny_http::Request,
    control: &WebControl,
) -> Option<tiny_http::Request> {
    let path = request.url().split('?').next().unwrap_or("").to_owned();
    let language = crate::services::i18n::language(&request, Some(control.language()));
    let method = request.method().as_str().to_owned();
    let service = control.auth.read().unwrap().clone();
    let mut cookies = Vec::new();
    let result = match (method.as_str(), path.as_str()) {
        ("GET", "/api/control/profiles") => Some(
            service
                .as_ref()
                .map(|service| json!(service.profiles()))
                .ok_or("services-access-unavailable".to_owned()),
        ),
        ("POST", "/api/control/login") => Some((|| {
            let mut body = Vec::new();
            request
                .as_reader()
                .take(8193)
                .read_to_end(&mut body)
                .map_err(|e| e.to_string())?;
            if body.len() > 8192 {
                return Err("services-login-data-is-too-large".into());
            }
            let body: serde_json::Value =
                serde_json::from_slice(&body).map_err(|e| e.to_string())?;
            let profile_id = body["profile_id"].as_str().unwrap_or("").to_owned();
            let token = service.as_ref().ok_or("services-access-unavailable")?.login(
                &profile_id,
                body["password"].as_str().unwrap_or(""),
            )?;
            cookies.push(format!(
                "omni_control={token}; Path=/; HttpOnly; SameSite=Lax"
            ));
            cookies.push(format!(
                "omni_chat={token}; Path=/; HttpOnly; SameSite=Lax"
            ));
            Ok(json!({"ok":true,"token":token,"profile_id":profile_id}))
        })()),
        ("POST", "/api/control/logout") => {
            if let Some(service) = &service {
                service.logout(&token(&request));
            }
            cookies.push("omni_control=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".into());
            cookies.push("omni_chat=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".into());
            Some(Ok(json!({"ok":true})))
        }
        _ => None,
    };
    if let Some(result) = result {
        let (status, value) = match result {
            Ok(value) => (200, value),
            Err(error) => (401, json!({"error":language.message(&error)})),
        };
        let mut response =
            tiny_http::Response::from_string(value.to_string()).with_status_code(status);
        for (key, value) in [
            ("Content-Type", "application/json"),
            ("Cache-Control", "no-store"),
        ] {
            response.add_header(tiny_http::Header::from_bytes(key, value).unwrap());
        }
        for cookie in cookies {
            response.add_header(tiny_http::Header::from_bytes("Set-Cookie", cookie).unwrap());
        }
        let _ = request.respond(response);
        return None;
    }
    if authenticated(&request, control) {
        return Some(request);
    }
    super::http::reply(
        request,
        401,
        "application/json",
        json!({"error":language.text("control-enter-profile-and-password")}).to_string().into_bytes(),
    );
    None
}
