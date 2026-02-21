// rulebound interactive constraint solver demo
// ES6 module entry point — Sudoku + Map Coloring modes

import init, { solve_sudoku, solve_demo, StepSolver, solve_map_coloring, MapStepSolver } from '../pkg/rulebound_demo.js';

// --- State ---
let canvas, ctx;
let currentMode = 'sudoku';
let stepSolver = null;       // StepSolver or MapStepSolver
let autoplayInterval = null;
let autoplaySpeed = 200;

// Sudoku state
let grid = Array(81).fill(0);
let domains = Array(81).fill(null).map(() => [1,2,3,4,5,6,7,8,9]);
let givens = Array(81).fill(false);
let selectedCell = -1;

// Map coloring state
const AUSTRALIA = {
    nodes: ['WA','NT','SA','Q','NSW','V','T'],
    positions: [
        [120, 350], // WA
        [320, 150], // NT
        [320, 350], // SA
        [500, 150], // Q
        [500, 320], // NSW
        [440, 460], // V
        [500, 560], // T
    ],
    edges: [[0,1],[0,2],[1,2],[1,3],[2,3],[2,4],[2,5],[3,4],[4,5]],
    colors: 3,
};
let mapColors = Array(7).fill(-1); // -1 = uncolored
let mapDomains = Array(7).fill(null).map(() => [0,1,2]);

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
    givenText: '#ffffff',
    domainText: '#556677',
    highlight: '#4444aa',
};

// Palette for map coloring nodes
const MAP_PALETTE = ['#ff6655', '#55cc55', '#5588ff', '#ffcc44', '#cc55cc', '#55cccc'];

// ─── Sudoku Drawing ─────────────────────────────────────────────────────────

function drawSudokuGrid() {
    if (!ctx) return;

    ctx.fillStyle = COLORS.bg;
    ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

    for (let row = 0; row < GRID_SIZE; row++) {
        for (let col = 0; col < GRID_SIZE; col++) {
            const idx = row * GRID_SIZE + col;
            const x = col * CELL_SIZE;
            const y = row * CELL_SIZE;

            ctx.fillStyle = idx === selectedCell ? COLORS.cellSelected : COLORS.cellBg;
            ctx.fillRect(x + 1, y + 1, CELL_SIZE - 2, CELL_SIZE - 2);

            const value = grid[idx];
            if (value > 0) {
                ctx.fillStyle = givens[idx] ? COLORS.givenText : COLORS.solvedText;
                ctx.font = 'bold 36px monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                ctx.fillText(String(value), x + CELL_SIZE / 2, y + CELL_SIZE / 2);
            } else if (domains[idx] && domains[idx].length > 0 && domains[idx].length < 9) {
                ctx.fillStyle = COLORS.domainText;
                ctx.font = '12px monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                for (const v of domains[idx]) {
                    const dv = v - 1;
                    const dx = (dv % 3) * (CELL_SIZE / 3) + CELL_SIZE / 6;
                    const dy = Math.floor(dv / 3) * (CELL_SIZE / 3) + CELL_SIZE / 6;
                    ctx.fillText(String(v), x + dx, y + dy);
                }
            }
        }
    }

    ctx.strokeStyle = COLORS.gridLine;
    ctx.lineWidth = 1;
    for (let i = 0; i <= GRID_SIZE; i++) {
        const pos = i * CELL_SIZE;
        ctx.beginPath(); ctx.moveTo(pos, 0); ctx.lineTo(pos, CANVAS_SIZE); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(0, pos); ctx.lineTo(CANVAS_SIZE, pos); ctx.stroke();
    }

    ctx.strokeStyle = COLORS.thickLine;
    ctx.lineWidth = 3;
    for (let i = 0; i <= 3; i++) {
        const pos = i * 3 * CELL_SIZE;
        ctx.beginPath(); ctx.moveTo(pos, 0); ctx.lineTo(pos, CANVAS_SIZE); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(0, pos); ctx.lineTo(CANVAS_SIZE, pos); ctx.stroke();
    }
}

// ─── Map Coloring Drawing ───────────────────────────────────────────────────

