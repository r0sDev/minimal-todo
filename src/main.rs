use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DATA_FILE: &str = ".local/share/todo-rust/todos.json";

#[derive(Serialize, Deserialize)]
struct Todo {
    id: u32,
    status: Status,
    description: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
enum Status {
    Pending,
    Done,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::Done => write!(f, "done"),
        }
    }
}

#[derive(Parser)]
#[command(name = "todo-rust", about = "self tasks organizer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { description: String },
    List { filter: Option<String> },
    Done { id: u32 },
    Delete { id: u32 },
    Edit { id: u32, description: String },
    Search { query: String },
    Stats,
}

struct TodoStore {
    todos: Vec<Todo>,
    path: PathBuf,
    next_id: u32,
}

impl TodoStore {
    fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let path = home.join(DATA_FILE);
        let todos = Self::load(&path);
        let next_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        TodoStore { todos, path, next_id }
    }

    fn load(path: &PathBuf) -> Vec<Todo> {
        if path.exists() {
            let data = fs::read_to_string(path).unwrap_or_default();
            if data.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&data).unwrap_or_default()
            }
        } else {
            Vec::new()
        }
    }

    fn save(&mut self) {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let data = serde_json::to_string_pretty(&self.todos).unwrap_or_default();
        fs::write(&self.path, data).expect("No se pudo guardar el archivo");
    }

    fn add(&mut self, description: String) {
        let id = self.next_id;
        self.todos.push(Todo {
            id,
            status: Status::Pending,
            description: description.clone(),
        });
        self.next_id += 1;
        self.save();
	println!("");
        println!(" ------------- Tarea #{} añadida: {}", id, description);
    }

    fn list(&self, filter: &str) {
        println!("");
        let mut count = 0;
        for todo in &self.todos {
            if filter == "all" || todo.status.to_string() == filter {
                count += 1;
                match todo.status {
                    Status::Pending => {
                        println!(" [ ] #{} {}", todo.id, todo.description);
                    }
                    Status::Done => {
                        println!(" [x] #{} {}", todo.id, todo.description);
                    }
                }
            }
        }

        if count == 0 {
            println!("  sin tareas");
        }

        let pending = self.todos.iter().filter(|t| t.status == Status::Pending).count();
        let done = self.todos.iter().filter(|t| t.status == Status::Done).count();
        println!("");
        println!(" Pendientes: {}  |  Completadas: {}", pending, done);
    }

    fn mark_done(&mut self, id: u32) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id && t.status == Status::Pending) {
            todo.status = Status::Done;
            let desc = todo.description.clone();
            self.save();
	    println!("");
            println!(" Tarea #{} marcada como hecha: {}", id, desc);
        } else {
	    println!("");
            println!(" Tarea #{} no encontrada o ya completada.", id);
        }
    }

    fn delete(&mut self, id: u32) {
        if let Some(pos) = self.todos.iter().position(|t| t.id == id) {
            let desc = self.todos[pos].description.clone();
            self.todos.remove(pos);
            self.save();
            println!("Tarea #{} eliminada: {}", id, desc);
        } else {
            println!("Tarea #{} no encontrada.", id);
        }
    }

    fn edit(&mut self, id: u32, new_desc: String) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.description = new_desc.clone();
            self.save();
	    println!("");
            println!(" Tarea #{} actualizada: {}", id, new_desc);
        } else {
	    println!("");
            println!(" Tarea #{} no encontrada.", id);
        }
    }

    fn search(&self, query: &str) {
        println!("");
        println!("  RESULTADOS para: {}", query);

        let mut count = 0;
        for todo in &self.todos {
            if todo.description.to_lowercase().contains(&query.to_lowercase()) {
                count += 1;
                match todo.status {
                    Status::Pending => {
                        println!("  [ ] #{} {}", todo.id, todo.description);
                    }
                    Status::Done => {
                        println!("  [x] #{} {}", todo.id, todo.description);
                    }
                }
            }
        }

        if count == 0 {
            println!("  (sin coincidencias)");
        }
    }

    fn stats(&self) {
        let total = self.todos.len();
        let pending = self.todos.iter().filter(|t| t.status == Status::Pending).count();
        let done = self.todos.iter().filter(|t| t.status == Status::Done).count();

        println!("");
        println!("  ESTADISTICAS");
        println!("");
        println!("  Total tareas:   {}", total);
        println!("  Pendientes:     {}", pending);
        println!("  Completadas:    {}", done);

        if total > 0 {
            let pct = done * 100 / total;
            println!("  Progreso:       {}%", pct);
            println!("");
            let bar_width = 30;
            let filled = pct * bar_width / 100;
            let empty = bar_width - filled;
            print!("  ");
            for _ in 0..filled {
                print!("#");
            }
            for _ in 0..empty {
                print!(".");
            }
            println!("] {}%", pct);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let mut store = TodoStore::new();

    match cli.command {
        Commands::Add { description } => {
            store.add(description);
        }
        Commands::List { filter } => {
            let f = filter.unwrap_or_else(|| "all".to_string());
            store.list(&f);
        }
        Commands::Done { id } => {
            store.mark_done(id);
        }
        Commands::Delete { id } => {
            store.delete(id);
        }
        Commands::Edit { id, description } => {
            store.edit(id, description);
        }
        Commands::Search { query } => {
            store.search(&query);
        }
        Commands::Stats => {
            store.stats();
        }
    }
}
