// rulebound interactive demo
// ES6 module entry point

import init, { solve_demo, StepSolver } from '../pkg/rulebound_demo.js';

// --- State ---
let canvas, ctx;
let solver = null;
let autoplayInterval = null;
let grid = Array(81).fill(null); // 9x9 Sudoku grid, null = unsolved
let domains = Array(81).fill(null); // domain sets per cell

// --- Constants ---
const CELL_SIZE = 80;
const GRID_SIZE = 9;
const CANVAS_SIZE = GRID_SIZE * CELL_SIZE;
const COLORS = {
    bg: '#1a1a2e',
    gridLine: '#444466',
    thickLine: '#8888aa',
    cellBg: '#22223a',
    solvedText: '#e0e0ff',
    domainText: '#667788',
    highlight: '#4444aa',
    given: '#ffcc44',
};

// --- Canvas Drawing ---

function drawGrid() {
    if (!ctx) return;

    ctx.fillStyle = COLORS.bg;
    ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

    // Draw cells
    for (let row = 0; row < GRID_SIZE; row++) {
        for (let col = 0; col < GRID_SIZE; col++) {
            const idx = row * GRID_SIZE + col;
            const x = col * CELL_SIZE;
            const y = row * CELL_SIZE;

            // Cell background
            ctx.fillStyle = COLORS.cellBg;
            ctx.fillRect(x + 1, y + 1, CELL_SIZE - 2, CELL_SIZE - 2);

            const value = grid[idx];
            if (value !== null) {
                // Solved cell: draw large number
                ctx.fillStyle = COLORS.solvedText;
                ctx.font = 'bold 36px monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                ctx.fillText(String(value + 1), x + CELL_SIZE / 2, y + CELL_SIZE / 2);
            } else if (domains[idx]) {
                // Unsolved: draw domain candidates as small numbers
                ctx.fillStyle = COLORS.domainText;
                ctx.font = '12px monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                for (let v = 0; v < 9; v++) {
                    if (domains[idx].includes(v)) {
                        const dx = (v % 3) * (CELL_SIZE / 3) + CELL_SIZE / 6;
                        const dy = Math.floor(v / 3) * (CELL_SIZE / 3) + CELL_SIZE / 6;
                        ctx.fillText(String(v + 1), x + dx, y + dy);
                    }
                }
            }
        }
    }

    // Grid lines
    ctx.strokeStyle = COLORS.gridLine;
    ctx.lineWidth = 1;
    for (let i = 0; i <= GRID_SIZE; i++) {
        const pos = i * CELL_SIZE;
        ctx.beginPath();
        ctx.moveTo(pos, 0);
        ctx.lineTo(pos, CANVAS_SIZE);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(0, pos);
        ctx.lineTo(CANVAS_SIZE, pos);
        ctx.stroke();
    }

    // Thick lines for 3x3 boxes
    ctx.strokeStyle = COLORS.thickLine;
    ctx.lineWidth = 3;
    for (let i = 0; i <= 3; i++) {
        const pos = i * 3 * CELL_SIZE;
        ctx.beginPath();
        ctx.moveTo(pos, 0);
        ctx.lineTo(pos, CANVAS_SIZE);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(0, pos);
        ctx.lineTo(CANVAS_SIZE, pos);
        ctx.stroke();
    }
}

// --- Actions ---

function resetGrid() {
    grid = Array(81).fill(null);
    domains = Array(81).fill(null).map(() => [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    solver = null;
    stopAutoplay();
    updateStats({ variables: 81, constraints: '--', propagations: 0, backtracks: 0, time: '--', status: 'Ready' });
    drawGrid();
}

function generate() {
    const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
    const configJson = JSON.stringify({ mode: 'sudoku', seed });

    try {
        const result = solve_demo(configJson);
        const parsed = JSON.parse(result);
        setMessage(`Generated (seed=${seed}): ${parsed.status}`);
    } catch (e) {
        setMessage(`Error: ${e.message}`);
    }

    // For now, just reset with empty grid
    resetGrid();
}

function stepSolve() {
    if (!solver) {
        const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
        solver = new StepSolver(JSON.stringify({ mode: 'sudoku', seed }));
    }

    const eventJson = solver.step();
    const event = JSON.parse(eventJson);
    setMessage(`Step: ${event.type}`);
    drawGrid();
}

function toggleAutoplay() {
    if (autoplayInterval) {
        stopAutoplay();
    } else {
        autoplayInterval = setInterval(stepSolve, 200);
        document.getElementById('btn-autoplay').textContent = 'Stop';
    }
}

function stopAutoplay() {
    if (autoplayInterval) {
        clearInterval(autoplayInterval);
        autoplayInterval = null;
    }
    document.getElementById('btn-autoplay').textContent = 'Auto-play';
}

// --- UI Helpers ---

function updateStats(stats) {
    for (const [key, value] of Object.entries(stats)) {
        const el = document.getElementById(`stat-${key}`);
        if (el) el.textContent = String(value);
    }
}

function setMessage(text) {
    const el = document.getElementById('message');
    if (el) {
        el.textContent = text;
        el.style.display = text ? 'block' : 'none';
    }
}

// --- Init ---

async function main() {
    await init();

    canvas = document.getElementById('canvas');
    canvas.width = CANVAS_SIZE;
    canvas.height = CANVAS_SIZE;
    ctx = canvas.getContext('2d');

    // Wire up buttons
    document.getElementById('btn-generate').addEventListener('click', generate);
    document.getElementById('btn-step').addEventListener('click', stepSolve);
    document.getElementById('btn-autoplay').addEventListener('click', toggleAutoplay);
    document.getElementById('btn-reset').addEventListener('click', resetGrid);

    // Initial draw
    resetGrid();
    setMessage('WASM loaded. Click Generate to start.');
}

main().catch(err => {
    console.error('Failed to initialize rulebound demo:', err);
    const msg = document.getElementById('message');
    if (msg) {
        msg.textContent = `Init error: ${err.message}`;
        msg.style.display = 'block';
    }
});
