use rand::RngExt;
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;
use crate::AppState;
use crate::common::repository::{channel_repo, pairing_repo};
use crate::feature::{InboundMessage, OutboundMessage};
use crate::prompts::PromptRegistry;

pub async fn process_generate_pairing(
    state: &AppState,
    msg: &InboundMessage,
    user_id: Uuid,
) -> anyhow::Result<()> {
    let conv_id = if msg.is_group {
        sqlx::query!(
            "SELECT c.id
            FROM channel_group as g
            RIGHT JOIN conversations as c ON c.id = g.conversation_id
            WHERE g.external_group_id = $1 AND g.channel = $2",
            msg.conversation_id,
            msg.channel
        )
            .fetch_one(&state.pool)
            .await
            .map_or_else(|_| Uuid::nil(), |result| result.id)
    } else {
        sqlx::query!(
            "SELECT c.id
            FROM channels as g
            RIGHT JOIN conversations as c ON c.id = g.conversation_id
            WHERE g.external_chat_id = $1 AND g.channel_type = $2",
            msg.conversation_id,
            msg.channel
        )
            .fetch_one(&state.pool)
            .await
            .map_or_else(|_| Uuid::nil(), |result| result.id)
    };
    if conv_id.is_nil() {
        info!("Failed get conversation_id");

        let _ = send_message(
            state,
            msg,
            "Whoops! 🏍️💨 Only a registered Nomi user can pair me with a group.".to_string(),
            msg.metadata.clone(),
        )
            .await;
        return Ok(());
    }
    match pairing_repo::create_pairing_code(&state, conv_id, user_id).await {
        Ok(code) => {
            let _ = send_message(
                state,
                msg,
                format!("/pair {}", code.pairing_code),
                msg.metadata.clone(),
            )
                .await;

            let _ = send_message(
                state,
                msg,
                format!(
                    "User the code to conversation you want pair with. \n Expired at:{}",
                    code.expires_at.to_rfc3339()
                ),
                msg.metadata.clone(),
            )
                .await;
            Ok(())
        }
        Err(err) => {
            info!("Failed create pairing code :{}", err);
            let _ = send_message(
                state,
                msg,
                "Whoops! 🏍️💨 Only a registered Nomi user can pair me with a group.".to_string(),
                msg.metadata.clone(),
            )
                .await;
            Ok(())
        }
    }
}
pub async fn process_pairing(
    state: &AppState,
    msg: &InboundMessage,
    text: &str,
    user_id: Uuid,
) -> anyhow::Result<()> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() >= 2 {
        let code = parts[1].to_uppercase();
        if let Some(conv_id) = pairing_repo::validate_pairing_code(&state.pool, &code).await? {
            let display_name = match msg.metadata.clone() {
                None => None,
                Some(meta) => meta
                    .get("display_name")
                    .map_or_else(|| None, |v| Some(v.to_string())),
            };

            pairing_repo::complete_pairing(&state.pool, &code, user_id).await?;
            channel_repo::link_channel(
                &state.pool,
                &msg.channel,
                &msg.sender_id,
                &msg.conversation_id,
                conv_id,
                user_id,
                display_name,
            )
                .await?;

            let _ = state
                .send_to_user(
                    user_id.to_string().as_str(),
                    "pairing_success",
                    json!({
                        "conversation_id": conv_id,
                        "platform": msg.channel,
                        "message": format!("Successfully paired with {}!", msg.channel)
                    }),
                    &OutboundMessage {
                        is_group: msg.is_group,
                        sender_id: msg.sender_id.clone(),
                        conversation_id: msg.conversation_id.clone(),
                        text: "Pairing successful! This conversation is now linked.".to_string(),
                        channel: msg.channel.clone(),
                        video_url: None,
                        image_url: None,
                        audio_url: None,
                        doc_url: None,
                        sticker_url: None,
                        metadata: msg.metadata.clone(),
                    },
                )
                .await;

            return Ok(());
        }
    }
    Ok(())
}

