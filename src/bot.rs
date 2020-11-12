use crate::{POOL, SECRET};
use futures_util::StreamExt;
use gotham::anyhow::Error;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use sqlx::query;
use std::env;
use telegram_bot::*;
use tokio::runtime::Runtime;

pub(super) fn start() {
	let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must contain a valid token");
	let api = Api::new(token);
	info!("Starting Telegram Bot");

	let mut rt = Runtime::new().expect("Failed to create runtime");
	let res: Result<(), Error> = rt.block_on(async move {
		let mut stream = api.stream();
		while let Some(update) = stream.next().await {
			let update = update?;
			match update.kind {
				UpdateKind::Message(msg) => match handle_msg(&api, &msg).await {
					Ok(_) => {},
					Err(err) => error!("Unable to handle msg {:?}: {}", msg, err)
				},
				other => warn!("Ignoring unknown update: {:?}", other)
			};
		}
		Ok(())
	});
	match res {
		Ok(_) => warn!("Telegram Bot terminated"),
		Err(err) => error!("Telegram Bot terminated due to error: {}", err)
	}
}

async fn handle_msg(api: &Api, msg: &Message) -> Result<(), Error> {
	let user = match &msg.chat {
		MessageChat::Private(user) => user,
		MessageChat::Group(group) => return leave_group(api, &group).await,
		MessageChat::Supergroup(group) => return leave_group(api, &group).await,
		MessageChat::Unknown(_) => {
			warn!("Ignoring message in unknown chat");
			return Ok(());
		}
	};

	let text = match &msg.kind {
		MessageKind::Text { data, .. } => data,
		_ => {
			api.send(msg.text_reply("Sorry, this bot only reads text messages.")).await?;
			return Ok(());
		}
	};

	let user_id = user.id.to_string();
	info!("Handling message from {}", user_id);
	let user_token = query!(
		"SELECT u.user_token FROM poll_user u INNER JOIN tg_user t ON t.poll_user = u.id WHERE t.user_id = $1;",
		&user_id
	)
	.fetch_optional(&*POOL)
	.await?;
	let user_token = match user_token {
		Some(record) => record.user_token,
		None => {
			info!("Creating new token for {}", user_id);
			let mut mac = Hmac::<Sha256>::new_varkey(SECRET.as_bytes())?;
			mac.update(user_id.as_bytes());
			let user_token = base64::encode_config(mac.finalize().into_bytes(), base64::URL_SAFE_NO_PAD);

			let id = query!(
				"INSERT INTO poll_user (firstname, lastname, user_token) VALUES ($1, $2, $3) RETURNING id;",
				user.first_name,
				user.last_name,
				user_token
			)
			.fetch_one(&*POOL)
			.await?
			.id;
			query!(
				"INSERT INTO tg_user (user_id, username, poll_user) VALUES ($1, $2, $3);",
				user_id,
				user.username,
				id
			)
			.execute(&*POOL)
			.await?;

			user_token
		}
	};

	if text.starts_with("/start") {
		api.send(SendMessage::new(
			user,
			format!(
				"Welcome to the Pollus Bot! I'm here to give you access to our great Pollus Polls! Your personal link is https://pollus.msrd0.de/auth/{token}. If you forget your link, just type '/link' and I'll send it to again!",
				token = user_token
			)
		)).await?;
	} else if text.starts_with("/link") {
		api.send(SendMessage::new(
			user,
			format!(
				"Your personal link is https://pollus.msrd0.de/auth/{token}",
				token = user_token
			)
		))
		.await?;
	} else {
		api.send(msg.text_reply("Sorry, I was unable to understand your request."))
			.await?;
	}

	Ok(())
}

async fn leave_group<C: ToChatRef>(api: &Api, chat: &C) -> Result<(), Error> {
	api.send(SendMessage::new(chat, "Sorry, this bot only supports private chats."))
		.await?;
	api.send(LeaveChat::new(chat)).await?;
	Ok(())
}
