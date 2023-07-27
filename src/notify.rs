use std::time::Duration;

use reqwest::{Client, ClientBuilder};

use crate::{db::{Schedule, DB}, USER_AGENT};

pub async fn notify_up(db: &DB, schedule: &Schedule) {
	let client: Client = match ClientBuilder::new()
		.timeout(Duration::from_secs(30))
		.user_agent(USER_AGENT)
		.build()
	{
		Ok(client) => client,
		Err(err) => {
			println!(
				"Failed to create HTTP client for sending notification: {}",
				err
			);
			return;
		}
	};
}
