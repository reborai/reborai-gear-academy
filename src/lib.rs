#![no_std]
#![allow(static_mut_refs)]
use gstd::{exec, msg, prelude::*};
use pebbles_game_io::*;

static mut GAME_STATE: Option<GameState> = None;

#[cfg(test)]
fn get_random_u32() -> u32 {
    2
}

#[cfg(not(test))]
fn get_random_u32() -> u32 {
    let seed = msg::id();
    let (hash, _) = exec::random(seed.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

fn get_game_state() -> &'static mut GameState {
    unsafe {
        let game_state = GAME_STATE.as_mut().expect("GAME_STATE isn't initialized");
        game_state
    }
}

fn turn_if_first_player_is_program() {
    let game_state = get_game_state();
    if game_state.first_player == Player::Program {
        let random_count = get_random_u32() % get_game_state().max_pebbles_per_turn + 1;
        let count = match game_state.difficulty {
            DifficultyLevel::Easy => random_count,
            DifficultyLevel::Hard => {
                if game_state.pebbles_remaining % (game_state.max_pebbles_per_turn + 1) == 0 {
                    random_count
                } else {
                    game_state.pebbles_remaining % game_state.max_pebbles_per_turn
                }
            }
        };
        turn(count);
    }
}

fn turn(count: u32) {
    let game_state = get_game_state();
    let count = count.clamp(1, game_state.max_pebbles_per_turn);
    game_state.pebbles_remaining = game_state.pebbles_remaining.saturating_sub(count);

    msg::send(
        msg::source(),
        PebblesEvent::CounterTurn(game_state.first_player, count),
        0,
    )
    .expect("Unable to send");

    gstd::debug!(
        "After send: state={:?}",
        PebblesEvent::CounterTurn(game_state.first_player, count)
    );

    if game_state.pebbles_remaining == 0 {
        game_state.winner = Some(game_state.first_player);
    } else {
        game_state.first_player = match game_state.first_player {
            Player::User => Player::Program,
            Player::Program => Player::User,
        };
    }

    gstd::debug!("After turn: state={:?}", get_game_state());
}

fn restart(pebbles_init: PebblesInit) {
    let random_num = get_random_u32();
    let game_state = GameState {
        pebbles_count: pebbles_init.pebbles_count,
        max_pebbles_per_turn: pebbles_init.max_pebbles_per_turn,
        pebbles_remaining: pebbles_init.pebbles_count,
        difficulty: pebbles_init.difficulty,
        first_player: if random_num % 2 == 0 {
            Player::User
        } else {
            Player::Program
        },
        winner: None,
    };
    unsafe {
        GAME_STATE = Some(game_state);
    }
    gstd::debug!(
        "Restart state: {:?}, random_num = {random_num}",
        get_game_state()
    );
    turn_if_first_player_is_program();
}

#[no_mangle]
unsafe extern "C" fn init() {
    let pebbles_init = msg::load::<PebblesInit>().expect("Failed to load PebblesInit");
    restart(pebbles_init);
}

#[no_mangle]
unsafe extern "C" fn handle() {
    let pebbles_action = msg::load::<PebblesAction>().expect("Failed to load PebblesAction");
    let game_state = get_game_state();
    match pebbles_action {
        PebblesAction::Turn(count) => {
            if game_state.winner.is_some() {
                msg::reply(PebblesEvent::Won(game_state.winner.unwrap()), 0)
                    .expect("Unable to reply");
            } else {
                turn(count);
                turn_if_first_player_is_program();
                msg::reply("turned", 0).expect("Unable to send");
            }
        }
        PebblesAction::GiveUp => {
            if game_state.winner.is_some() {
                msg::reply(PebblesEvent::Won(game_state.winner.unwrap()), 0)
                    .expect("Unable to send");
            } else {
                game_state.winner = Some(Player::Program);
                msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to send");
            }
        }
        PebblesAction::Restart(pebbles_init) => restart(pebbles_init),
    }
}

#[no_mangle]
unsafe extern "C" fn state() {
    let game_state = get_game_state();
    gstd::debug!("State: {game_state:?}");
    msg::reply(game_state, 0).expect("Unable to share the state");
}
