#[macro_use] extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Player {
    id: u64,
    name: String,
    score: u64,
    secret_number: u64,  // Added secret_number field
    attempts_left: u64,  // Added attempts_left field
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct PublicPlayer {
    id: u64,
    name: String,
    score: u64,
    attempts_left: u64,
    //created_at: u64,
    //updated_at: Option<u64>,
    clue: String,  // Added clue field
}

impl Storable for Player {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Player {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            0
        ).expect("Cannot create a counter")
    );
    static STORAGE: RefCell<StableBTreeMap<u64, Player, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct PlayerPayload {
    name: String,
}

#[ic_cdk::query]
fn get_player(id: u64) -> Result<PublicPlayer, Error> {
    match _get_player(&id) {
        Some(player) => {
            // Return only non-sensitive information
            Ok(PublicPlayer {
                id: player.id,
                name: player.name,
                score: player.score,
                attempts_left: player.attempts_left,
                clue: "".to_string(), // No clue for get_player query
            })
        }
        None => Err(Error::NotFound {
            msg: format!("A player with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_player(payload: PlayerPayload) -> Option<PublicPlayer> {
    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    }).expect("cannot increment id counter");

    let player = Player {
        id,
        name: payload.name,
        score: 0, // Initialize score to 0
        secret_number: 0,  // Initialize secret_number
        attempts_left: 7,  // Set attempts_left to 5
        created_at: time(),
        updated_at: None,
    };

    do_insert(&player);

    Some(PublicPlayer {
        id: player.id,
        name: player.name,
        score: player.score,
        attempts_left: player.attempts_left,
        clue: "".to_string(), // No clue when player is added
    })
}


#[ic_cdk::update]
fn update_score(id: u64, score: u64) -> Result<PublicPlayer, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut player) => {
            player.score = score;
            player.updated_at = Some(time());
            do_insert(&player);

            // Return a PublicPlayer object without exposing sensitive fields
            Ok(PublicPlayer {
                id: player.id,
                name: player.name.clone(),
                score: player.score,
                attempts_left: player.attempts_left,
                clue: "".to_string()
            })
        }
        None => Err(Error::NotFound {
            msg: format!("Could not update score for player with id={}. Player not found.", id),
        }),
    }
}


#[ic_cdk::update]
fn delete_player(id: u64) -> Result<Player, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(player) => Ok(player),
        None => Err(Error::NotFound {
            msg: format!("Could not delete player with id={}. Player not found.", id),
        }),
    }
}

// Helper method to perform insert
fn do_insert(player: &Player) {
    STORAGE.with(|service| service.borrow_mut().insert(player.id, player.clone()));
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// A helper method to get a player by id. Used in get_player/update_score
fn _get_player(id: &u64) -> Option<Player> {
    STORAGE.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn play_game(id: u64, guess: u64) -> Result<PublicPlayer, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut player) => {
            // Initialize the secret number and attempts left if not set
            if player.secret_number == 0 {
                player.secret_number = time() as u64 % 101; // Set a random secret number between 0 and 100
                player.attempts_left = 7;
            }

            if player.attempts_left == 0 {
                player.updated_at = Some(time());
                player.attempts_left = 7; // Reset attempts for a new game
                player.secret_number = time() as u64 % 101; // Set a new secret number
                return Err(Error::NotFound {
                    msg: format!("Game over! Player id={} has no more attempts.", id),
                });
            }

            let clue_msg: &str; // Declare the clue message variable
            if guess == player.secret_number {
                player.score += 10;
                player.updated_at = Some(time());
                player.attempts_left = 7; // Reset attempts for a new game
                player.secret_number = time() as u64 % 101; // Set a new secret number
                clue_msg = "Congratulations! You've guessed the number! A new game has started.";
            } else {
                player.attempts_left -= 1;
                clue_msg = if guess < player.secret_number {
                    "The secret number is higher than your guess."
                } else {
                    "The secret number is lower than your guess."
                };
            }

            // Update player data in storage
            do_insert(&player);

            // Return only non-sensitive information with the clue
            Ok(PublicPlayer {
                id: player.id,
                name: player.name,
                score: player.score,
                attempts_left: player.attempts_left,
                clue: clue_msg.to_string(), // Include clue in the response
            })
        }
        None => {
            // If the player is not found, return an error
            Err(Error::NotFound {
                msg: format!(
                    "Player with id={} not found. No clue available.",
                    id
                ),
            })
        }
    }
}



ic_cdk::export_candid!();
