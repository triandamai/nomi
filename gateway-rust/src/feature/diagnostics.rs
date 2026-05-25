use crate::AppState;
use axum::{extract::State, Json};
use serde::Serialize;
use serde_json::json;
use tracing::info;

#[derive(Serialize)]
pub struct DiagnosticReport {
    pub status: String,
    pub database: DatabaseDiagnostic,
    pub redis: RedisDiagnostic,
    pub gemini: GeminiDiagnostic,
    pub mqtt: MqttDiagnostic,
}

#[derive(Serialize)]
pub struct DatabaseDiagnostic {
    pub status: String,
    pub total_conversations: i64,
    pub total_messages: i64,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct RedisDiagnostic {
    pub status: String,
    pub ping_response: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct GeminiDiagnostic {
    pub status: String,
    pub model_test_response: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct MqttDiagnostic {
    pub status: String,
    pub host: String,
    pub error: Option<String>,
}

pub async fn handle_diagnostics(
    State(state): State<AppState>,
) -> Json<DiagnosticReport> {
    info!("Running system diagnostics...");

    // 1. Database Diagnostic
    let mut db_status = "healthy".to_string();
    let mut total_conversations = 0;
    let mut total_messages = 0;
    let mut db_error = None;

    match sqlx::query!("SELECT COUNT(*) as count FROM conversations")
        .fetch_one(&state.pool)
        .await
    {
        Ok(res) => {
            total_conversations = res.count.unwrap_or(0);
        }
        Err(e) => {
            db_status = "unhealthy".to_string();
            db_error = Some(format!("Conversations count query failed: {}", e));
        }
    }

    if db_status == "healthy" {
        match sqlx::query!("SELECT COUNT(*) as count FROM messages")
            .fetch_one(&state.pool)
            .await
        {
            Ok(res) => {
                total_messages = res.count.unwrap_or(0);
            }
            Err(e) => {
                db_status = "unhealthy".to_string();
                db_error = Some(format!("Messages count query failed: {}", e));
            }
        }
    }

    // 2. Redis Diagnostic
    let mut redis_status = "healthy".to_string();
    let mut ping_response = None;
    let mut redis_error = None;

    // Use connection verification or set/get check
    let test_key = "nomi:diagnostic:ping";
    match state.redis.set_ex(test_key, "PONG", 10).await {
        Ok(_) => {
            match state.redis.get(test_key).await {
                Ok(val) => {
                    ping_response = val;
                }
                Err(e) => {
                    redis_status = "unhealthy".to_string();
                    redis_error = Some(format!("Redis GET test failed: {}", e));
                }
            }
        }
        Err(e) => {
            redis_status = "unhealthy".to_string();
            redis_error = Some(format!("Redis SET test failed: {}", e));
        }
    }

    // 3. Gemini Diagnostic
    let mut gemini_status = "healthy".to_string();
    let mut model_test_response = None;
    let mut gemini_error = None;

    let gemini_test = state.gemini
        .generate_content()
        .with_user_message("Respond with exactly: 'CONNECTED'.")
        .execute()
        .await;

    match gemini_test {
        Ok(resp) => {
            let txt = resp.text();
            if txt.trim().is_empty() {
                gemini_status = "degraded".to_string();
                gemini_error = Some("Gemini returned successfully but with empty content.".to_string());
            } else {
                model_test_response = Some(txt);
            }
        }
        Err(e) => {
            gemini_status = "unhealthy".to_string();
            gemini_error = Some(format!(
                "Gemini API request failed. This often happens if the Google Generative Language API is blocked in your VPS region or your GEMINI_API_KEY is restricted/invalid. Error: {:?}",
                e
            ));
        }
    }

    // 4. MQTT Diagnostic
    let mut mqtt_status = "healthy".to_string();
    let mut mqtt_error = None;
    let mqtt_host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    // Verify gateway can publish a small test event
    let test_topic = "nomi/diagnostic/ping";
    let test_payload = json!({ "timestamp": chrono::Utc::now().to_rfc3339() });
    match state.mqtt.publish_event(test_topic, &test_payload.to_string(), rumqttc::QoS::AtMostOnce).await {
        Ok(_) => {}
        Err(e) => {
            mqtt_status = "unhealthy".to_string();
            mqtt_error = Some(format!("Failed to queue publish event to MQTT: {:?}", e));
        }
    }

    let overall_status = if db_status == "healthy" && redis_status == "healthy" && gemini_status == "healthy" && mqtt_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    }.to_string();

    Json(DiagnosticReport {
        status: overall_status,
        database: DatabaseDiagnostic {
            status: db_status,
            total_conversations,
            total_messages,
            error: db_error,
        },
        redis: RedisDiagnostic {
            status: redis_status,
            ping_response,
            error: redis_error,
        },
        gemini: GeminiDiagnostic {
            status: gemini_status,
            model_test_response,
            error: gemini_error,
        },
        mqtt: MqttDiagnostic {
            status: mqtt_status,
            host: mqtt_host,
            error: mqtt_error,
        },
    })
}