pub async fn process_register(state: &AppState, msg: &InboundMessage) -> anyhow::Result<()> {
    info!(
        "start registering from channel {} sender_id {}",
        msg.channel, msg.sender_id
    );

    if msg.is_group {
        return process_group_registration(state, msg).await;
    }
    let channel_exists = sqlx::query!("SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",msg.channel,msg.conversation_id)
        .fetch_optional(&state.pool)
        .await;
    if let Err(err) = channel_exists {
        info!("failed register because error getting information: {}", err);
        let _ = send_message(
            state,
            msg,
            PromptRegistry::error_general_trouble().to_string(),
            msg.metadata.clone(),
        )
            .await;
        return Ok(());
    }
    let channel_result = channel_exists?;
    if let Some(value) = channel_result {
        info!("failed register because user exist: {}", value.user_id);
        let _ = send_message(
            state,
            msg,
            PromptRegistry::error_account_exists().to_string(),
            msg.metadata.clone(),
        )
            .await;
        return Ok(());
    }

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            let _ = send_message(
                state,
                msg,
                "Something wrong happen, try again later.".to_string(),
                msg.metadata.clone(),
            )
                .await;
            return Ok(());
        }
    };

    info!("begin create user \n");

    let display_name = match msg.metadata.clone() {
        None => msg.sender_id.clone(),
        Some(meta) => meta
            .get("display_name")
            .map_or_else(|| msg.sender_id.clone(), |v| v.to_string()),
    };

    let u_id = match sqlx::query!(
            "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
            msg.sender_id,
            display_name
        ).fetch_one(&mut *tx).await {
        Ok(r) => r.id,
        Err(e) => {
            error!("Failed to resolve user: {}", e);
            let _ = tx.rollback().await;
            let _ = send_message(state, msg, "Failed resolver user account..".to_string(),  msg.metadata.clone()).await;
            return Ok(());
        }
    };

    info!("begin create conversation \n");
    // Create new conversation
    let conv_id = Uuid::new_v4();
    let title = format!("{} via {}", msg.conversation_id, msg.channel);

    if let Err(e) = sqlx::query!(
        "INSERT INTO conversations (id, title,soul_content,bootstrap_content) VALUES ($1, $2,$3,$4)",
        conv_id,
        title,
        PromptRegistry::default_soul_prompts(),
        PromptRegistry::default_bootstrap_content()
    )
        .execute(&mut *tx)
        .await
    {
        error!("Failed to create conversation: {}", e);
        let _ = tx.rollback().await;
        let _ = send_message(state, msg, "Failed to create conversation.".to_string(),  msg.metadata.clone(),).await;

        return Ok(());
    }

    info!("begin create channels");
    if let Err(e) = sqlx::query!(
            "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) VALUES ($1, $2, $3, $4, $5)",
            msg.channel,
            msg.sender_id,
            msg.conversation_id,
            conv_id,
            u_id
        ).execute(&mut *tx).await {
        error!("Failed to link channel: {}", e);
        let _ = tx.rollback().await;
        let _ = send_message(state, msg, "Failed link channel..".to_string(),  msg.metadata.clone(),).await;

        return Ok(());
    }

    if let Err(e) = sqlx::query!(
            "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            conv_id,
            u_id
        ).execute(&mut *tx).await {
        error!("Failed to add member: {}", e);
        let _ = tx.rollback().await;
        let _ = send_message(state, msg, "Failed to join conversation.".to_string(),  msg.metadata.clone(),).await;
        return Ok(());
    }

    if let Err(e) = tx.commit().await {
        error!("Failed to commit registration: {}", e);
        let _ = send_message(
            state,
            msg,
            "Failed register".to_string(),
            msg.metadata.clone(),
        )
            .await;
        return Ok(());
    }

    let _ = send_message(
        state,
        msg,
        "Success register account, you can now /login for access dashboard".to_string(),
        msg.metadata.clone(),
    )
        .await;
    Ok(())
}

