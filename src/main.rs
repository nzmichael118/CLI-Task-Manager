use chrono::{format::strftime::StrftimeItems, Local, NaiveDateTime};
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs::File, usize};
use structopt::StructOpt;
// --- Arg parsing struct and enums -------

#[derive(Debug, StructOpt)]
#[structopt(name = "Taskmanager", about = "Another RUST task manager")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}
#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "add", about = "Add a new task")]
    Add {
        #[structopt(name = "name", help = "Name of the task")]
        name: String,
        #[structopt(short = "d", long = "description", help = "Description of task")]
        description: Option<String>,
        #[structopt(short = "u", long = "urgency", help = "Urgency of task")]
        urgency: Option<f32>,
        #[structopt(short = "D", long = "due-time", help = "Due time of task")]
        due_time: Option<String>,
    },
    #[structopt(name = "view", about = "View task by ID")]
    View {
        #[structopt(name = "id", help = "Index of task")]
        id: usize,
    },
    #[structopt(name = "list", about = "List all the tasks")]
    List,
    #[structopt(name = "edit", about = "Edit a tasks values by ID")]
    Edit {
        #[structopt(name = "id", about = "ID of task")]
        id: usize,
        #[structopt(short = "n", long = "name", help = "Name of the task")]
        name: Option<String>,
        #[structopt(short = "d", long = "description", help = "Description of task")]
        description: Option<String>,
        #[structopt(short = "u", long = "urgency", help = "Urgency of task")]
        urgency: Option<f32>,
        #[structopt(short = "D", long = "due-time", help = "Due time of task")]
        due_time: Option<String>,
    },
    #[structopt(name = "start", about = "Set a task to active by ID")]
    Start { id: usize },
    #[structopt(name = "stop", about = "Set a task to inactive by ID")]
    Stop { id: usize },
    #[structopt(name = "done", about = "Set a task to Complete by ID")]
    Done { id: usize },
    #[structopt(name = "remove", about = "Remove a task by ID")]
    Remove { id: usize },
}

