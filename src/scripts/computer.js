const { invoke } = window.__TAURI__.tauri;

/*
    Global Variables
*/

var haltComputation = true;

// Operation count will serve to take our long computations and break them into chunks of this many operations
// This will allow us to use javascript's event loop to manage incoming messages since long computations will be more broken up
const maxOperations = 10000;
let operationCount = maxOperations;
const operationDelay = 50;

// For managing the amount of memory used for typed arrays
let allocatedMemory = 0;
const maxMemory = 256 * 1024 * 1024 / 7;

function printBoard(boardArray, offset) {
    let printString = "";
    for(let j=0; j < 6; j++) {
        let rowString = "";
        for(let i=0; i < 7; i++) {
            if(boardArray[j][i] == 0) {
                rowString += '| ';
            } else if(boardArray[j][i] == 1) {
                rowString += '|0';
            } else if(boardArray[j][i] == 2) {
                rowString += '|1';
            }
        }
        printString = rowString + '|\n' + printString;
    }

    console.log(printString);
}

async function generateBoards() {
    if(haltComputation) {
        return;
    }

    await invoke("generate_x_states", { x: maxOperations });
    allocatedMemory += maxOperations * 64 * 7;
    haltComputation = allocatedMemory >= maxMemory;

    let moveScores = await evaluate();
    postMessage({ moveScores });

    if(haltComputation) {
        postMessage({ notComputing: true });
        return;
    }

    setTimeout(generateBoards, operationDelay);
}

/*
 * Worker functionality
 */

async function move(col) {
    let successfulMove, gameOver, whoWon = await invoke("make_move", { col });

    if(!successfulMove) {
        console.error("move() called on invalid column");
        return;
    }

    let gameStatus = { status: gameOver };
    if(gameOver) gameStatus.winner = whoWon - 1;

    if(gameStatus['status']) {
        if(gameStatus['winner'] >= 0) {
            console.log('Won game: ' + whoWon);
        } else {
            console.log('Draw')
        }
    } else {
        let boardArray = await invoke("get_position");
        printBoard(boardArray);
    }

    if(gameOver) {
        haltComputation = true;
        postMessage({complete: true, notComputing: true});
    } else {
        allocatedMemory /= 7;
        if(haltComputation) {
            haltComputation = false;
            setTimeout(generateBoards, operationDelay);
        }
        postMessage({notComputing: false});
    }

    return gameStatus;
}

async function reset() {
    await invoke("new_game");
    allocatedMemory = 0;
    haltComputation = false;
    postMessage({complete: false, notComputing: false});

    setTimeout(generateBoards, operationDelay);
}

async function evaluate() {
    let moveScoresArray = await invoke("get_move_scores");
    let moveScores = {};
    for(let element of moveScoresArray) {
        let move, score = element;
        moveScores[move] = score;
    }
    console.log("Move scores: " + moveScores);
    return moveScores
}

onmessage = async function (message) {
    let operation = message.data.type;

    if(operation === 'update') {
        let column = message.data.column;
        let gameOver = await move(column);
        let moveScores = await evaluate();

        let returnMessage = {
            gameOver: gameOver,
            moveScores: moveScores,
            move: column,
            depth: 0,
        }

        // We want to only do this if the game isn't over
        // Otherwise we risk an error in getValidMoves about trying to access layer 1
        if(!gameOver.status) {
            returnMessage['validMoves'] = Object.keys(moveScores);
        }

        postMessage(returnMessage);

    } else if(operation === 'reset') {
        reset();
    } else {
        console.error('computer got unknown operation in message:', message);
    }
}