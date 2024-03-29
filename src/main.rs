use chrono::{format::strftime::StrftimeItems, Local, NaiveDateTime};
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs::File, usize};
use structopt::StructOpt;
use term_size::dimensions;
// CONSTS
//
// Urgencies
const URGENCY_MULTIPLIER: f32 = 0.5;
const DEFAULT_URGENCY: f32 = 3.0;
const MINIMUM_URGENCY: f32 = 0.0;
const MAXIMUM_URGENCY: f32 = 10.0;

// Error Messages
const ERR_INVALID_ID: &str = "Invalid ID";

const DEFAULT_TERMINAL_WIDTH: usize = 95;

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
    due_time: Option<NaiveDateTime>,
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

    fn calculate_urgencies(&mut self) {
        for task in self.tasks.iter_mut() {
            if task.status != Status::Done {
                match task.due_time {
                    Some(due_time) => {
                        // Calculate ratio from start to due-time and set minimum urgency
                        let total_time_difference = due_time - task.start_time.unwrap();
                        let time_difference_since_start_time =
                            Local::now().naive_local() - task.start_time.unwrap();
                        let difference_difference_ratio: f32 =
                            time_difference_since_start_time.num_seconds() as f32
                                / total_time_difference.num_seconds() as f32;

                        let minimum_urgency: f32 = difference_difference_ratio * MAXIMUM_URGENCY;
                        if minimum_urgency > task.urgency {
                            //println!("{} task urgency changed to {}", task.title, minimum_urgency);
                            task.urgency = minimum_urgency; // Intentially by design to let overdue projects go above urgency 10
                        }
                    }
                    None => {
                        // Calculate Days since task to find a minimum urgency
                        let current_time = Local::now().naive_local();
                        let time_difference = current_time - task.start_time.unwrap();
                        let days_difference = time_difference.num_days();
                        let mut minimum_urgency: f32 = days_difference as f32 * URGENCY_MULTIPLIER;
                        if minimum_urgency > MAXIMUM_URGENCY {
                            minimum_urgency = MAXIMUM_URGENCY;
                        }
                        if minimum_urgency > task.urgency {
                            // println!("{} task urgency changed to {}", task.title, minimum_urgency);
                            task.urgency = minimum_urgency;
                        }
                    }
                }
            }
        }
    }

    fn sort_by_urgencies(&mut self) {
        self.tasks
            .sort_by_key(|s| std::cmp::Reverse(s.urgency.to_bits()));
    }

    fn add_task(&mut self, title: String) {
        let new_task = {
            Task {
                title,
                description: String::new(),
                status: Status::Inactive,
                urgency: DEFAULT_URGENCY,
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
            eprintln!("{ERR_INVALID_ID}");
        }
    }
    fn set_task_description(&mut self, id: usize, new_description: String) {
        if self.verify_id(id) {
            self.tasks[id].description = new_description;
        } else {
            eprintln!("{ERR_INVALID_ID}");
        }
    }
    fn set_task_status(&mut self, id: usize, new_status: Status) {
        if self.verify_id(id) {
            self.tasks[id].status = new_status;
        } else {
            eprintln!("{ERR_INVALID_ID}");
        }
    }

    fn set_urgency(&mut self, id: usize, new_urgency: f32) {
        if self.verify_id(id) {
            if new_urgency >= MINIMUM_URGENCY && new_urgency <= MAXIMUM_URGENCY {
                self.tasks[id].urgency = new_urgency;
            } else {
                eprintln!(
                    "Urgency must be between {MINIMUM_URGENCY} and {MAXIMUM_URGENCY}, you inputted {}",
                    new_urgency
                );
            }
        } else {
            eprintln!("{ERR_INVALID_ID}");
        }
    }

    fn set_partial_due_date(&mut self, id: usize, date_str: &str) {
        let datetime_string = format!("{} 17:00:00", date_str);
        let datetime_str: &str = &datetime_string;
        match NaiveDateTime::parse_from_str(datetime_str, "%d/%m/%Y %H:%M:%S") {
            Ok(date) => self.set_due_date(id, date),
            Err(err) => {
                eprintln!(
                    "{}, submitted: {}, expected format d/m/y",
                    err, datetime_str
                );
            }
        }
    }
    fn set_due_date(&mut self, id: usize, new_due_date: NaiveDateTime) {
        if self.verify_id(id) {
            self.tasks[id].due_time = Some(new_due_date);
        } else {
            eprintln!("{ERR_INVALID_ID}");
        }
    }

    fn remove_task_by_id(&mut self, id: usize) {
        if self.verify_id(id) {
            self.tasks.remove(id);
        } else {
            eprintln!("{ERR_INVALID_ID}");
        }
    }
    // -------------------------
    fn list_tasks(&mut self) {
        if self.tasks.is_empty() {
            println!("There are currently no tasks :)");
        } else {
            let term_width = match dimensions() {
                Some((w, _)) => w,
                None => {
                    println!("Unable to determine terminal width using default width {DEFAULT_TERMINAL_WIDTH}");
                    DEFAULT_TERMINAL_WIDTH
                }
            };

            println!(
                "ID | URG | {:width$} | STATUS ",
                "DESCRIPTION",
                width = term_width - 32
            ); // Hard coded mess

            for (index, task) in self.tasks.iter().enumerate() {
                let status_to_str = match task.status {
                    Status::Inactive => "Inactive",
                    Status::Active => "Active",
                    Status::Done => "Done",
                };
//                let format = StrftimeItems::new("%d/%m/%Y");
//                let formatted_time = task.start_time.unwrap().format_with_items(format);
                let title_cut = format!("{:.width$}", task.title, width = term_width - 32);
                // New and Improved!
                println!("{:^3}| {:^3} | {:<description_length$} | {:.8}",
                         index, task.urgency, title_cut, status_to_str, description_length = term_width - 32 ); // gross hardcode
            }
        }
    }
    // ---
    fn show_task(&mut self, id: usize) {
        if self.verify_id(id) {
            println!(
                " -{}- {} --- urgency: {:.3}",
                id, self.tasks[id].title, self.tasks[id].urgency
            );
            println!("  {}", self.tasks[id].description);
            let format = StrftimeItems::new("%H:%M, %d/%m/%Y");
            let formatted_start_time = self.tasks[id].start_time.unwrap().format_with_items(format);
            match self.tasks[id].due_time {
                Some(_) => {
                    let format = StrftimeItems::new("%H:%M, %d/%m/%Y");
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
    // Crash if task.json in XDG_app_data/task/task.json doesnt exist
    let mut task_manager = match TaskManager::load_from_file(&app_data_dir) {
        Ok(contents) => contents,
        Err(_) => TaskManager::new(),
    };

    task_manager.calculate_urgencies();
    task_manager.sort_by_urgencies();

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
                task_manager.set_partial_due_date(task_manager.tasks.len() - 1, date_str);
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
                task_manager.set_partial_due_date(id, date_str);
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
