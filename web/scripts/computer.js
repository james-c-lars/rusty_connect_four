import * as wasm_funcs from "../../rust-wasm/pkg/rusty_connect_four.js";

/*
 * Worker functionality
 */

function reset() {
    wasm_funcs.new_game();
}

onmessage = function (message) {
    let operation = message.data.type;

    if(operation === 'update') {
        let column = message.data.column;

        let [success, gameOver, winner] = wasm_funcs.make_move(column);

        if(success == 0) {
            console.error("Piece could not be placed in column: ",column);
            return;
        }

        let moveScores = JSON.parse(wasm_funcs.get_move_scores());

        let returnMessage = {
            gameOver: gameOver > 0,
            moveScores,
            move: column,
            depth: 0
        }

        // We want to only do this if the game isn't over
        // Otherwise we risk an error in getValidMoves about trying to access layer 1
        if(gameOver == 0) {
            let moves = Object.keys(move_scores);
            let validMoves = [];
            for(let move=0; move < 7; move++) {
                validMoves.push(moves.includes(move.toString()));
            }

            returnMessage['validMoves'] = validMoves;
        }

        postMessage(returnMessage);

    } else if(operation === 'reset') {
        reset();
    } else if(operation === 'dump') {
        postMessage({
            hello: "hi"
        })
    } else {
        console.error('computer got unknown operation in message:', message);
    }

    wasm_funcs.generate_x_states(100000);
}