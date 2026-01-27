let gameState = null;
let selectedSquare = null;
let highlightedMoves = [];
let playerSide = null;

const API_BASE = '/api';

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    document.getElementById('newGameBtn').addEventListener('click', startNewGame);
    document.getElementById('backToSetup').addEventListener('click', showSetup);
});

function showSetup() {
    document.getElementById('setupPanel').style.display = 'block';
    document.getElementById('gameContainer').style.display = 'none';
    selectedSquare = null;
    highlightedMoves = [];
}

function showGame() {
    document.getElementById('setupPanel').style.display = 'none';
    document.getElementById('gameContainer').style.display = 'grid';
}

async function startNewGame() {
    const variant = document.getElementById('variant').value;
    const playerSideValue = document.getElementById('playerSide').value;
    const botType = document.getElementById('botType').value;

    playerSide = playerSideValue;

    try {
        const response = await fetch(`${API_BASE}/new-game`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                variant: variant,
                player_side: playerSideValue,
                bot_type: botType
            })
        });

        if (!response.ok) {
            throw new Error('Failed to start game');
        }

        gameState = await response.json();
        showGame();
        updateUI();

        if (gameState.message) {
            showMessage(gameState.message);
        }
    } catch (error) {
        alert('Error starting game: ' + error.message);
    }
}

async function makeMove(fromRow, fromCol, toRow, toCol) {
    try {
        const response = await fetch(`${API_BASE}/move`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                from_row: fromRow,
                from_col: fromCol,
                to_row: toRow,
                to_col: toCol
            })
        });

        const result = await response.json();

        if (!response.ok) {
            showMessage('Error: ' + (result.error || 'Invalid move'), 'error');
            return false;
        }

        // Update game state
        await refreshGameState();

        if (result.message) {
            showMessage(result.message);
        }

        return true;
    } catch (error) {
        showMessage('Error making move: ' + error.message, 'error');
        return false;
    }
}

async function refreshGameState() {
    try {
        const response = await fetch(`${API_BASE}/game-state`);
        gameState = await response.json();
        updateUI();
    } catch (error) {
        console.error('Error refreshing game state:', error);
    }
}

function updateUI() {
    if (!gameState) return;

    // Update info panel
    document.getElementById('currentVariant').textContent = gameState.variant;
    document.getElementById('playerRole').textContent = playerSide;
    document.getElementById('currentTurn').textContent = gameState.current_player;

    // Update selected piece display
    if (selectedSquare) {
        document.getElementById('selectedPiece').textContent =
            `(${selectedSquare.row}, ${selectedSquare.col})`;
    } else {
        document.getElementById('selectedPiece').textContent = 'None';
    }

    // Render board
    renderBoard();

    // Check for game over
    if (gameState.game_over) {
        const winner = gameState.winner || 'Draw';
        showMessage(`🎉 Game Over! Winner: ${winner}`, 'success');
    } else if (gameState.current_player === playerSide) {
        showMessage('Your turn! Select a piece to move.', 'info');
    } else {
        showMessage('Opponent is thinking...', 'info');
    }
}

function renderBoard() {
    const boardElement = document.getElementById('board');
    const size = gameState.board.length;

    boardElement.innerHTML = '';
    boardElement.style.gridTemplateColumns = `repeat(${size}, 50px)`;
    boardElement.style.gridTemplateRows = `repeat(${size}, 50px)`;

    for (let row = 0; row < size; row++) {
        for (let col = 0; col < size; col++) {
            const square = createSquare(row, col);
            boardElement.appendChild(square);
        }
    }
}

function createSquare(row, col) {
    const square = document.createElement('div');
    square.className = 'square';
    square.dataset.row = row;
    square.dataset.col = col;

    // Determine square type
    const size = gameState.board.length;
    const isCorner = (row === 0 && col === 0) ||
        (row === 0 && col === size - 1) ||
        (row === size - 1 && col === 0) ||
        (row === size - 1 && col === size - 1);
    const isThrone = row === Math.floor(size / 2) && col === Math.floor(size / 2);

    if (isCorner) {
        square.classList.add('corner');
    } else if (isThrone) {
        square.classList.add('throne');
    } else {
        square.classList.add('normal');
    }

    // Add piece
    const piece = gameState.board[row][col];
    if (piece !== '.') {
        square.textContent = piece;
        square.classList.add(`piece-${piece}`);
    }

    // Highlight selected square
    if (selectedSquare && selectedSquare.row === row && selectedSquare.col === col) {
        square.classList.add('selected');
    }

    // Highlight valid moves
    if (highlightedMoves.some(m => m.to_row === row && m.to_col === col)) {
        square.classList.add('highlighted');
    }

    // Add click handler
    square.addEventListener('click', () => handleSquareClick(row, col));

    return square;
}

function handleSquareClick(row, col) {
    if (gameState.game_over) {
        showMessage('Game is over. Start a new game!', 'info');
        return;
    }

    if (gameState.current_player !== playerSide) {
        showMessage('Wait for your turn!', 'warning');
        return;
    }

    const piece = gameState.board[row][col];

    // If clicking a highlighted move destination
    if (highlightedMoves.some(m => m.to_row === row && m.to_col === col)) {
        if (selectedSquare) {
            makeMove(selectedSquare.row, selectedSquare.col, row, col);
            clearSelection();
        }
        return;
    }

    // If clicking on own piece, select it
    if (piece !== '.' && isOwnPiece(piece)) {
        selectSquare(row, col);
    } else {
        clearSelection();
    }
}

function isOwnPiece(piece) {
    if (playerSide === 'Defenders') {
        return piece === 'D' || piece === 'K';
    } else {
        return piece === 'A';
    }
}

function selectSquare(row, col) {
    selectedSquare = { row, col };

    // Find valid moves for this piece
    highlightedMoves = gameState.legal_moves.filter(
        m => m.from_row === row && m.from_col === col
    );

    updateUI();
}

function clearSelection() {
    selectedSquare = null;
    highlightedMoves = [];
    updateUI();
}

function showMessage(message, type = 'info') {
    const messageElement = document.getElementById('gameMessage');
    messageElement.textContent = message;

    // Add styling based on type
    messageElement.style.borderLeftColor =
        type === 'error' ? '#dc3545' :
            type === 'success' ? '#28a745' :
                type === 'warning' ? '#ffc107' :
                    '#667eea';
}
