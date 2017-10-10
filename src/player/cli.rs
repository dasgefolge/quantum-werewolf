use std::{fmt, io};

use console::Term;

use dialoguer;

use game::Party;
use player::Player;

/// A command-lind interface shared by multiple players
pub struct Cli<'a> {
    term: &'a Term,
    /// The player who last performed private communication with the game, or `None` if the last communication was public.
    active_player: Option<&'a CliPlayer<'a>>
}

impl<'a, 'b: 'a> From<&'b Term> for Cli<'a> {
    fn from(term: &'b Term) -> Cli<'a> {
        Cli { term, active_player: None }
    }
}

impl<'a> fmt::Debug for Cli<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cli {{ term: _, active_player: {:?} }}", self.active_player)
    }
}

/// A player who sends game actions via the command line.
#[derive(Debug)]
pub struct CliPlayer<'a> {
    cli: &'a Cli<'a>,
    name: String
}

impl<'a> CliPlayer<'a> {
    /// Creates a new CLI player with the given player name.
    pub fn new(name: String, cli: &'a Cli) -> CliPlayer<'a> {
        CliPlayer { cli, name }
    }

    fn choice_secret(&self, msg: &str, mut options: Vec<String>) -> io::Result<String> {
        self.move_terminal();
        self.cli.term.write_line(msg)?;
        let idx = {
            let option_slices = options.iter().map(String::as_ref).collect::<Vec<_>>();
            dialoguer::Select::new().items(&option_slices).interact()?
        };
        Ok(options.swap_remove(idx))
    }

    //fn input_secret(&self, msg: &str) -> String {
    //    self.move_terminal();
    //    print!("[ ?? ] @{}: {}: ", self.name, msg);
    //    stdout().flush().expect("failed to flush stdout");
    //    let mut name = String::new();
    //    stdin().read_line(&mut name).expect("failed to read player input");
    //    assert_eq!(name.pop(), Some('\n'));
    //    name
    //}

    fn move_terminal(&self) {
        //TODO confirm terminal move
    }

    fn print_secret(&self, msg: &str) -> io::Result<()> {
        self.move_terminal();
        println!("secret message for {}:", self.name);
        println!("{}", msg);
        print!("[ ** ] press any key to hide this message");
        self.cli.term.read_key()?;
        self.cli.term.clear_last_lines(3)?;
        Ok(())
    }
}

impl<'a> Player for CliPlayer<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn recv_id(&self, player_id: usize) {
        self.print_secret(&format!("your secret player ID is {}", player_id)[..]).expect("failed to relay player ID");
    }

    fn choose_heal_target(&self, mut possible_targets: Vec<String>) -> Option<String> {
        possible_targets.push("skip".into());
        let result = self.choice_secret("player to heal", possible_targets).expect("failed to get heal target");
        if result == "skip" {
            None
        } else {
            Some(result)
        }
    }

    fn choose_investigation_target(&self, mut possible_targets: Vec<String>) -> Option<String> {
        possible_targets.push("skip".into());
        let result = self.choice_secret("player to investigate", possible_targets).expect("failed to get investigation target");
        if result == "skip" {
            None
        } else {
            Some(result)
        }
    }

    fn recv_investigation(&self, player_name: &str, party: Party) {
        self.print_secret(&format!("{} investigated as {}", player_name, party)[..]).expect("failed to relay investigation result");
    }

    fn choose_werewolf_kill_target(&self, possible_targets: Vec<String>) -> String {
        self.choice_secret("player to werewolf-kill", possible_targets).expect("failed to get werewolf kill target")
    }

    fn recv_exile(&self, reason: &str) {
        let _ = self.print_secret(&format!("you have been exiled for {}", reason)[..]);
    }
}
