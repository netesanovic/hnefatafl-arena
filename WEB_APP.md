# Web Application

Play Hnefatafl against AI bots in your browser!

## Running the Web App

```bash
cargo run --example web_app
```

Then open your browser to **http://127.0.0.1:3000**

## Features

- **Interactive Board**: Click and drag pieces to make your moves
- **Play as Either Side**: Choose to play as Attackers or Defenders
- **Multiple Variants**: Play Brandubh (7×7) or Copenhagen Hnefatafl (11×11)
- **AI Opponents**: Challenge the Greedy Bot or Random Bot
- **Real-time Feedback**: See legal moves highlighted when you select a piece
- **Instant Bot Response**: The AI makes its move immediately after yours

## Game Setup

1. Select your preferred variant (Brandubh is faster)
2. Choose which side you want to play
3. Select your opponent bot
4. Click "Start Game"

## How to Play

- Click on one of your pieces to see its legal moves (highlighted in green)
- Click on a highlighted square to move there
- The bot will respond automatically
- Game ends when:
  - Defenders win: King reaches any corner
  - Attackers win: King is captured

## Controls

- **New Game**: Start a fresh game with new settings
- The current turn is always displayed
- Selected piece position is shown in the info panel

Enjoy strategizing against the bots! 🛡️⚔️