pub async fn process_group_registration(
    state: &AppState,
    msg: &InboundMessage,
) -> anyhow::Result<()> {
    info!(
        "Registering group: {} on channel {}",
        msg.conversation_id, msg.channel
    );

    let get_user = channel_repo::get_channel_info(&state.pool, &msg.channel, &msg.sender_id).await;

    if let Err(e) = get_user {
        info!("Only registered user can pairing group :{}", e);
        let _ = send_message(
            state,
            msg,
            "Access restricted. Only registered Arta users can pair groups. Please sign up to continue! 🛡️".to_string(),
            None,
        ).await;
        return Ok(());
    }
    let existing_user = get_user.unwrap();
    if let None = existing_user {
        info!("Group already registered");
        let _ = send_message(
            state,
            msg,
            "Access restricted. Only registered Arta users can pair groups. Please sign up to continue! 🛡️".to_string(),
            None,
        ).await;
        return Ok(());
    }
    let existing_user = existing_user.unwrap();
    let mut tx = state.pool.begin().await?;
    let existing_channel =
        channel_repo::get_channel_group_info(&state.pool, &msg.channel, &msg.sender_id).await;
    if let Ok(data) = existing_channel {
        if data.is_some() {
            info!("Group already registered");
            let existing_group_channel = data.unwrap();
            let _ = channel_repo::link_channel_group(
                &state.pool,
                &msg.channel,
                &msg.conversation_id,
                existing_group_channel.conversation_id,
            )
                .await;
            let _ = send_message(
                state,
                msg,
                "We’re already in sync! This group is fully registered and ready to roll. 🏍️💨"
                    .to_string(),
                None,
            )
                .await;
            return Ok(());
        }
    }
    info!("Group not registered, try to registering");
    // 2. Create new conversation for this group

    let conv_id = Uuid::new_v4();
    let title = format!("Group: {} via {}", msg.conversation_id, msg.channel);

    let trx_convo = sqlx::query!(
        "INSERT INTO conversations (id, title,soul_content,bootstrap_content) VALUES ($1, $2,$3,$4) RETURNING id",
        conv_id,
        title,
        PromptRegistry::default_soul_prompts(),
        PromptRegistry::default_bootstrap_content()
    )
        .fetch_one(&mut *tx)
        .await;

    if let Err(e) = trx_convo {
        info!("Group failed to register registered:{}", e);
        let _ = tx.rollback().await;
        let _ = send_message(
            state,
            msg,
            "Almost there!\n I ran into a slight sync issue with the group registration. ✨ \nI'm smoothing it out behind the scenes as we speak. \n Try again in a few—I’ll be ready for ya! 🚀".to_string(),
            None,
        ).await;
        return Ok(());
    }

    let trx_convo = trx_convo.unwrap();
    let save_members = sqlx::query!(
        "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) RETURNING conversation_id, user_id",
        trx_convo.id,
        existing_user.user_id,
    ).fetch_one(&mut *tx).await;

    if let Err(e) = save_members {
        info!("Group saving member failed to register:{}", e);
        let _ = tx.rollback().await;
        let _ = send_message(
            state,
            msg,
            "Almost there!\n I ran into a slight sync issue with the group registration. ✨ \nI'm smoothing it out behind the scenes as we speak. \n Try again in a few—I’ll be ready for ya! 🚀".to_string(),
            None,
        ).await;
        return Ok(());
    }

    let link_channel_group = sqlx::query!(
        "INSERT INTO channel_group (conversation_id, channel, external_group_id, registered_at,is_active)
         VALUES ($1, $2, $3, now(), true)",
        trx_convo.id,
        msg.channel,
        msg.conversation_id
    ).execute(&mut *tx).await;

    if let Err(e) = link_channel_group {
        info!("Group save channel failed to register:{}", e);
        let _ = tx.rollback().await;
        let _ = send_message(
            state,
            msg,
            "Almost there!\n I ran into a slight sync issue with the group registration. ✨ \nI'm smoothing it out behind the scenes as we speak. \n Try again in a few—I’ll be ready for ya! 🚀".to_string(),
            None,
        ).await;
        return Ok(());
    }
    let trx = tx.commit().await;

    if let Err(err) = trx {
        info!("Failed register group:{}", err);
        let _ = send_message(
            state,
            msg,
            "Almost there!\n I ran into a slight sync issue with the group registration. ✨ \nI'm smoothing it out behind the scenes as we speak. \n Try again in a few—I’ll be ready for ya! 🚀".to_string(),
            None,
        ).await;
        return Ok(());
    }

    let _ = send_message(
        state,
        msg,
        "Group paired and idling! 🏍️💨 Just mention me or send a command whenever you're ready to roll.️".to_string(),
        None,
    ).await;
    Ok(())
}
pub async fn process_login(state: &AppState, msg: &InboundMessage) -> anyhow::Result<()> {
    info!(
        "start login from channel {} sender_id {}",
        msg.channel, msg.sender_id
    );
    // Check if user/channel exists
    let channel_exists = sqlx::query!(
            "SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",
            msg.channel,
            msg.conversation_id
        ).fetch_optional(&state.pool).await;

    if let Err(err) = channel_exists {
        info!(
            "Login from channel {} sender_id {} failed: {}",
            msg.channel, msg.sender_id, err
        );
        let _ = send_message(state, msg, "Channel not registered, Use /register for new user use, if you already had account, get pairing code from dashboard and use /pair <PAIRING CODE>".to_string(),  msg.metadata.clone()).await;
        return Ok(());
    }
    if let Ok(None) = channel_exists {
        info!("channel doesnt exist:");
        let _ = send_message(state, msg, "Channel not registered, Use /register for new user use, if you already had account, get pairing code from dashboard and use /pair <PAIRING CODE>".to_string(), msg.metadata.clone()).await;
        return Ok(());
    }

    let channel_data = channel_exists.unwrap().unwrap();
    let user_id = channel_data.user_id;

    // Generate OTP
    let otp_code: u32 = rand::rng().random_range(100000..999999);
    let otp_str = otp_code.to_string();
    let redis_key = format!("otp:{}", user_id);

    if let Err(e) = state.redis.set_ex(&redis_key, &otp_str, 300).await {
        error!("Failed to store OTP in Redis: {}", e);
        let _ = send_message(
            state,
            msg,
            "I'm having trouble, please try again later.".to_string(),
            msg.metadata.clone(),
        )
            .await;
        return Ok(());
    }

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let login_url = format!("{}/login?id={}", app_url, user_id);
    //hack: we need to sent message twice because it will sent as 2 bubble
    let outbound_text = format!("Your verification code is: {}", otp_str);
    let _ = send_message(state, msg, outbound_text, msg.metadata.clone()).await;
    let outbound_text = format!("Click here to login: {}", login_url);
    let _ = send_message(state, msg, outbound_text, msg.metadata.clone()).await;
    Ok(())
}

