use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use log::{error, info};
use serde_json::json;

use crate::compliance::validator::validate_deed;
use crate::ledger::deed_event::{DeedEvent};
use crate::ledger::metrics::BioloadMetrics;
use crate::token::mint::mint_church;

use super::types::{
    AutoChurchMintParams, AutoChurchMintResult, AutoChurchValidateParams,
    AutoChurchValidateResult, AutoChurchVisualizeParams, AutoChurchVisualizeResult,
    JsonRpcError, JsonRpcRequest, JsonRpcResponse,
};

/// Start a simple line-delimited JSON-RPC 2.0 TCP server.
/// Each line is a full JSON-RPC request, response is a single line.
pub fn start_rpc_server(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    info!("Auto_Church RPC server listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                error!("RPC accept error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(stream: TcpStream) {
    let peer = stream.peer_addr().ok();
    info!("RPC client connected: {:?}", peer);

    let reader = BufReader::new(stream.try_clone().expect("clone stream"));
    for line in reader.lines() {
        match line {
            Ok(line) if !line.trim().is_empty() => {
                let response_text = dispatch_request(&line);
                if let Err(e) = writeln!(&mut &stream, "{}", response_text) {
                    error!("RPC write error: {}", e);
                    break;
                }
            }
            Ok(_) => {}
            Err(e) => {
                error!("RPC read error: {}", e);
                break;
            }
        }
    }

    info!("RPC client disconnected: {:?}", peer);
}

fn dispatch_request(raw: &str) -> String {
    let parsed: Result<JsonRpcRequest, _> = serde_json::from_str(raw);
    match parsed {
        Ok(req) => {
            let resp = handle_rpc(req);
            serde_json::to_string(&resp).unwrap_or_else(|e| {
                serde_json::to_string(&JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Internal error".to_string(),
                        data: Some(json!({ "serde_error": e.to_string() })),
                    }),
                    id: json!(null),
                })
                .unwrap()
            })
        }
        Err(e) => serde_json::to_string(&JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32700,
                message: "Parse error".to_string(),
                data: Some(json!({ "detail": e.to_string() })),
            }),
            id: json!(null),
        })
        .unwrap(),
    }
}

fn handle_rpc(req: JsonRpcRequest) -> JsonRpcResponse {
    match req.method.as_str() {
        // Auto_Church surface:

        // auto_church.mint_deed
        "auto_church.mint_deed" => {
            let parsed: Result<AutoChurchMintParams, _> =
                serde_json::from_value(req.params.clone());
            match parsed {
                Ok(params) => {
                    let deed = DeedEvent::new(
                        params.prev_hash,
                        params.actor_id,
                        params.target_ids,
                        params.deed_type,
                        params.tags,
                        params.context_json,
                        params.ethics_flags,
                        params.life_harm_flag,
                    );

                    let metrics =
                        BioloadMetrics::new(params.bioload_delta, params.roh, params.decay);

                    if let Err(e) = validate_deed(&deed, metrics.roh, metrics.decay) {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: 1001,
                                message: "Deed validation failed".to_string(),
                                data: Some(json!({ "error": e.to_string() })),
                            }),
                            id: req.id,
                        };
                    }

                    let church_minted = mint_church(&deed, &metrics);

                    let payload = AutoChurchMintResult {
                        deed,
                        metrics,
                        church_minted,
                    };

                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(json!(payload)),
                        error: None,
                        id: req.id,
                    }
                }
                Err(e) => invalid_params(req.id, e.to_string()),
            }
        }

        // auto_church.validate_deed
        "auto_church.validate_deed" => {
            let parsed: Result<AutoChurchValidateParams, _> =
                serde_json::from_value(req.params.clone());
            match parsed {
                Ok(params) => {
                    let res = validate_deed(&params.deed, params.roh, params.decay);
                    let payload = match res {
                        Ok(_) => AutoChurchValidateResult {
                            valid: true,
                            error_message: None,
                        },
                        Err(e) => AutoChurchValidateResult {
                            valid: false,
                            error_message: Some(e.to_string()),
                        },
                    };

                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(json!(payload)),
                        error: None,
                        id: req.id,
                    }
                }
                Err(e) => invalid_params(req.id, e.to_string()),
            }
        }

        // auto_church.xr_visualize_ledger
        "auto_church.xr_visualize_ledger" => {
            let parsed: Result<AutoChurchVisualizeParams, _> =
                serde_json::from_value(req.params.clone());
            match parsed {
                Ok(params) => {
                    // Fire-and-forget visualization: runs in-process and
                    // returns an ACK to the RPC client.
                    let events = params.events;
                    // Bevy App is not serializable; spawn thread for XR-grid launch.
                    std::thread::spawn(move || {
                        let _app = crate::ledger::deed_event::xr_visualize_ledger(&events);
                        // In a real system you would call _app.run().
                    });

                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(json!(AutoChurchVisualizeResult { launched: true })),
                        error: None,
                        id: req.id,
                    }
                }
                Err(e) => invalid_params(req.id, e.to_string()),
            }
        }

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: Some(json!({ "method": req.method })),
            }),
            id: req.id,
        },
    }
}

fn invalid_params(id: serde_json::Value, detail: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": detail })),
        }),
        id,
    }
}
