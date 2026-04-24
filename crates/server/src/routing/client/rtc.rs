use salvo::Writer;
use salvo::prelude::handler;
use palpo_core::client::rtc::transports::*;
use serde_json::Map;

use crate::{AuthArgs, JsonResult, json_ok};

// `GET /_matrix/client/unstable/org.matrix.msc4143/rtc/transports`
#[handler]
pub async fn get_transport_msc4143(_aa: AuthArgs) -> JsonResult<RtcTransportsResBody> {
    let mut transport_info = Map::new();
    transport_info.insert("livekit_service_url".to_string(), serde_json::json!("https://matrix-rtc.pigeonmails.cc"));
    let rtc_transports = RtcTransport::new("livekit".to_string(), transport_info);
    let v = Vec::from([rtc_transports.unwrap()]);

    json_ok(RtcTransportsResBody::new(v))
}