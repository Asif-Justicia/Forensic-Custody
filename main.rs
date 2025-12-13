use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};
use std::time::{SystemTime, UNIX_EPOCH};

struct Evidence {
    id: String,
    hash: String,
    current_custodian: String,
    history: Vec<String>,
}

fn get_current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// ---------------- Core Functions ----------------
fn register_evidence(evidences: &mut Vec<Evidence>) {
    let mut input = String::new();
    print!("Enter Evidence ID: ");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    let id = input.trim().to_string();

    input.clear();
    print!("Enter Evidence Hash: ");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    let hash = input.trim().to_string();

    input.clear();
    print!("Enter Initial Custodian ID: ");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    let custodian = input.trim().to_string();

    evidences.push(Evidence {
        id,
        hash,
        current_custodian: custodian,
        history: vec![],
    });

    println!("\nEvidence registered successfully!");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

fn transfer_custody(evidences: &mut Vec<Evidence>) {
    if evidences.is_empty() {
        println!("No evidence registered yet!");
        std::io::stdin().read_line(&mut String::new()).unwrap();
        return;
    }

    let mut input = String::new();
    print!("Enter Evidence ID to transfer: ");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    let id = input.trim();

    if let Some(e) = evidences.iter_mut().find(|e| e.id == id) {
        input.clear();
        print!("Enter new custodian ID: ");
        stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let to = input.trim().to_string();

        e.history.push(format!(
            "Transfer from {} to {} at {}",
            e.current_custodian,
            to,
            get_current_timestamp()
        ));
        e.current_custodian = to;

        println!("Custody transferred successfully!");
        println!("Press Enter to continue...");
        std::io::stdin().read_line(&mut String::new()).unwrap();
    } else {
        println!("Evidence ID not found!");
        println!("Press Enter to continue...");
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}

fn view_history(evidences: &Vec<Evidence>) {
    if evidences.is_empty() {
        println!("No evidence registered yet!");
        std::io::stdin().read_line(&mut String::new()).unwrap();
        return;
    }

    let mut input = String::new();
    print!("Enter Evidence ID to view history: ");
    stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    let id = input.trim();

    if let Some(e) = evidences.iter().find(|e| e.id == id) {
        println!("\n=== Evidence History ===");
        println!("Current Custodian: {}", e.current_custodian);
        for h in &e.history {
            println!("{}", h);
        }
        println!("Press Enter to continue...");
        std::io::stdin().read_line(&mut String::new()).unwrap();
    } else {
        println!("Evidence ID not found!");
        println!("Press Enter to continue...");
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}

// ---------------- Retro Arrow Menu ----------------
fn main() -> crossterm::Result<()> {
    let mut evidences: Vec<Evidence> = vec![];

    let menu_items = vec![
        "Register new evidence",
        "Transfer custody",
        "View evidence history",
        "Exit",
    ];

    loop {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        let mut selected = 0;

        loop {
            execute!(stdout(), cursor::MoveTo(0, 0))?;
            println!("🎮 Digital Evidence Retro Menu 🎮\n");
            for (i, item) in menu_items.iter().enumerate() {
                if i == selected {
                    execute!(
                        stdout(),
                        SetBackgroundColor(Color::Blue),
                        SetForegroundColor(Color::White),
                        Print(format!("> {}\n", item)),
                        ResetColor
                    )?;
                } else {
                    println!("  {}", item);
                }
            }

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode()?;

        match selected {
            0 => register_evidence(&mut evidences),
            1 => transfer_custody(&mut evidences),
            2 => view_history(&evidences),
            3 => {
                println!("Exiting...");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
