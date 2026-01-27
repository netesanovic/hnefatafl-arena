use crate::bot::Bot;
use crate::game::{GameResult, GameState, Player, Variant};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct MatchConfig {
    pub time_per_move: Duration,
    pub max_moves: usize,
    pub enable_pondering: bool,
}

impl Default for MatchConfig {
    fn default() -> Self {
        MatchConfig {
            time_per_move: Duration::from_secs(5),
            max_moves: 200,
            enable_pondering: true,
        }
    }
}

pub enum MatchResult {
    AttackersWin { winner_name: String, moves: usize },
    DefendersWin { winner_name: String, moves: usize },
    Draw { moves: usize },
    Timeout { violator: String, winner: String },
    IllegalMove { violator: String, winner: String },
}

impl MatchResult {
    pub fn winner(&self) -> Option<&str> {
        match self {
            MatchResult::AttackersWin { winner_name, .. } => Some(winner_name),
            MatchResult::DefendersWin { winner_name, .. } => Some(winner_name),
            MatchResult::Timeout { winner, .. } => Some(winner),
            MatchResult::IllegalMove { winner, .. } => Some(winner),
            MatchResult::Draw { .. } => None,
        }
    }
}

pub struct Match {
    config: MatchConfig,
    state: GameState,
    attacker_bot: Box<dyn Bot>,
    defender_bot: Box<dyn Bot>,
    verbose: bool,
}

impl Match {
    pub fn new(
        attacker_bot: Box<dyn Bot>,
        defender_bot: Box<dyn Bot>,
        config: MatchConfig,
        verbose: bool,
    ) -> Self {
        Self::with_variant(
            attacker_bot,
            defender_bot,
            config,
            verbose,
            Variant::Brandubh,
        )
    }

    pub fn with_variant(
        attacker_bot: Box<dyn Bot>,
        defender_bot: Box<dyn Bot>,
        config: MatchConfig,
        verbose: bool,
        variant: Variant,
    ) -> Self {
        Match {
            config,
            state: GameState::new(variant),
            attacker_bot,
            defender_bot,
            verbose,
        }
    }

    pub fn play(&mut self) -> MatchResult {
        // Notify bots that game is starting
        self.attacker_bot.game_start(Player::Attackers);
        self.defender_bot.game_start(Player::Defenders);

        if self.verbose {
            println!("Match starting:");
            println!("  Attackers: {}", self.attacker_bot.name());
            println!("  Defenders: {}", self.defender_bot.name());
            if self.config.enable_pondering {
                println!("  Pondering: ENABLED");
            }
            println!("\nInitial board:");
            println!("{}", self.state.display_board());
        }

        while !self.state.is_game_over() && self.state.move_count() < self.config.max_moves {
            let current_player = self.state.current_player();
            let result = if self.config.enable_pondering {
                self.play_move_with_pondering(current_player)
            } else {
                self.play_move_without_pondering(current_player)
            };

            if let Some(result) = result {
                return result;
            }
        }

        // Game ended normally
        self.attacker_bot.game_end();
        self.defender_bot.game_end();

        let moves = self.state.move_count();

        if let Some(result) = self.state.result() {
            match result {
                GameResult::AttackersWin => {
                    if self.verbose {
                        println!("\n{} wins as Attackers!", self.attacker_bot.name());
                    }
                    MatchResult::AttackersWin {
                        winner_name: self.attacker_bot.name().to_string(),
                        moves,
                    }
                }
                GameResult::DefendersWin => {
                    if self.verbose {
                        println!("\n{} wins as Defenders!", self.defender_bot.name());
                    }
                    MatchResult::DefendersWin {
                        winner_name: self.defender_bot.name().to_string(),
                        moves,
                    }
                }
                GameResult::Draw => {
                    if self.verbose {
                        println!("\nGame is a draw!");
                    }
                    MatchResult::Draw { moves }
                }
            }
        } else {
            // Max moves reached
            if self.verbose {
                println!("\nMax moves ({}) reached - Draw!", self.config.max_moves);
            }
            MatchResult::Draw { moves }
        }
    }

    fn play_move_without_pondering(&mut self, current_player: Player) -> Option<MatchResult> {
        let bot = match current_player {
            Player::Attackers => &mut self.attacker_bot,
            Player::Defenders => &mut self.defender_bot,
        };

        if self.verbose {
            println!(
                "\nMove {}: {} to play",
                self.state.move_count() + 1,
                bot.name()
            );
            println!("Legal moves: {}", self.state.legal_moves().len());
        }

        // Get move from bot with time limit
        let start = Instant::now();
        let mv = bot.get_move(&self.state, self.config.time_per_move);
        let elapsed = start.elapsed();

        self.handle_move_result(mv, elapsed, current_player)
    }