pub async fn get_help_command(state: &AppState, msg: &InboundMessage) -> anyhow::Result<()> {
    let message = format!(
        "Hello there! 👋 \n
            I'm **Nomi**, Trian's AI collaborator. I help him manage his projects, track his adventures on the road, and keep his digital ecosystem running smoothly. \n
            If you're a friend of Trian's, I'd love to get to know you! To get started and secure your access to our chat, could you please use one of the commands below?\n
                {} — If this is your first time chatting with me, use this to set up your profile. \n
                {} — If we've spoken before, use this to jump right back into our conversation.\n\
                {} - Ask for help\n
                {} - If you want to connected another channel whatsapp,telegram,slack etc.\n
                {} - Connect app with pairing code,The code are generated  from `/linkapp`.\n
            It’s a pleasure to meet you, and I look forward to assisting you once you're signed in! ✨\n",
        "**`/register`**",
        "**`/login`**",
        "**`/help`**",
        "**`/linkapp`**",
        "**`/pair <PAIRING CODE>`**",
    );

    let _ = send_message(state, msg, message, msg.metadata.clone()).await;
    Ok(())
}

pub async fn send_message(
    state: &AppState,
    msg: &InboundMessage,
    text: String,
    meta: Option<serde_json::Value>,
) -> anyhow::Result<()> {
    let _ = state
        .publish_outbond(&OutboundMessage {
            is_group: msg.is_group,
            sender_id: "nomi".to_string(),
            conversation_id: msg.conversation_id.to_string(),
            text: text,
            channel: msg.channel.to_string(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: meta,
        })
        .await;
    Ok(())
}