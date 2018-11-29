use rustyline::{Editor, error::ReadlineError};
use std::path::PathBuf;
use log::error;

mod parser;
mod db;

use self::db::{Db, DbState, Password};

pub struct Wallet {
    prompt: String,
    db: Db
}

impl Wallet {
    pub fn new(home: PathBuf) -> Wallet {
        let db = Db::new(home.join("db"));
        let prompt = (if db.state() == DbState::Locked {
            "locked>> "
        } else {
            "new>> "
        }).to_owned();
        Wallet {
            db,
            prompt
        }
    }

    pub fn start(mut self) {
        let mut rl = Editor::<()>::new();
        loop {
            let readline = rl.readline(&self.prompt);
            match readline {
                Ok(line) => {
                    if line.is_empty() { continue }
                    let mut args = parser::parse_line(&line);

                    match self.process_line(&mut args) {
                        Ok(store_history) => {
                            if store_history {
                                rl.add_history_entry(line.as_ref());
                            }
                        },
                        Err(s) => {
                            error!("{}", s);
                        }
                    }

                    for a in args {
                        sodiumoxide::utils::memzero(&mut a.into_bytes());
                    }
                    sodiumoxide::utils::memzero(&mut line.into_bytes());
                },
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("Closing walllet...");
                    break
                },
                Err(err) => {
                    error!("Error reading input: {:?}", err);
                    break
                }
            }
        }
    }

    fn process_line(&mut self, args: &mut Vec<String>) -> Result<bool, String> {
        if args.len() == 0 { return Ok(false) }
        match &*args[0] {
            "new" => {
                if args.len() != 2 {
                    return Err("Missing password arg or too many args supplied".to_owned())
                }

                let state = self.db.state();
                if state != DbState::New {
                    if state == DbState::Locked {
                        return Err("Use unlock to use the existing wallet".to_owned())
                    } else if state == DbState::Unlocked {
                        return Err("Existing wallet already unlocked".to_owned())
                    } else {
                        return Err(format!("Unknown state: {:?}", state))
                    }
                }

                let pass = &Password(args.remove(1).into_bytes());
                self.db.set_password(pass);
                self.prompt = "locked>> ".to_owned();
                return Ok(false)
            },
            "unlock" => {
                if args.len() != 2 {
                    return Err("Missing password arg or too many args supplied".to_owned())
                }

                let state = self.db.state();
                if state != DbState::Locked {
                    if state == DbState::New {
                        return Err("A wallet has not yet been created, use new to create one".to_owned())
                    } else if state == DbState::Unlocked {
                        return Err("Wallet already unlocked".to_owned())
                    } else {
                        return Err(format!("Unknown state: {:?}", state))
                    }
                }

                let pass = &Password(args.remove(1).into_bytes());
                return if self.db.unlock(pass) {
                    self.prompt = "unlocked>> ".to_owned();
                    Ok(false)
                } else {
                    return Err("Failed to unlock wallet...incorrect password".to_owned())
                }
            },
            "help" => {
                Self::print_usage("Displaying help...");
            },
            _ => {
                Self::print_usage(&format!("Invalid command: {}", args[0]));
            }
        }
        Ok(true)
    }

    fn print_usage(header: &str) {
        let mut cmds = Vec::new();
        cmds.push(["help", "Displays this help menu"]);
        cmds.push(["new", "Creates a new wallet"]);
        cmds.push(["unlock", "Unlocks an existing wallet"]);

        let mut max_len = 0;
        for cmd in &cmds {
            assert!(cmd.len() == 2);
            let cmd_len = cmd[0].len();
            if cmd_len > max_len { max_len = cmd_len; }
        }

        println!("{}\n", header);
        for cmd in &cmds {
            let mut c = cmd[0].to_owned();
            if c.len() < max_len {
                for _ in 0 .. max_len - c.len() { c.push(' '); }
            }
            println!("  {}  {}", c, cmd[1]);
        }
        println!("");
    }
}
