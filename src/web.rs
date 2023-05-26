pub mod web {
	use crate::db::DB;
	use rocket::{catch, fs::NamedFile, get, State};
	use rocket_dyn_templates::{Template, context};
	use std::path::{Path, PathBuf};

	#[catch(404)]
	pub fn not_found() -> Template {
		Template::render("404", context! {})
	}

	#[get("/<file..>")]
	pub async fn static_files(_db: &State<DB>, file: PathBuf) -> Option<NamedFile> {
		NamedFile::open(Path::new("static/").join(file)).await.ok()
	}

	pub mod frontend {
		use rocket::get;
		use rocket_dyn_templates::{context, Template};

		#[get("/")]
		pub fn index() -> Template {
			Template::render(
				"index",
				context! {
					name: "Thomas"
				},
			)
		}
	}

	pub mod api {
		use crate::db::{User, DB};
		use pwhash::bcrypt;
		use rocket::{get, State};

		#[get("/")]
		pub fn index() -> String {
			String::from("Hello world!")
		}

		#[get("/user/<id>")]
		pub async fn user(db: &State<DB>, id: i32) -> String {
			// Get name from db
			let user: User = db.get_user(id).await.unwrap();
			format!("User name is {}", user.name)
		}

		#[get("/hash/<password>")]
		pub async fn hash(_db: &State<DB>, password: String) -> String {
			let h: String = bcrypt::hash(password).unwrap();
			format!("Hashed password is {}, the lenght is {}", h, h.len())
		}

		#[get("/hash/<password>/<h>")]
		pub async fn verify(_db: &State<DB>, password: String, h: String) -> String {
			let result: bool = bcrypt::verify(password, &h);
			if result {
				String::from("Password matches hash")
			} else {
				String::from("Password does not match hash")
			}
		}
	}
}
