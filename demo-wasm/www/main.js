// rulebound interactive Sudoku demo
// ES6 module entry point

import init, { solve_sudoku, solve_demo, StepSolver } from '../pkg/rulebound_demo.js';

// --- State ---
let canvas, ctx;
let stepSolver = null;
let autoplayInterval = null;
let grid = Array(81).fill(0); // 0 = empty, 1-9 = value
let domains = Array(81).fill(null).map(() => [1,2,3,4,5,6,7,8,9]);
let givens = Array(81).fill(false);
let selectedCell = -1;
let stats = { variables: 0, constraints: 27, propagations: 0, backtracks: 0, time: '--', status: 'Ready' };

// --- Constants ---
const CELL_SIZE = 80;
const GRID_SIZE = 9;
const CANVAS_SIZE = GRID_SIZE * CELL_SIZE;
const COLORS = {
    bg: '#1a1a2e',
    gridLine: '#444466',
    thickLine: '#8888aa',
    cellBg: '#22223a',
    cellSelected: '#333355',
    solvedText: '#00ddff',
    givenText: '#ffcc44',
    domainText: '#556677',
    highlight: '#4444aa',
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
            ctx.fillStyle = idx === selectedCell ? COLORS.cellSelected : COLORS.cellBg;
            ctx.fillRect(x + 1, y + 1, CELL_SIZE - 2, CELL_SIZE - 2);

            const value = grid[idx];
            if (value > 0) {
                // Solved/given cell: draw large number
                ctx.fillStyle = givens[idx] ? COLORS.givenText : COLORS.solvedText;
                ctx.font = 'bold 36px monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                ctx.fillText(String(value), x + CELL_SIZE / 2, y + CELL_SIZE / 2);
            } else if (domains[idx] && domains[idx].length > 0 && domains[idx].length < 9) {
                // Unsolved: draw domain candidates as small numbers
                ctx.fillStyle = COLORS.domainText;
                ctx.font = '12px monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                for (const v of domains[idx]) {
                    const dv = v - 1; // 0-indexed for positioning
                    const dx = (dv % 3) * (CELL_SIZE / 3) + CELL_SIZE / 6;
                    const dy = Math.floor(dv / 3) * (CELL_SIZE / 3) + CELL_SIZE / 6;
                    ctx.fillText(String(v), x + dx, y + dy);
                }
            }
        }
    }

    // Grid lines
    ctx.strokeStyle = COLORS.gridLine;
    ctx.lineWidth = 1;
    for (let i = 0; i <= GRID_SIZE; i++) {
        const pos = i * CELL_SIZE;
        ctx.beginPath(); ctx.moveTo(pos, 0); ctx.lineTo(pos, CANVAS_SIZE); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(0, pos); ctx.lineTo(CANVAS_SIZE, pos); ctx.stroke();
    }

    // Thick lines for 3x3 boxes
    ctx.strokeStyle = COLORS.thickLine;
    ctx.lineWidth = 3;
    for (let i = 0; i <= 3; i++) {
        const pos = i * 3 * CELL_SIZE;
        ctx.beginPath(); ctx.moveTo(pos, 0); ctx.lineTo(pos, CANVAS_SIZE); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(0, pos); ctx.lineTo(CANVAS_SIZE, pos); ctx.stroke();
    }
}

// --- Actions ---

function resetGrid() {
    grid = Array(81).fill(0);
    domains = Array(81).fill(null).map(() => [1,2,3,4,5,6,7,8,9]);
    givens = Array(81).fill(false);
    selectedCell = -1;
    stepSolver = null;
    stopAutoplay();
    stats = { variables: 81, constraints: 27, propagations: 0, backtracks: 0, time: '--', status: 'Ready' };
    updateStats(stats);
    drawGrid();
    setMessage('Click a cell to place a value (1-9), then Solve or Step.');
}

function generate() {
    const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
    const t0 = performance.now();

    // Solve an empty grid to get a full solution
    const emptyPuzzle = JSON.stringify({ puzzle: Array(81).fill(0).join(','), seed: seed });

    try {
        const resultJson = solve_sudoku('[' + Array(81).fill(0).join(',') + ']');
        const result = JSON.parse(resultJson);

        if (result.status === 'solved') {
            const solution = result.solution;

            // Start with full solution
            grid = [...solution];
            givens = Array(81).fill(true);

            // Remove cells to create puzzle (keep ~30 givens)
            let rng = seed;
            const indices = Array.from({length: 81}, (_, i) => i);
            // Simple shuffle
            for (let i = indices.length - 1; i > 0; i--) {
                rng = (rng * 1103515245 + 12345) & 0x7fffffff;
                const j = rng % (i + 1);
                [indices[i], indices[j]] = [indices[j], indices[i]];
            }

            // Remove ~51 cells (keep ~30)
            const removeCount = 51;
            for (let k = 0; k < removeCount && k < indices.length; k++) {
                const idx = indices[k];
                grid[idx] = 0;
                givens[idx] = false;
            }

            // Reset domains for empty cells
            domains = grid.map((v, i) => v > 0 ? [v] : [1,2,3,4,5,6,7,8,9]);

            const elapsed = (performance.now() - t0).toFixed(1);
            stats = { variables: 81, constraints: 27, propagations: result.propagations, backtracks: result.backtracks, time: elapsed + 'ms', status: 'Generated' };
            updateStats(stats);
            stepSolver = null;
            drawGrid();
            setMessage(`Puzzle generated (seed=${seed}, ${givens.filter(g => g).length} givens)`);
        } else {
            setMessage(`Generation failed: ${result.message}`);
        }
    } catch (e) {
        setMessage(`Error: ${e.message}`);
    }
}