function drawMapColoring() {
    if (!ctx) return;

    ctx.fillStyle = COLORS.bg;
    ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

    const g = AUSTRALIA;

    // Draw edges
    ctx.strokeStyle = COLORS.gridLine;
    ctx.lineWidth = 2;
    for (const [a, b] of g.edges) {
        const [ax, ay] = g.positions[a];
        const [bx, by] = g.positions[b];
        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(bx, by);
        ctx.stroke();
    }

    // Draw nodes
    const nodeRadius = 30;
    for (let i = 0; i < g.nodes.length; i++) {
        const [x, y] = g.positions[i];
        const color = mapColors[i];

        // Node circle
        ctx.beginPath();
        ctx.arc(x, y, nodeRadius, 0, Math.PI * 2);
        if (color >= 0 && color < MAP_PALETTE.length) {
            ctx.fillStyle = MAP_PALETTE[color];
        } else {
            ctx.fillStyle = COLORS.cellBg;
        }
        ctx.fill();
        ctx.strokeStyle = i === selectedCell ? '#ffffff' : COLORS.thickLine;
        ctx.lineWidth = i === selectedCell ? 3 : 2;
        ctx.stroke();

        // Node label
        ctx.fillStyle = '#ffffff';
        ctx.font = 'bold 14px monospace';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(g.nodes[i], x, y);

        // Domain hints below node
        if (color < 0 && mapDomains[i] && mapDomains[i].length < g.colors) {
            ctx.fillStyle = COLORS.domainText;
            ctx.font = '10px monospace';
            const domStr = mapDomains[i].map(d => d + 1).join(',');
            ctx.fillText(`{${domStr}}`, x, y + nodeRadius + 14);
        }
    }

    // Title
    ctx.fillStyle = COLORS.thickLine;
    ctx.font = '16px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    ctx.fillText('Australia Map Coloring', CANVAS_SIZE / 2, 20);
}

function drawGrid() {
    if (currentMode === 'sudoku') {
        drawSudokuGrid();
    } else {
        drawMapColoring();
    }
}

// ─── Sudoku Actions ─────────────────────────────────────────────────────────

