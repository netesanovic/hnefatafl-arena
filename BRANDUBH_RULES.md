# Brandubh Rules - Detailed

## Overview

This implementation of Brandubh follows the Copenhagen rules (https://aagenielsen.dk/copenhagen_rules.php) with specific modifications.

## Board Setup

7x7 board with:
- **8 Attackers** (black pieces) - positioned in a cross formation on the edges
- **4 Defenders** (white pieces) - surrounding the king
- **1 King** - in the center throne
- **4 Corners** - king's escape squares
- **1 Throne** - center square, special properties

```
    0  1  2  3  4  5  6 
 0  X  .  .  A  .  .  X 
 1  .  .  .  A  .  .  . 
 2  .  .  .  D  .  .  . 
 3  A  A  D  K  D  A  A 
 4  .  .  .  D  .  .  . 
 5  .  .  .  A  .  .  . 
 6  X  .  .  A  .  .  X 
```

## Movement Rules

1. All pieces move like rooks in chess (orthogonally, any distance)
2. Pieces cannot jump over other pieces
3. **Only the king** can occupy corners or return to the throne
4. Once the king leaves the throne, no other piece can occupy it

## Capture Rules

### Regular Pieces (Attackers and Defenders)

A piece is captured by being sandwiched between two enemy pieces on opposite sides.

**Hostile Squares:**
- **Corners**: Hostile to ALL pieces (including defenders and the king)
- **Throne**: 
  - Always hostile to attackers
  - Hostile to defenders only when empty
  - NOT hostile to the king

### King Capture (Position-Dependent Rules)

The king's capture requirements depend on its position:

#### 1. King at Normal Position (NOT at or next to throne)
- Captured like any regular piece
- Requires two attackers on **opposite sides** (horizontal OR vertical)
- Hostile squares (corners) can substitute for one attacker
- Example: King at (1,3) can be captured by attackers at (1,2) and (1,4)

#### 2. King ON the Throne (center position)
- Must be surrounded by attackers on **all 4 sides** (N, S, E, W)
- No hostile square substitution
- Example: King at (3,3) requires attackers at (2,3), (4,3), (3,2), and (3,4)

#### 3. King NEXT TO the Throne (orthogonally adjacent)
- Must be surrounded on the **3 remaining sides** by attackers
- The throne counts as occupied (blocked side)
- Example: King at (2,3) requires attackers at (2,2), (2,4), and (1,3)
  - The throne at (3,3) blocks the south side

## Win Conditions

### Defenders Win
- King reaches any of the 4 corners: (0,0), (0,6), (6,0), or (6,6)
- King must actually move onto the corner square

### Attackers Win
- King is captured according to the position-dependent rules above

### Draw
- 100 moves without a victory
- No legal moves available (rare)

## Excluded Copenhagen Rules

The following Copenhagen rules are **NOT** implemented:

1. **No Shieldwall Rule (4b)**: Rows of pieces are not captured together
2. **No Exit Forts (6b)**: No special edge positions for defender victory
3. **No Surrounding Rule (7b)**: Pieces cannot be captured by surrounding alone (only sandwiching)

## Strategy Implications

### Key Differences from Traditional Rules

1. **King is vulnerable early**: Away from throne, king can be captured with just 2 attackers
2. **Throne area is critical**: King must avoid leaving throne area too early
3. **Corner hostility matters**: Corners can be used to capture defenders and the king
4. **Throne hostility**: Empty throne helps attackers capture defenders

### For Defenders
- Keep king near throne longer for better protection
- Once away from throne, ensure king has escape route
- Use corners carefully (they're hostile to your pieces too)

### For Attackers
- Lure king away from throne for easier capture
- Use corners and empty throne as capture aids
- Focus on cutting off escape routes early

## Implementation Notes

The rules are implemented in `src/game.rs`:
- `is_king_captured()`: Main king capture logic with position checks
- `is_surrounded_on_all_sides()`: For king on throne
- `is_surrounded_next_to_throne()`: For king adjacent to throne
- `is_hostile_to_king()`: Determines hostile squares for king capture
- `can_capture()`: Regular piece captures with corner/throne hostility

## Testing King Captures

You can test these rules by creating specific board positions:

```rust
let mut state = GameState::new_brandubh();
// Create test positions to verify capture rules
// ... move pieces to test positions
```

## Historical Context

These rules represent a modified Copenhagen-style interpretation of Brandubh, designed for:
- Clear, unambiguous capture rules
- Position-based king strength (stronger near throne)
- Fast-paced tactical gameplay
- Balanced competitive play
