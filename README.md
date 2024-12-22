# Guess the Number Game on IC

## Overview
This project is a smart contract-based "Guess the Number" game deployed on the Internet Computer (IC). Players can create accounts, play the game by guessing a random number, and earn points for correct guesses. The backend is written in Rust, leveraging stable storage structures for persistent data management.

## Features
- **Player Management:**
  - Add new players.
  - Retrieve player details (excluding sensitive fields).
  - Update player scores.
  - Delete player data.
- **Gameplay:**
  - Players guess a random secret number.
  - Feedback (clues) provided for each guess.
  - Players earn points for correct guesses and can retry after using up all attempts.
- **Data Storage:**
  - Player data stored using stable storage (BTreeMap).
  - Persistent data including player scores, secret numbers, and attempts.

## APIs

### Add Player
- **Endpoint:** `add_player`
- **Payload:**
  ```json
  {
    "name": "PlayerName"
  }
  ```
- **Response:**
  ```json
  {
    "id": 1,
    "name": "PlayerName",
    "score": 0,
    "attempts_left": 7,
    "clue": ""
  }
  ```

### Get Player
- **Endpoint:** `get_player`
- **Params:**
  - `id` (u64): Player ID
- **Response:**
  ```json
  {
    "id": 1,
    "name": "PlayerName",
    "score": 10,
    "attempts_left": 5,
    "clue": ""
  }
  ```

### Update Score
- **Endpoint:** `update_score`
- **Params:**
  - `id` (u64): Player ID
  - `score` (u64): New Score
- **Response:**
  ```json
  {
    "id": 1,
    "name": "PlayerName",
    "score": 20,
    "attempts_left": 5,
    "clue": ""
  }
  ```

### Delete Player
- **Endpoint:** `delete_player`
- **Params:**
  - `id` (u64): Player ID
- **Response:**
  ```json
  {
    "id": 1,
    "name": "PlayerName",
    "score": 20,
    "secret_number": 45,
    "attempts_left": 5,
    "created_at": 123456789,
    "updated_at": 123456790
  }
  ```

### Play Game
- **Endpoint:** `play_game`
- **Params:**
  - `id` (u64): Player ID
  - `guess` (u64): Player's guess
- **Response:**
  ```json
  {
    "id": 1,
    "name": "PlayerName",
    "score": 30,
    "attempts_left": 6,
    "clue": "The secret number is higher than your guess."
  }
  ```

## Implementation Details

### Modules
1. **Player and PublicPlayer Structs:**
   - `Player`: Stores complete player data, including sensitive fields like `secret_number`.
   - `PublicPlayer`: Excludes sensitive fields, used for API responses.

2. **Stable Storage:**
   - `StableBTreeMap` for managing player data persistently.
   - `IdCell` for generating unique player IDs.

3. **Error Handling:**
   - Custom `Error` enum for managing player not found and other scenarios.

### Gameplay Logic
- A random secret number is generated for each player when they first play.
- Players have up to 7 attempts to guess the number.
- Clues are provided to guide players.
- Score increases by 10 points for correct guesses.

### Libraries Used
- [serde](https://serde.rs): For serialization and deserialization.
- [candid](https://github.com/dfinity/candid): For managing IC-specific data types.
- [ic-stable-structures](https://github.com/dfinity/ic-stable-structures): For stable memory management.

## Setup

### Prerequisites
- Rust with `wasm32-unknown-unknown` target.
- Internet Computer SDK (dfx).

### Steps
1. Clone the repository:
   ```bash
   git clone <repository_url>
   cd <repository_name>
   ```
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Deploy to the IC:
   ```bash
   dfx deploy
   ```




