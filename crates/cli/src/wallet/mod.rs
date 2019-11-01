use rustyline::{error::ReadlineError, Editor};
use std::path::PathBuf;
use url::Url;

mod cmd;
mod db;
mod parser;
mod script_builder;

use self::db::{Db, DbState};

pub struct Wallet {
    prompt: String,
    url: Url,
    db: Db,
    // Current ID to be sent when making requests
    req_id: u32,
}

impl Wallet {
    pub fn new(home: PathBuf) -> Wallet {
        let db = Db::new(home.join("wallet_db"));
        let prompt = (if db.state() == DbState::Locked {
            "locked>> "
        } else {
            "new>> "
        })
        .to_owned();

        let mut url: Url = "ws://localhost:7777".parse().unwrap();
        if url.host_str().is_none() {
            panic!("Expected url to have host");
        }
        if url.port().is_none() {
            url.set_port(Some(7777)).unwrap();
        }

        Wallet {
            db,
            prompt,
            url,
            req_id: 0,
        }
    }

    pub fn start(mut self) {
        let mut rl = Editor::<()>::new();
        loop {
            let readline = rl.readline(&self.prompt);
            match readline {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }
                    let mut args = parser::parse_line(&line);
                    let (store_history, err_msg) = self.process_line(&mut args);
                    if store_history {
                        rl.add_history_entry(line);
                    } else {
                        sodiumoxide::utils::memzero(&mut line.into_bytes());
                    }

                    for a in args {
                        sodiumoxide::utils::memzero(&mut a.into_bytes());
                    }

                    if let Err(msg) = err_msg {
                        println!("{}", msg);
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("Closing wallet...");
                    break;
                }
                Err(err) => {
                    println!("Error reading input: {:?}", err);
                    break;
                }
            }
        }
    }

    fn process_line(&mut self, args: &mut Vec<String>) -> (bool, Result<(), String>) {
        if args.is_empty() {
            return (false, Ok(()));
        }
        println!();
        match &*args[0] {
            "new" => (false, cmd::create_wallet(self, args)),
            "unlock" => (false, cmd::unlock(self, args)),
            "create_account" => (true, cmd::account::create(self, args)),
            "import_account" => (true, cmd::account::import(self, args)),
            "delete_account" => (true, cmd::account::delete(self, args)),
            "list_accounts" => (true, cmd::account::list(self, args)),
            "get_account" => (true, cmd::account::get(self, args)),
            "get_addr_info" => (true, cmd::account::get_addr_info(self, args)),
            "build_script" => (true, cmd::build_script(self, args)),
            "check_script_size" => (true, cmd::check_script_size(self, args)),
            "script_to_p2sh" => (true, cmd::script_to_p2sh(self, args)),
            "decode_tx" => (true, cmd::decode_tx(self, args)),
            "sign_tx" => (true, cmd::sign_tx(self, args)),
            "unsign_tx" => (true, cmd::unsign_tx(self, args)),
            "broadcast" => (true, cmd::broadcast(self, args)),
            "build_mint_tx" => (true, cmd::build_mint_tx(self, args)),
            "build_transfer_tx" => (true, cmd::build_transfer_tx(self, args)),
            "get_properties" => (true, cmd::get_properties(self, args)),
            "get_block" => (true, cmd::get_block(self, args)),
            "help" => {
                Self::print_usage("Displaying help...");
                (true, Ok(()))
            }
            _ => {
                Self::print_usage(&format!("Invalid command: {}", args[0]));
                (true, Ok(()))
            }
        }
    }

    fn print_usage(header: &str) {
        let mut cmds = Vec::<[&str; 2]>::new();
        cmds.push(["help", "Display this help menu"]);
        cmds.push(["new <password>", "Create a new wallet"]);
        cmds.push(["unlock <password>", "Unlock an existing wallet"]);
        cmds.push(["create_account <account>", "Create an account"]);
        cmds.push(["import_account <account> <wif>", "Import an account"]);
        cmds.push(["delete_account <account>", "Delete an existing account"]);
        cmds.push(["list_accounts", "List all accounts"]);
        cmds.push([
            "get_account <account>",
            "Retrieve account keys and addresses",
        ]);
        cmds.push([
            "get_addr_info <account|p2sh>",
            "Retrieve account or P2SH address information",
        ]);
        cmds.push(["build_script <...op>", "Builds a script"]);
        cmds.push([
            "check_script_size <raw_hex>",
            "Checks if the script is too large and prints the size in bytes",
        ]);
        cmds.push([
            "script_to_p2sh <raw_hex>",
            "Converts a script to a payable P2SH address",
        ]);
        cmds.push([
            "decode_tx <tx_hex>",
            "Decodes a transaction and prints it to console",
        ]);
        cmds.push(["sign_tx <tx_hex> <account...>", "Signs a raw transaction"]);
        cmds.push([
            "unsign_tx <sig_pos> <tx_hex>",
            "Removes a signature from a raw transaction",
        ]);
        cmds.push([
            "broadcast <tx_hex>",
            "Broadcasts a transaction to the network",
        ]);
        cmds.push([
            "build_mint_tx <ts_offset> <mark_asset> <owner_script> <attachment_path>",
            "Builds a mint transaction",
        ]);
        cmds.push([
            "build_transfer_tx <ts_offset> <from:script_hex> <to:p2sh> <amount:mark_asset> <fee:mark_asset> <memo>",
            "Builds a transfer transaction",
        ]);
        cmds.push(["get_properties", "Retrieve global network properties"]);
        cmds.push(["get_block <height>", "Retrieve a block from the network"]);

        let mut max_len = 0;
        for cmd in &cmds {
            assert_eq!(cmd.len(), 2);
            let cmd_len = cmd[0].len();
            if cmd_len > max_len {
                max_len = cmd_len;
            }
        }

        println!("{}\n", header);
        for cmd in &cmds {
            let mut c = cmd[0].to_owned();
            if c.len() < max_len {
                for _ in 0..max_len - c.len() {
                    c.push(' ');
                }
            }
            println!("  {}  {}", c, cmd[1]);
        }
        println!();
    }
}
