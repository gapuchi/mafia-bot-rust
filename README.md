# Rocket League Mafia Bot

A Discord bot for running Mafia games in Rocket League, inspired by
[SunlessKhan's Rocket League Mafia](https://www.youtube.com/watch?v=nZjNx7UlqWY&t=628s).

The bot splits everyone in the game master's voice channel into Blue and Orange
teams, secretly assigns Mafia and Villager roles, and coordinates the vote after
the match.

## How a game works

1. Join a Discord voice channel with everyone who is playing.
2. Start a Mafia game with `$mafia new` (or `/mafia new`).
3. Each player uses **Reveal My Role** to see their team and role privately.
4. After the Rocket League match, the game master reacts with 🔷 or 🔶 to record
   the winning team.
5. The bot opens a vote for the players on the losing team and reveals the Mafia
   after voting finishes.

By default, games with up to six players have one Mafia; larger games have two.
Pass a number to choose a different count:

```text
$mafia new 2
```

Mafia players are distributed between the teams as evenly as possible.

To split the voice channel into teams without assigning Mafia roles, use:

```text
$game new
```

Commands are case-insensitive and can use either the `$` prefix or Discord slash
commands. Available commands are:

- `$ping`
- `$game new`
- `$mafia new [mafia_count]`
- `$register` — register the slash commands with Discord

## Run the bot

### Discord setup

Create an application and bot in the
[Discord Developer Portal](https://discord.com/developers/applications), then:

1. Enable the **Message Content Intent** if you want to use `$` prefix commands.
2. Invite the bot with the `bot` and `applications.commands` scopes.
3. Give it permission to view channels, send messages, embed links, add
   reactions, read message history, and manage messages.

Copy the example environment file and add the bot token:

```sh
cp .env.example .env
```

```dotenv
DISCORD_TOKEN=your-token-here
```

The bot reads `DISCORD_TOKEN` from the environment and automatically loads a
local `.env` file when present.

### With Cargo

This project uses Rust 2024 edition. With a current Rust toolchain installed:

```sh
cargo run
```

### With Nix

The included flake provides the Rust development environment:

```sh
nix develop
cargo run
```

To build the release package with Nix:

```sh
nix build
./result/bin/mafia-bot
```

## Current limitations

- Game state is stored in memory and is lost when the bot restarts.
- A running bot process tracks one active game at a time.
- Voting currently reveals the Mafia but does not calculate scores or display
  vote totals.