function solvePuzzle() {
    const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
    const t0 = performance.now();

    const puzzleArray = '[' + grid.join(',') + ']';

    try {
        const resultJson = solve_sudoku(puzzleArray);
        const result = JSON.parse(resultJson);

        if (result.status === 'solved') {
            grid = result.solution;
            domains = grid.map(v => [v]);
            const elapsed = (performance.now() - t0).toFixed(1);
            stats.propagations = result.propagations;
            stats.backtracks = result.backtracks;
            stats.time = elapsed + 'ms';
            stats.status = 'Solved';
            stats.variables = 81;
            updateStats(stats);
            drawGrid();
            setMessage(`Solved! (${result.propagations} propagations, ${result.backtracks} backtracks, ${elapsed}ms)`);
        } else {
            setMessage(`No solution: ${result.message}`);
            stats.status = 'Failed';
            updateStats(stats);
        }
    } catch (e) {
        setMessage(`Error: ${e.message}`);
    }
}

function stepSolve() {
    if (!stepSolver) {
        const puzzleArray = '[' + grid.join(',') + ']';
        const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
        stepSolver = new StepSolver(JSON.stringify({ puzzle: puzzleArray, seed }));
        // Get initial state
        updateFromSolver();
    }

    const eventJson = stepSolver.step();
    const event = JSON.parse(eventJson);

    // Update grid from solver state
    updateFromSolver();

    // Update stats
    stats.status = event.type;
    if (event.variables_solved !== undefined) {
        stats.variables = event.variables_solved;
    }
    updateStats(stats);

    const messages = {
        'propagated': `Propagated: ${event.variables_solved || '?'} solved, ${event.remaining || '?'} remaining`,
        'select': `Selected cell R${event.row+1}C${event.col+1} (${event.domain_size} candidates)`,
        'assign': `Assigned R${event.row+1}C${event.col+1} = ${event.value}`,
        'solved': event.message || 'Solved!',
        'failed': event.message || 'No solution',
        'backtrack': event.message || 'Backtracking...',
    };
    setMessage(messages[event.type] || `Step: ${event.type}`);

    if (event.type === 'select') {
        selectedCell = event.variable;
    }

    if (event.type === 'solved' || event.type === 'failed') {
        stopAutoplay();
    }

    drawGrid();
}

function updateFromSolver() {
    if (!stepSolver) return;
    try {
        const stateJson = stepSolver.get_state();
        const state = JSON.parse(stateJson);
        grid = state.grid;
        domains = state.domains;
        givens = state.givens;
        stats.variables = state.variables_solved;
    } catch (e) {
        // Ignore parse errors
    }
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

// --- Cell Interaction ---

function handleCanvasClick(e) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const col = Math.floor(x / CELL_SIZE);
    const row = Math.floor(y / CELL_SIZE);

    if (row >= 0 && row < 9 && col >= 0 && col < 9) {
        const idx = row * 9 + col;
        if (!givens[idx] || grid[idx] === 0) {
            // Cycle value: 0 -> 1 -> 2 -> ... -> 9 -> 0
            grid[idx] = (grid[idx] % 9) + 1;
            if (grid[idx] === 10) grid[idx] = 0;

            if (grid[idx] > 0) {
                givens[idx] = true;
                domains[idx] = [grid[idx]];
            } else {
                givens[idx] = false;
                domains[idx] = [1,2,3,4,5,6,7,8,9];
            }

            selectedCell = idx;
            stepSolver = null; // Reset step solver when grid changes
            drawGrid();
        }
    }
}

// --- UI Helpers ---

function updateStats(s) {
    for (const [key, value] of Object.entries(s)) {
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
    document.getElementById('btn-solve').addEventListener('click', solvePuzzle);

    // Canvas click for cell editing
    canvas.addEventListener('click', handleCanvasClick);

    // Initial draw
    resetGrid();
    setMessage('WASM loaded. Click cells to place values, then Solve or Step.');
}

main().catch(err => {
    console.error('Failed to initialize rulebound demo:', err);
    const msg = document.getElementById('message');
    if (msg) {
        msg.textContent = `Init error: ${err.message}`;
        msg.style.display = 'block';
    }
});
