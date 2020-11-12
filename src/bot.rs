use futures_util::StreamExt;
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
				UpdateKind::Message(msg) => handle_msg(&api, msg).await?,
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

async fn handle_msg(api: &Api, msg: Message) -> Result<(), Error> {
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
			return api
				.send(msg.text_reply("Sorry, this bot only reads text messages."))
				.await
				.map(|_| ())
		},
	};

	let user_token = "THISTOKENISNONSENSE";

	if text.starts_with("/start") {
		api.send(SendMessage::new(user, format!("Welcome to the Pollus Bot! I'm here to give you access to our great Pollus Polls! Your personal link is https://pollus.msrd0.de/auth/{token}. If you forget your link, just type '/link' and I'll send it to again!", token = user_token))).await?;
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
