use sqlx::{
	migrate, mysql::MySqlQueryResult, query, query_as, types::chrono::NaiveDateTime, Error, MySql,
	MySqlPool, Pool,
};
use std::env;

pub struct DB {
	pub pool: MySqlPool,
}

impl DB {
	pub async fn new() -> Result<DB, String> {
		let url: String =
			env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");

		match url.split('/').last() {
			Some(_) => (),
			None => {
				return Err(String::from(
					"DATABASE_URL environment variable does not contain a database name",
				))
			}
		}

		let pool: Pool<MySql> = match MySqlPool::connect(&url).await {
			Ok(pool) => pool,
			Err(err) => return Err(format!("Failed to connect to database\n{}", err)),
		};

		let db: DB = DB { pool };
		match migrate!().run(&db.pool).await {
			Ok(_) => Ok(db),
			Err(err) => Err(format!("Failed to migrate database\n{}", err)),
		}
	}

	pub async fn get_user(&self, id: i32) -> Result<User, Error> {
		query_as!(User, "SELECT id, name FROM users WHERE id = ?", id)
			.fetch_one(&self.pool)
			.await
	}

	pub async fn get_schedule(&self, id: i32) -> Result<Schedule, Error> {
		query_as!(Schedule, "SELECT id, name, cron, enabled as 'enabled: bool', target, method as 'method: Method', is_up as 'is_up: bool', last_down, down_reason, created_at FROM schedules WHERE id = ?", id)
			.fetch_one(&self.pool)
			.await
	}

	pub async fn set_schedule_up(&self, id: i32) -> Result<(), Error> {
		self.update_schedule_status(true, None, id).await
	}

	pub async fn set_schedule_down(&self, id: i32, down_reason: String) -> Result<(), Error> {
		self.update_schedule_status(false, Some(down_reason), id).await
	}

	pub async fn update_schedule_status(
		&self,
		is_up: bool,
		down_reason: Option<String>,
		id: i32,
	) -> Result<(), Error> {
		match is_up {
			true => query!(
				"UPDATE schedules SET is_up = ?, down_reason = ? WHERE id = ?",
				is_up,
				down_reason,
				id
			)
			.execute(&self.pool)
			.await?,

			false => query!(
				"UPDATE schedules SET is_up = ?, down_reason = ?, last_down = CURRENT_TIMESTAMP() WHERE id = ?",
				is_up,
				down_reason,
				id
			)
			.execute(&self.pool)
			.await?,
		};

		Ok(())
	}

	pub async fn get_schedules(&self) -> Result<Vec<Schedule>, Error> {
		query_as!(
			Schedule,
			"SELECT id, name, cron, enabled as 'enabled: bool', target, method as 'method: Method', is_up as 'is_up: bool', last_down, down_reason, created_at FROM schedules"
		)
		.fetch_all(&self.pool)
		.await
	}

	pub async fn get_jobs_to_run(&self) -> Result<Vec<Job>, Error> {
		query_as!(Job, "SELECT id, type as 'job_type: JobType', linked_id, run_at FROM jobs WHERE run_at <= NOW()")
			.fetch_all(&self.pool)
			.await
	}

	pub async fn get_job_by_linked_id(&self, linked_id: i32) -> Result<Job, Error> {
		query_as!(Job, "SELECT id, type as 'job_type: JobType', linked_id, run_at FROM jobs WHERE linked_id = ? LIMIT 1", linked_id)
			.fetch_one(&self.pool)
			.await
	}

	pub async fn create_job(
		&self,
		job_type: JobType,
		run_at: NaiveDateTime,
		linked_id: Option<i32>,
	) -> Result<(), Error> {
		let result: Result<MySqlQueryResult, Error> = match linked_id {
			Some(linked_id) => {
				query!(
					"INSERT INTO jobs (type, run_at, linked_id) VALUES (?, ?, ?)",
					job_type as i32,
					run_at,
					linked_id
				)
				.execute(&self.pool)
				.await
			}
			None => {
				query!(
					"INSERT INTO jobs (type, run_at) VALUES (?, ?)",
					job_type as i32,
					run_at
				)
				.execute(&self.pool)
				.await
			}
		};

		match result {
			Ok(_) => Ok(()),
			Err(err) => Err(err),
		}
	}

	pub async fn delete_job(&self, id: u64) -> Result<(), Error> {
		query!("DELETE FROM jobs WHERE id = ?", id)
			.execute(&self.pool)
			.await?;

		Ok(())
	}

	pub async fn close(self) {
		self.pool.close().await;
	}
}

pub struct User {
	pub id: i32,
	pub name: String,
}

pub struct Schedule {
	pub id: i32,
	pub name: String,
	pub cron: String,
	pub enabled: bool,
	pub target: String,
	pub method: Method,
	pub is_up: bool,
	pub last_down: Option<NaiveDateTime>,
	pub down_reason: Option<String>,
	pub created_at: NaiveDateTime,
}

pub struct Job {
	pub id: u64,
	pub job_type: JobType,
	pub linked_id: Option<i32>,
	pub run_at: NaiveDateTime,
}

#[derive(sqlx::Type)]
#[repr(i32)]
pub enum JobType {
	Monitoring = 1,
}

#[derive(sqlx::Type)]
#[repr(i8)]
pub enum Method {
	HEAD = 1,
	GET = 2,
	POST = 3,
}
