use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, prelude::FromRow, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs::OpenOptions;
use tauri::{App, Manager as _};

type Db = Pool<Sqlite>;

struct AppState {
    db: Db,
}

pub fn run() {
     tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            add_todo,
            get_todos,
            update_todo,
            delete_todo
        ])
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let db = setup_db(&app).await;

                app.manage(AppState { db });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error building the app");

}

async fn setup_db(app: &App) -> Db {
    let mut path = app.path().app_data_dir().expect("failed to get data_dir");

    match std::fs::create_dir_all(path.clone()) {
        Ok(_) => {}
        Err(err) => {
            panic!("error creating directory {}", err);
        }
    };

    path.push("db.sqlite");

    Sqlite::create_database(
        format!(
            "sqlite:{}",
            path.to_str().expect("path should be something")
        )
        .as_str(),
    )
    .await
    .expect("failed to create database");

    let db = SqlitePoolOptions::new()
        .connect(path.to_str().unwrap())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    db
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
enum TodoStatus {
    Incomplete,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Todo {
    id: u16,
    description: String,
    status: TodoStatus,
}

#[tauri::command]
async fn add_todo(state: tauri::State<'_, AppState>, description: &str) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("INSERT INTO todos (description, status) VALUES (?1, ?2)")
        .bind(description)
        .bind(TodoStatus::Incomplete)
        .execute(db)
        .await
        .map_err(|e| format!("Error saving todo: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_todos(state: tauri::State<'_, AppState>) -> Result<Vec<Todo>, String> {
    let db = &state.db;

    let todos: Vec<Todo> = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch(db)
        .try_collect()
        .await
        .map_err(|e| format!("Failed to get todos {}", e))?;

    Ok(todos)
}

#[tauri::command]
async fn update_todo(state: tauri::State<'_, AppState>, todo: Todo) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("UPDATE todos SET description = ?1, status = ?2 WHERE id = ?3")
        .bind(todo.description)
        .bind(todo.status)
        .bind(todo.id)
        .execute(db)
        .await
        .map_err(|e| format!("could not update todo {}", e))?;

    Ok(())
}

#[tauri::command]
async fn delete_todo(state: tauri::State<'_, AppState>, id: u16) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| format!("could not delete todo {}", e))?;

    Ok(())
}

