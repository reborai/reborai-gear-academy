#[cfg(test)]
mod tests {
    use gtest::{Program, System};
    use pebbles_game_io::*;
    const USER_OWNER: u64 = 42;
    const PROGRAM_OWNER: u64 = 46;
    const PEBBLES_INIT: PebblesInit = PebblesInit {
        pebbles_count: 15,
        max_pebbles_per_turn: 4,
        difficulty: DifficultyLevel::Easy,
    };

    #[test]
    fn program_test() {
        let system = System::new();
        system.init_logger();
        let program = Program::current(&system);
        assert_eq!(program.id(), 1.into());

        let init = PEBBLES_INIT.clone();
        system.mint_to(USER_OWNER, 79888888888898777777777);
        program.send(USER_OWNER, init);
        system.run_next_block();
        let game_state: GameState = program.read_state(b"").unwrap();
        assert_eq!(game_state.pebbles_count, PEBBLES_INIT.pebbles_count);
    }

    #[test]
    fn win_test() {
        let system = System::new();
        system.init_logger();
        let program = Program::current(&system);
        system.mint_to(USER_OWNER, 79888888888898777777777);
        let _ = program.send(USER_OWNER, PEBBLES_INIT.clone());
        system.run_next_block();
        program.send(USER_OWNER, PebblesAction::Turn(4));
        system.run_next_block();
        program.send(USER_OWNER, PebblesAction::Turn(4));
        system.run_next_block();
        program.send(USER_OWNER, PebblesAction::Turn(4));
        system.run_next_block();
        program.send(USER_OWNER, PebblesAction::Turn(4));
        system.run_next_block();
        let state: GameState = program.read_state("").expect("not msg");
        assert_eq!(state.winner, Some(Player::Program));
    }

    #[test]
    fn give_up() {
        let system = System::new();
        system.init_logger();
        let program = Program::current(&system);
        system.mint_to(PROGRAM_OWNER, 10000000000000045);
        let _ = program.send(PROGRAM_OWNER, PEBBLES_INIT.clone());
        system.run_next_block();
        program.send(PROGRAM_OWNER, PebblesAction::GiveUp);
        system.run_next_block();
        let state: GameState = program.read_state("").expect("not msg");
        assert_eq!(state.winner, Some(Player::Program));
    }

    #[test]
    fn restart() {
        let system = System::new();
        system.init_logger();
        let program = Program::current(&system);
        system.mint_to(PROGRAM_OWNER, 10000000000000045);
        let _ = program.send(PROGRAM_OWNER, PEBBLES_INIT.clone());
        system.run_next_block();
        let _ = program.send(PROGRAM_OWNER, PebblesAction::Restart(PEBBLES_INIT));
        system.run_next_block();
        let game_state: GameState = program.read_state(b"").unwrap();
        assert_eq!(game_state.pebbles_count, PEBBLES_INIT.pebbles_count);
        assert_eq!(
            game_state.max_pebbles_per_turn,
            PEBBLES_INIT.max_pebbles_per_turn
        );
        assert_eq!(game_state.difficulty, PEBBLES_INIT.difficulty);
        assert_eq!(game_state.pebbles_remaining, game_state.pebbles_count);
    }
}