function resetSudoku() {
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

function resetMapColoring() {
    mapColors = Array(AUSTRALIA.nodes.length).fill(-1);
    mapDomains = Array(AUSTRALIA.nodes.length).fill(null).map(() =>
        Array.from({length: AUSTRALIA.colors}, (_, i) => i)
    );
    selectedCell = -1;
    stepSolver = null;
    stopAutoplay();
    stats = { variables: AUSTRALIA.nodes.length, constraints: AUSTRALIA.edges.length, propagations: 0, backtracks: 0, time: '--', status: 'Ready' };
    updateStats(stats);
    drawGrid();
    setMessage('Click a node to cycle colors, then Solve or Step.');
}

function resetGrid() {
    if (currentMode === 'sudoku') {
        resetSudoku();
    } else {
        resetMapColoring();
    }
}

function generate() {
    if (currentMode !== 'sudoku') {
        resetMapColoring();
        setMessage('Map Coloring mode: click Solve or Step to color the graph.');
        return;
    }

    const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
    const t0 = performance.now();

    try {
        const resultJson = solve_sudoku('[' + Array(81).fill(0).join(',') + ']');
        const result = JSON.parse(resultJson);

        if (result.status === 'solved') {
            const solution = result.solution;
            grid = [...solution];
            givens = Array(81).fill(true);

            let rng = seed;
            const indices = Array.from({length: 81}, (_, i) => i);
            for (let i = indices.length - 1; i > 0; i--) {
                rng = (rng * 1103515245 + 12345) & 0x7fffffff;
                const j = rng % (i + 1);
                [indices[i], indices[j]] = [indices[j], indices[i]];
            }

            const removeCount = 51;
            for (let k = 0; k < removeCount && k < indices.length; k++) {
                const idx = indices[k];
                grid[idx] = 0;
                givens[idx] = false;
            }

            domains = grid.map(v => v > 0 ? [v] : [1,2,3,4,5,6,7,8,9]);

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

function solveSudoku() {
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

function solveMapColoringInstant() {
    const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
    const t0 = performance.now();

    const g = AUSTRALIA;
    const config = JSON.stringify({
        nodes: g.nodes.length,
        colors: g.colors,
        edges: g.edges,
        seed: seed,
    });

    try {
        const resultJson = solve_map_coloring(config);
        const result = JSON.parse(resultJson);

        if (result.status === 'solved') {
            mapColors = result.solution;
            mapDomains = mapColors.map(c => [c]);
            const elapsed = (performance.now() - t0).toFixed(1);
            stats.propagations = result.propagations;
            stats.backtracks = result.backtracks;
            stats.time = elapsed + 'ms';
            stats.status = 'Solved';
            stats.variables = g.nodes.length;
            stats.constraints = g.edges.length;
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

function solvePuzzle() {
    if (currentMode === 'sudoku') {
        solveSudoku();
    } else {
        solveMapColoringInstant();
    }
}

// ─── Step Solving ───────────────────────────────────────────────────────────

function stepSolve() {
    if (currentMode === 'sudoku') {
        stepSolveSudoku();
    } else {
        stepSolveMap();
    }
}

function stepSolveSudoku() {
    if (!stepSolver) {
        const puzzleArray = '[' + grid.join(',') + ']';
        const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
        stepSolver = new StepSolver(JSON.stringify({ puzzle: puzzleArray, seed }));
        updateFromSudokuSolver();
    }

    const eventJson = stepSolver.step();
    const event = JSON.parse(eventJson);
    updateFromSudokuSolver();

    stats.status = event.type;
    updateStats(stats);

    const messages = {
        'propagated': `Propagated: var ${event.variable}, domain ${JSON.stringify(event.domain)}`,
        'assigned': `Assigned var ${event.variable} = ${event.value}`,
        'solved': 'Solved!',
        'contradiction': 'Contradiction — no solution',
        'backtrack': `Backtracking (depth ${event.depth})`,
    };
    setMessage(messages[event.type] || `Step: ${event.type}`);

    if (event.type === 'assigned' && event.variable !== undefined) {
        selectedCell = event.variable;
    }

    if (event.type === 'solved' || event.type === 'contradiction') {
        stopAutoplay();
    }

    drawGrid();
}

function updateFromSudokuSolver() {
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

function stepSolveMap() {
    if (!stepSolver) {
        const seed = parseInt(document.getElementById('seed-input').value, 10) || 42;
        const g = AUSTRALIA;
        const config = JSON.stringify({
            nodes: g.nodes.length,
            colors: g.colors,
            edges: g.edges,
            seed: seed,
        });
        stepSolver = new MapStepSolver(config);
        updateFromMapSolver();
    }

    const eventJson = stepSolver.step();
    const event = JSON.parse(eventJson);
    updateFromMapSolver();

    stats.status = event.type;
    updateStats(stats);

    const messages = {
        'propagated': `Propagated: node ${event.variable}, domain ${JSON.stringify(event.domain)}`,
        'assigned': `Assigned node ${event.variable} = color ${event.value + 1}`,
        'solved': 'Solved!',
        'contradiction': 'Contradiction — no solution',
        'backtrack': `Backtracking (depth ${event.depth})`,
    };
    setMessage(messages[event.type] || `Step: ${event.type}`);

    if (event.type === 'assigned' && event.variable !== undefined) {
        selectedCell = event.variable;
    }

    if (event.type === 'solved' || event.type === 'contradiction') {
        stopAutoplay();
    }

    drawGrid();
}

function updateFromMapSolver() {
    if (!stepSolver) return;
    try {
        const stateJson = stepSolver.get_state();
        const state = JSON.parse(stateJson);
        mapColors = state.colors;
        mapDomains = state.domains;
        stats.variables = state.variables_solved;
    } catch (e) {
        // Ignore parse errors
    }
}

// ─── Autoplay ───────────────────────────────────────────────────────────────

function toggleAutoplay() {
    if (autoplayInterval) {
        stopAutoplay();
    } else {
        autoplayInterval = setInterval(stepSolve, autoplaySpeed);
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

// ─── Canvas Interaction ─────────────────────────────────────────────────────

function handleCanvasClick(e) {
    const rect = canvas.getBoundingClientRect();
    const scaleX = CANVAS_SIZE / rect.width;
    const scaleY = CANVAS_SIZE / rect.height;
    const x = (e.clientX - rect.left) * scaleX;
    const y = (e.clientY - rect.top) * scaleY;

    if (currentMode === 'sudoku') {
        handleSudokuClick(x, y);
    } else {
        handleMapClick(x, y);
    }
}

function handleSudokuClick(x, y) {
    const col = Math.floor(x / CELL_SIZE);
    const row = Math.floor(y / CELL_SIZE);

    if (row >= 0 && row < 9 && col >= 0 && col < 9) {
        const idx = row * 9 + col;
        if (!givens[idx] || grid[idx] === 0) {
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
            stepSolver = null;
            drawGrid();
        }
    }
}

function handleMapClick(x, y) {
    const g = AUSTRALIA;
    const nodeRadius = 30;

    for (let i = 0; i < g.nodes.length; i++) {
        const [nx, ny] = g.positions[i];
        const dist = Math.sqrt((x - nx) ** 2 + (y - ny) ** 2);
        if (dist <= nodeRadius) {
            // Cycle color: -1 -> 0 -> 1 -> 2 -> -1
            mapColors[i] = mapColors[i] >= g.colors - 1 ? -1 : mapColors[i] + 1;
            mapDomains[i] = mapColors[i] >= 0 ? [mapColors[i]] : Array.from({length: g.colors}, (_, j) => j);
            selectedCell = i;
            stepSolver = null;
            drawGrid();
            return;
        }
    }
}

// ─── UI Helpers ─────────────────────────────────────────────────────────────

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

function switchMode(mode) {
    currentMode = mode;
    stepSolver = null;
    stopAutoplay();
    selectedCell = -1;
    resetGrid();
}

// ─── Init ───────────────────────────────────────────────────────────────────

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

    // Mode selector
    document.getElementById('mode-select').addEventListener('change', (e) => {
        switchMode(e.target.value);
    });

    // Speed slider
    const speedSlider = document.getElementById('speed-slider');
    const speedLabel = document.getElementById('speed-label');
    speedSlider.addEventListener('input', () => {
        autoplaySpeed = parseInt(speedSlider.value, 10);
        speedLabel.textContent = autoplaySpeed + 'ms';
        // If autoplay is running, restart with new speed
        if (autoplayInterval) {
            clearInterval(autoplayInterval);
            autoplayInterval = setInterval(stepSolve, autoplaySpeed);
        }
    });

    // Canvas click
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