// ------------Structs and Enums ---------------
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Task {
    title: String,
    description: String,
    status: Status,
    urgency: f32,
    start_time: Option<NaiveDateTime>,
    due_time: Option<NaiveDateTime>, // May not exist
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TaskManager {
    tasks: Vec<Task>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Status {
    Inactive,
    Active,
    Done,
}
// ------------- Implimentations ----------------
impl TaskManager {
    fn new() -> Self {
        TaskManager { tasks: Vec::new() }
    }
    fn save_to_file(&self, filename: &PathBuf) -> Result<(), Box<dyn Error>> {
        let file = File::create(filename)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn load_from_file(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let task_manager: TaskManager = serde_json::from_reader(reader)?;
        Ok(task_manager)
    }

    fn add_task(&mut self, title: String) {
        let new_task = {
            Task {
                title,
                description: String::new(),
                status: Status::Inactive,
                urgency: 3.0,
                start_time: Some(Local::now().naive_local()),
                due_time: None,
            }
        };
        self.tasks.push(new_task);
    }

    fn verify_id(&mut self, id: usize) -> bool {
        if id < self.tasks.len() {
            return true;
        }
        false
    }
    // ----- Task Setters -----
    fn set_task_name(&mut self, id: usize, new_name: String) {
        if self.verify_id(id) {
            self.tasks[id].title = new_name;
        } else {
            eprintln!("Invalid ID");
        }
    }
    fn set_task_description(&mut self, id: usize, new_description: String) {
        if self.verify_id(id) {
            self.tasks[id].description = new_description;
        } else {
            eprintln!("Invalid ID");
        }
    }
    fn set_task_status(&mut self, id: usize, new_status: Status) {
        if self.verify_id(id) {
            self.tasks[id].status = new_status;
        } else {
            eprintln!("Invalid ID");
        }
    }

    fn set_urgency(&mut self, id: usize, new_urgency: f32) {
        if self.verify_id(id) {
            if new_urgency >= 0.0 && new_urgency <= 10.0 {
                self.tasks[id].urgency = new_urgency;
            } else {
                eprintln!(
                    "Urgency must be between 0.0 and 10.0, you inputted {}",
                    new_urgency
                );
            }
        } else {
            eprintln!("Invalid ID");
        }
    }

    fn set_due_date(&mut self, id: usize, new_due_date: NaiveDateTime) {
        if self.verify_id(id) {
            self.tasks[id].due_time = Some(new_due_date);
        } else {
            eprintln!("Invalid ID");
        }
    }

    fn remove_task_by_id(&mut self, id: usize) {
        if self.verify_id(id) {
            self.tasks.remove(id);
        } else {
            eprintln!("Invalid ID");
        }
    }
    // -------------------------
    fn list_tasks(&mut self) {
        if self.tasks.is_empty() {
            println!("There are currently no tasks :)");
        } else {
            println!("Tasks:");

            for (index, task) in self.tasks.iter().enumerate() {
                let status_to_str = match task.status {
                    Status::Inactive => "Inactive",
                    Status::Active => "Active",
                    Status::Done => "Done",
                };
                let format = StrftimeItems::new("%d/%m/%Y");
                let formatted_time = task.start_time.unwrap().format_with_items(format);
                println!(
                    " ~ {}: {:width$} | Status: {:w_8$}, Start: {} Urg: {}",
                    index,
                    task.title,
                    status_to_str,
                    formatted_time,
                    task.urgency,
                    width = 60,
                    w_8 = 8,
                );
            }
        }
    }
    // ---
    fn show_task(&mut self, id: usize) {
        if self.verify_id(id) {
            println!(
                " -{}- {} --- urgency: {}",
                id, self.tasks[id].title, self.tasks[id].urgency
            );
            println!("  {}", self.tasks[id].description);
            let format = StrftimeItems::new("%H:%M, %d-%m-%Y");
            let formatted_start_time = self.tasks[id].start_time.unwrap().format_with_items(format);
            match self.tasks[id].due_time {
                Some(_) => {
                    let format = StrftimeItems::new("%H:%M, %d-%m-%Y");
                    let formatted_due_time =
                        self.tasks[id].due_time.unwrap().format_with_items(format);
                    println!(
                        " - start: {}    due: {} ",
                        formatted_start_time, formatted_due_time
                    );
                }
                None => {
                    println!(" - start: {}    due: No Due Date", formatted_start_time);
                }
            }
        }
    }
}

// ------------------------
fn main() -> Result<(), Box<dyn Error>> {
    let mut app_data_dir = match data_dir() {
        Some(dir) => dir,
        None => {
            eprint!("Failed to determine Data Directory");
            return Ok(());
        }
    };
    app_data_dir.push("task");
    app_data_dir.push("task.json");
    //println!("{}", app_data_dir.display());

    let mut task_manager = TaskManager::load_from_file(&app_data_dir)?;
    //let mut task_manager = TaskManager::new();

    let opt = Opt::from_args();
    match opt.command {
        Command::Add {
            name,
            description,
            urgency,
            due_time,
        } => {
            task_manager.add_task(name);
            if let Some(description) = description {
                task_manager.set_task_description(task_manager.tasks.len() - 1, description);
            }
            if let Some(urgency) = urgency {
                task_manager.set_urgency(task_manager.tasks.len() - 1, urgency);
            }
            if let Some(due_time) = due_time {
                // Verify
                let date_str: &str = &due_time;
                match NaiveDateTime::parse_from_str(date_str, "%d/%m/%Y") {
                    // BUG TODO FIXME
                    Ok(date) => task_manager.set_due_date(task_manager.tasks.len() - 1, date),
                    Err(err) => {
                        eprintln!("Error parsing date {}, expected format d/m/y", err);
                    }
                }
            }
        }
        Command::View { id } => {
            task_manager.show_task(id);
        }
        Command::List => {
            task_manager.list_tasks();
        }
        Command::Edit {
            id,
            name,
            description,
            urgency,
            due_time,
        } => {
            if let Some(name) = name {
                task_manager.set_task_name(id, name);
            }
            if let Some(description) = description {
                task_manager.set_task_description(id, description);
            }
            if let Some(urgency) = urgency {
                task_manager.set_urgency(id, urgency);
            }
            if let Some(due_time) = due_time {
                let date_str: &str = &due_time;
                match NaiveDateTime::parse_from_str(date_str, "%d/%m/%Y") {
                    Ok(date) => task_manager.set_due_date(id, date),
                    Err(err) => {
                        eprintln!("Error parsing date {}, expected format d/m/y", err);
                    }
                }
            }
        }
        Command::Start { id } => {
            task_manager.set_task_status(id, Status::Active);
        }
        Command::Stop { id } => {
            task_manager.set_task_status(id, Status::Inactive);
        }
        Command::Done { id } => {
            task_manager.set_task_status(id, Status::Done);
            task_manager.set_urgency(id, 0.0);
        }
        Command::Remove { id } => {
            task_manager.remove_task_by_id(id);
        }
    }

    task_manager.save_to_file(&app_data_dir)?;
    Ok(())
}
// ------------------------ Debugs
#[cfg(test)]
mod tests {
    use crate::Status;
    use crate::TaskManager;
    #[test]
    fn create_and_modify_task() {
        let mut debug_manager = TaskManager::new();
        debug_manager.add_task("task_1".to_string());
        assert_eq!(debug_manager.tasks[0].title, "task_1");
        assert_eq!(debug_manager.tasks[0].status, Status::Inactive);
        debug_manager.set_task_status(0, Status::Active);
        assert_eq!(debug_manager.tasks[0].status, Status::Active);
        debug_manager.set_task_status(0, Status::Done);
        assert_eq!(debug_manager.tasks[0].status, Status::Done);
    }
}
