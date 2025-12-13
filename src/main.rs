use std::io::{self, Write, stdout};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use crossterm::{
    ExecutableCommand, cursor, terminal,
    event::{self, Event, KeyCode},
    style::{Color, SetForegroundColor, ResetColor, Stylize}
};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Role {
    Investigator,
    EvidenceOfficer,
    Analyst,
    Prosecutor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustodyEvent {
    from: Option<String>,
    to: String,
    timestamp: u64,
    action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Evidence {
    id: String,
    hash: String,
    created_at: u64,
    current_custodian: String,
    history: Vec<CustodyEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: usize,
    previous_hash: String,
    evidence_id: String,
    timestamp: u64,
    hash: String,
}

fn pause_message(msg: &str) -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    println!("\n{}\nPress Enter to continue...", msg);
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

fn read_input(prompt: &str) -> io::Result<String> {
    crossterm::terminal::disable_raw_mode()?;
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    crossterm::terminal::enable_raw_mode()?;
    Ok(input.trim().to_string())
}

fn compute_hash(index: usize, previous_hash: &str, evidence_id: &str, timestamp: u64) -> String {
    let data = format!("{}{}{}{}", index, previous_hash, evidence_id, timestamp);
    format!("{:x}", Sha256::digest(data.as_bytes()))
}

fn register_evidence(evidence: &mut Vec<Evidence>, chain: &mut Vec<Block>, custodian: Role) -> io::Result<()> {
    let id = read_input("Enter Evidence ID: ")?;
    let id_clone = id.clone(); // clone for later use

    if evidence.iter().any(|e| e.id == id) {
        pause_message("Evidence ID already exists!")?;
        return Ok(());
    }

    let content = read_input("Enter Evidence Content: ")?;
    let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
    let timestamp = Utc::now().timestamp() as u64;

    let new_evidence = Evidence {
        id: id.clone(),
        hash: hash.clone(),
        created_at: timestamp,
        current_custodian: format!("{:?}", custodian),
        history: vec![CustodyEvent {
            from: None,
            to: format!("{:?}", custodian),
            timestamp,
            action: "Initial Handoff".to_string(),
        }],
    };

    evidence.push(new_evidence.clone());

    let previous_hash = if let Some(prev) = chain.last() { prev.hash.clone() } else { "0".repeat(64) };
    let block_hash = compute_hash(chain.len(), &previous_hash, &id, timestamp);

    chain.push(Block {
        index: chain.len(),
        previous_hash,
        evidence_id: id_clone.clone(),
        timestamp,
        hash: block_hash,
    });

    pause_message(&format!("Evidence {} registered successfully!", id_clone))?;
    Ok(())
}

fn transfer_custody(evidence: &mut Vec<Evidence>, _chain: &mut Vec<Block>, new_custodian: Role) -> io::Result<()> {
    let id = read_input("Enter Evidence ID to transfer: ")?;
    if let Some(e) = evidence.iter_mut().find(|ev| ev.id == id) {
        let timestamp = Utc::now().timestamp() as u64;
        let prev_custodian = e.current_custodian.clone();
        e.current_custodian = format!("{:?}", new_custodian);
        e.history.push(CustodyEvent {
            from: Some(prev_custodian),
            to: format!("{:?}", new_custodian),
            timestamp,
            action: "Transferred".to_string(),
        });

        pause_message(&format!("Custody of {} transferred to {:?}", id, new_custodian))?;
    } else {
        pause_message("Evidence not found!")?;
    }
    Ok(())
}

fn verify_chain(chain: &Vec<Block>) -> io::Result<()> {
    let mut valid = true;
    for i in 1..chain.len() {
        if chain[i].previous_hash != chain[i-1].hash {
            valid = false;
            break;
        }
    }
    if valid {
        pause_message("Blockchain verified: all good!")?;
    } else {
        pause_message("Blockchain verification failed!")?;
    }
    Ok(())
}

fn display_evidence(evidence_list: &Vec<Evidence>) -> io::Result<()> {
    let mut stdout = stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(cursor::MoveTo(0,0))?;

    println!("{}", "=== Registered Evidence ===".green());
    for e in evidence_list {
        println!("{} {}", "ID:".yellow(), e.id);
        println!("{} {}", "Hash:".yellow(), e.hash);
        println!("{} {}", "Current Custodian:".yellow(), e.current_custodian);
        println!("{} {}", "Created At:".yellow(), Utc.timestamp_opt(e.created_at as i64, 0).unwrap().to_rfc3339());
        println!("{}", "History:".yellow());
        for h in &e.history {
            println!("  {} -> {} [{}] - {}", 
                h.from.clone().unwrap_or("None".to_string()).cyan(),
                h.to.clone().cyan(),
                Utc.timestamp_opt(h.timestamp as i64, 0).unwrap().to_rfc3339(),
                h.action.clone().magenta()
            );
        }
        println!("{}", "-".repeat(50));
    }

    pause_message("End of Evidence List")?;
    Ok(())
}

fn main() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    let menu_items = vec!["Register Evidence", "Transfer Custody", "Verify Chain", "View Evidence", "Exit"];
    let mut selected = 0;

    let mut evidence_list: Vec<Evidence> = vec![];
    let mut blockchain: Vec<Block> = vec![];

    loop {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.execute(cursor::MoveTo(0,0))?;

        println!("{}", "=== Forensic Custody System ===".green());
        for (i, item) in menu_items.iter().enumerate() {
            if i == selected {
                stdout.execute(SetForegroundColor(Color::Yellow))?;
                println!("> {}", item);
                stdout.execute(ResetColor)?;
            } else {
                println!("  {}", item);
            }
        }

        match event::read()? {
            Event::Key(event) => match event.code {
                KeyCode::Up => { if selected > 0 { selected -= 1; } }
                KeyCode::Down => { if selected < menu_items.len() - 1 { selected += 1; } }
                KeyCode::Enter => {
                    match menu_items[selected] {
                        "Register Evidence" => { register_evidence(&mut evidence_list, &mut blockchain, Role::Investigator)?; }
                        "Transfer Custody" => { transfer_custody(&mut evidence_list, &mut blockchain, Role::EvidenceOfficer)?; }
                        "Verify Chain" => { verify_chain(&blockchain)?; }
                        "View Evidence" => { display_evidence(&evidence_list)?; }
                        "Exit" => break,
                        _ => {}
                    }
                }
                KeyCode::Char('q') => break,
                _ => {}
            },
            _ => {}
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
