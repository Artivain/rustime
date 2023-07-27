use crate::db::DB;
use crate::scheduler::Scheduler;
use crate::web::web::{api, frontend, not_found, static_files};
use dotenvy::dotenv;
use rocket::http::Status;
use rocket::response::status;
use rocket::{catchers, get, routes, State};
use rocket_dyn_templates::{context, Template};

mod db;
mod notify;
mod scheduler;
mod web;

pub static USER_AGENT: &str = concat!("Rustime/", env!("CARGO_PKG_VERSION"));

#[rocket::main]
async fn main() {
	dotenv().ok();

	println!("Making sure required static files exist");
	match check_static() {
		Ok(_) => (),
		Err(err) => {
			emergency_web_server(format!("Please compile static files\n{}", err)).await;
			return;
		}
	}

	println!("Initializing database");
	let db: DB = match DB::new().await {
		Ok(db) => db,
		Err(err) => {
			emergency_web_server(format!("Failed to initialize database\n{}", err)).await;
			return;
		}
	};

	println!("Initializing scheduler");
	match Scheduler::new().await {
		Ok(_) => (),
		Err(err) => {
			emergency_web_server(format!("Failed to initialize scheduler\n{}", err)).await;
			return;
		}
	}

	println!("Starting web server");
	match rocket::build()
		.attach(Template::fairing())
		.mount("/", routes![static_files])
		.register("/", catchers![not_found])
		.mount(
			"/api",
			routes![api::index, api::user, api::hash, api::verify],
		)
		.mount("/", routes![frontend::index])
		.manage(db)
		.launch()
		.await
	{
		Ok(_) => (),
		Err(err) => {
			emergency_web_server(format!("Failed to launch web server\n{}", err)).await;
			return;
		}
	}
}

fn check_static() -> Result<(), String> {
	const STATIC_FILES: [&str; 3] = ["js/script.js", "css/style.css", "js/theme.js"];
	// For each file in STATIC_FILES, check if it exists and is not empty
	for file in STATIC_FILES.iter() {
		let path: String = format!("static/{}", file);
		let content: String = match std::fs::read_to_string(&path) {
			Ok(content) => content,
			Err(_) => return Err(format!("File {} does not exist", path)),
		};
		if content.is_empty() {
			return Err(format!("File {} is empty", path));
		}
	}
	Ok(())
}

async fn emergency_web_server(error: String) {
	rocket::build()
		.attach(Template::fairing())
		.manage(EmergencyWebServer { error })
		.mount("/", routes![emergency_route])
		.launch()
		.await
		.expect("Failed to launch emergency web server");
}

#[get("/")]
fn emergency_route(state: &State<EmergencyWebServer>) -> status::Custom<Template> {
	status::Custom(
		Status::InternalServerError,
		Template::render("emergency", context! {error: &state.error}),
	)
}

struct EmergencyWebServer {
	error: String,
}