    fn play_move_with_pondering(&mut self, current_player: Player) -> Option<MatchResult> {
        // Get references to both bots
        let (active_bot, pondering_bot) = match current_player {
            Player::Attackers => (&mut self.attacker_bot, &mut self.defender_bot),
            Player::Defenders => (&mut self.defender_bot, &mut self.attacker_bot),
        };

        if self.verbose {
            println!(
                "\nMove {}: {} to play (opponent pondering)",
                self.state.move_count() + 1,
                active_bot.name()
            );
            println!("Legal moves: {}", self.state.legal_moves().len());
        }

        // Start pondering in the opponent bot
        let state_for_pondering = self.state.clone();
        let pondering_active = Arc::new(Mutex::new(true));
        let _pondering_active_clone = Arc::clone(&pondering_active);

        // We can't easily move the bot into a thread due to ownership
        // Instead, call opponent_thinking on the pondering bot in the main thread periodically
        // A better implementation would use message passing or channels

        // For now, just call it once before getting the move
        pondering_bot.opponent_thinking(&state_for_pondering);

        // Get move from active bot with time limit
        let start = Instant::now();
        let mv = active_bot.get_move(&self.state, self.config.time_per_move);
        let elapsed = start.elapsed();

        // Stop pondering
        pondering_bot.stop_pondering();

        self.handle_move_result(mv, elapsed, current_player)
    }

    fn handle_move_result(
        &mut self,
        mv: Option<crate::game::Move>,
        elapsed: Duration,
        current_player: Player,
    ) -> Option<MatchResult> {
        let bot_name = match current_player {
            Player::Attackers => self.attacker_bot.name(),
            Player::Defenders => self.defender_bot.name(),
        };

        // Check timeout
        if elapsed > self.config.time_per_move {
            let violator = bot_name.to_string();
            let winner = match current_player {
                Player::Attackers => self.defender_bot.name().to_string(),
                Player::Defenders => self.attacker_bot.name().to_string(),
            };

            if self.verbose {
                println!(
                    "TIMEOUT: {} took {:?} (limit: {:?})",
                    violator, elapsed, self.config.time_per_move
                );
            }

            return Some(MatchResult::Timeout { violator, winner });
        }

        // Check if bot returned a move
        let mv = match mv {
            Some(m) => m,
            None => {
                // No legal moves or bot gave up
                if self.verbose {
                    println!("{} returned no move", bot_name);
                }

                return Some(MatchResult::Draw {
                    moves: self.state.move_count(),
                });
            }
        };

        if self.verbose {
            println!("{} plays: {} (took {:?})", bot_name, mv, elapsed);
        }

        // Make the move
        if let Err(e) = self.state.make_move(mv) {
            let violator = bot_name.to_string();
            let winner = match current_player {
                Player::Attackers => self.defender_bot.name().to_string(),
                Player::Defenders => self.attacker_bot.name().to_string(),
            };

            if self.verbose {
                println!("ILLEGAL MOVE: {} - {}", violator, e);
            }

            return Some(MatchResult::IllegalMove { violator, winner });
        }

        // Notify both bots of the move
        self.attacker_bot.notify_move(mv);
        self.defender_bot.notify_move(mv);

        if self.verbose {
            println!("{}", self.state.display_board());
        }

        None
    }
}

pub struct Tournament {
    bots: Vec<(String, Box<dyn Bot>)>,
    #[allow(dead_code)]
    config: MatchConfig,
    verbose: bool,
}

impl Tournament {
    pub fn new(config: MatchConfig, verbose: bool) -> Self {
        Tournament {
            bots: Vec::new(),
            config,
            verbose,
        }
    }

    pub fn add_bot(&mut self, name: String, bot: Box<dyn Bot>) {
        self.bots.push((name, bot));
    }

    pub fn run_round_robin(&mut self) -> TournamentResults {
        let mut results = TournamentResults::new();

        for i in 0..self.bots.len() {
            for j in (i + 1)..self.bots.len() {
                // Play two games: each bot plays as both attacker and defender
                if self.verbose {
                    println!("\n{}", "=".repeat(60));
                    println!("Match: {} vs {}", self.bots[i].0, self.bots[j].0);
                    println!("{}", "=".repeat(60));
                }

                // Game 1: i as attackers, j as defenders
                // Note: We can't move bots out, so this is a simplified version
                // In a real implementation, you'd need to handle bot ownership differently

                // For now, just record that matches would happen
                if self.verbose {
                    println!(
                        "Game 1: {} (Attackers) vs {} (Defenders)",
                        self.bots[i].0, self.bots[j].0
                    );
                    println!(
                        "Game 2: {} (Attackers) vs {} (Defenders)",
                        self.bots[j].0, self.bots[i].0
                    );
                }

                results.add_matchup(self.bots[i].0.clone(), self.bots[j].0.clone());
            }
        }

        results
    }
}

#[derive(Debug)]
pub struct TournamentResults {
    matchups: Vec<(String, String)>,
}

impl TournamentResults {
    pub fn new() -> Self {
        TournamentResults {
            matchups: Vec::new(),
        }
    }

    pub fn add_matchup(&mut self, bot1: String, bot2: String) {
        self.matchups.push((bot1, bot2));
    }

    pub fn display(&self) {
        println!("\nTournament Results:");
        println!("==================");
        for (bot1, bot2) in &self.matchups {
            println!("{} vs {}", bot1, bot2);
        }
    }
}

impl Default for TournamentResults {
    fn default() -> Self {
        Self::new()
    }
}
