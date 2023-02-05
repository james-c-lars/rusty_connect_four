const initialColorOne = '#b30000';
const initialColorTwo = '#000099';
var computerDelay = true;

// In case reset game is pressed while the computer is thinking
var resetAbort = false;

function removeLosing(moveList) {
    let trimmedMoveList = [];
    for(let move of moveList) {
        if(move[1] !== Number.NEGATIVE_INFINITY) {
            trimmedMoveList.push([move[0], move[1]]);
        }
    }
    return trimmedMoveList;
}

function usePossibleSelectionsToPickMove(moveList, possibleSelections) {
    const trimmedMoveList = removeLosing(moveList);
    let selectionLength = trimmedMoveList.length;
    if(selectionLength === 0) {
        selectionLength = moveList.length;
    } else {
        selectionLength = possibleSelections[selectionLength]
    }

    const selection = Math.floor(Math.random() * selectionLength);
    return moveList[selection][0];
}

const difficultyFuncs = {
    'easy': moveList => {
        //const possibleSelections = {1:1, 2:1, 3:2, 4:3, 5:4, 6:4, 7:5};
        const possibleSelections = {1:1, 2:2, 3:3, 4:4, 5:5, 6:6, 7:7};
        return usePossibleSelectionsToPickMove(moveList, possibleSelections);
    },
    'medium': moveList => {
        const possibleSelections = {1:1, 2:1, 3:1, 4:2, 5:2, 6:3, 7:3};
        return usePossibleSelectionsToPickMove(moveList, possibleSelections);
    },
    'hard': moveList => moveList[0][0]
}

function getBoard(index) {
    return Board.boardList[index];
}

// function tempMakeMove() {
//     getBoard(0).computer.postMessage({type: "makeMove"});
// }



class Piece extends HTMLElement{
    constructor() {
        super();
    }

    connectedCallback() {
        if(!this.getAttribute('class')) {
            this.setAttribute('class', 'blank-piece');
        }
    }

    color() {
        return this.style.getPropertyValue('--piece-color');
    }

    filled() {
        return this.getAttribute('class').split(' ').includes('full-piece');
    }

    fill(colorNum) {
        this.style.setProperty('--piece-color', colorNum == 1? 'var(--piece-color-one)' : 'var(--piece-color-two)');
        this.setAttribute('class', 'full-piece');
    }
    empty() {
        this.setAttribute('class', 'blank-piece');
    }

    instantFill(colorNum) {
        this.style.setProperty('--piece-color', colorNum == 1? 'var(--piece-color-one)' : 'var(--piece-color-two)');
        this.setAttribute('class', 'full-piece no-transition');
    }

    instantEmpty() {
        this.setAttribute('class', 'blank-piece no-transition');
    }

    show() {
        this.style.setProperty('opacity', '1');
    }
    hide() {
        this.style.setProperty('opacity', '0');
    }
} customElements.define('game-piece', Piece);



class Board extends HTMLElement {
    static boardCount = 0;
    static boardList = [];

    constructor() {
        super();

        this.index = Board.boardCount++;
        Board.boardList.push(this);

        // Get relevant elements

        // Board parts
        let boardTemplate = document.getElementById('template-board');
        let boardContent = boardTemplate.content.cloneNode(true);
        this.boardParent = boardContent.querySelector('.board');
        let boardDrop = boardContent.querySelector('.indicator-container');
        let boardContainer = boardContent.querySelector('.piece-container');
        this.gameOverText = boardContent.querySelector('.game-over');

        // Settings popup
        this.settingsPopup = boardContent.querySelector('.settings-popup');
        this.playerOneColor = this.settingsPopup.querySelector('#color-one');
        this.playerTwoColor = this.settingsPopup.querySelector('#color-two');
        let difficultyRadio = [this.settingsPopup.querySelector('#difficulty-easy'),
                               this.settingsPopup.querySelector('#difficulty-medium'),
                               this.settingsPopup.querySelector('#difficulty-hard')];
        let evalCheckbox = this.settingsPopup.querySelector('#evaluation-display');
        let computerDelayCheckbox = this.settingsPopup.querySelector('#computer-delay');
        let closePopupButton = this.settingsPopup.querySelector('#close-popup');

        // Eval bar
        this.evalBar = boardContent.querySelector('.eval-bar');
        let evalColsDiv = this.evalBar.querySelector('.eval-cols');
        this.depthCount = this.evalBar.querySelector('#depth-count');
        this.loadingMessage = this.evalBar.querySelector('#loading-message');

        // Settings bar
        this.beginButton = boardContent.querySelector('#begin-button')
        this.playerOneSelect = boardContent.querySelector('#player-one');
        this.playerTwoSelect = boardContent.querySelector('#player-two');
        this.settingsIcon = boardContent.querySelector('.setting-icon');

        /*
         * Add the game pieces to the board
         */
        this.pieces = [];
        for(let i=0; i < 7; i++) {
            this.pieces.push([]);

            // Create column div element
            let column = document.createElement('div');
            column.setAttribute('class', 'column');
            column.onclick = () => this.humanMove(i);
            column.onmouseover = () => this.colOnMouseOver(i);

            for(let j=0; j < 6; j++) {
                let piece = new Piece();

                this.pieces[i].push(piece);
                column.appendChild(piece);
            }
            boardContainer.insertBefore(column, this.settingsPopup);
        }

        // Whose turn it is
        this.turn = 1;
        this.move = 0;

        /*
         * Add the indicator element at the top of the board
         */
        this.pieceIndicator = new Piece();
        this.pieceIndicator.instantFill(this.turn);
        this.pieceIndicator.hide();
        this.pieceIndicator.style.setProperty('transition', 'transform 300ms 0ms ease');
        this.pieceIndicator.style.setProperty('box-shadow', '0 0 0 0 black');
        this.pieceIndicator.style.setProperty('background-color', 'transparent');
        boardDrop.appendChild(this.pieceIndicator);

        /*
         * Adding functionality for the settings popup
         */
        this.settingsIcon.onclick = () => this.openPopup();
        this.playerOneColor.value = initialColorOne;
        this.playerTwoColor.value = initialColorTwo;
        this.playerOneColor.onchange = () => this.changeColor(1);
        this.playerTwoColor.onchange = () => this.changeColor(2);
        difficultyRadio[0].onchange = () => this.computerMoveSelect = difficultyFuncs['easy'];
        difficultyRadio[1].onchange = () => this.computerMoveSelect = difficultyFuncs['medium'];
        difficultyRadio[2].onchange = () => this.computerMoveSelect = difficultyFuncs['hard'];
        computerDelayCheckbox.onchange = () => {
            computerDelay = computerDelayCheckbox.checked;
        }
        closePopupButton.onclick = () => this.closePopup();

        /*
         * Eval bar functionality
         */
        this.evalCols = [];
        for(let i=0; i < 7; i++) {
            let evalCol = document.createElement('p');
            evalCol.innerText = '0';
            this.evalCols.push(evalCol);
            evalColsDiv.appendChild(evalCol);
        }
        evalCheckbox.onchange = () => {
            if(evalCheckbox.checked) {
                this.evalBar.style.setProperty('display', 'flex');
            } else {
                this.evalBar.style.setProperty('display', 'none');
            }
        }

        /*
         * Misc functionality
         */
        boardContainer.onmouseout = () => this.boardOnMouseOut();
        this.beginButton.onclick = () => this.beginGame();
        this.playerOneSelect.onchange = () => this.changePlayers(1);
        this.playerTwoSelect.onchange = () => this.changePlayers(2);

        // Whether the board is interactable
        // Will become interactable when the move controller says it's the human's turn
        this.enable = false;

        // Attach a DecisionTree to the board
        this.initComputer();

        // Used to determine which move the computer makes
        // Default is that it always makes the best move
        this.computerMoveSelect = difficultyFuncs['hard'];

        // Whether the AI or a human is controlling each color
        this.control = {1:this.playerOneSelect.value, 2:this.playerTwoSelect.value};

        // Shadow root
        const shadowRoot = this.attachShadow({mode: 'open'});
        shadowRoot.appendChild(boardContent);
    }

    initComputer() {
        this.computer = new Worker('./scripts/computer.js', {type:'module'});
        this.computerValues = {
            moveScores: {0:0, 1:0, 2:0, 3:0, 4:0, 5:0, 6:0},
            gameOver: {status: false},
            complete: false,
            notComputing: false,
            move: 0
        }
        this.computer.onmessage = (message) => {
            console.log('Computer message data:', message.data);

            if('notComputing' in message.data) {
                if(message.data.notComputing) {
                    this.dispatchEvent(new Event('finishedComputing'));
                    this.loadingMessage.innerText = 'Finished';
                } else {
                    this.loadingMessage.innerText = 'Loading...';
                }
            }
            if('move' in message.data && message.data.move === this.move) {
                this.dispatchEvent(new Event('finishedUpdatingMove'));
            }
            if('moveScores' in message.data) {
                for(let i=0; i<7; i++) {
                    if(i in message.data.moveScores) {
                        this.evalCols[i].innerText = message.data.moveScores[i];
                    } else {
                        this.evalCols[i].innerText = '';
                    }
                }
            }
            if('depth' in message.data) {
                this.depthCount.innerText = `Depth: ${message.data.depth}`;
            }

            for(let key in message.data) {
                this.computerValues[key] = message.data[key];
            }
        }
        this.computer.onerror = err => {console.error(err); throw err;};
        this.computer.onmessageerror = (msgErr) => {
            console.error(msgErr); throw msgErr;
        };
    }

    beginGame() {
        this.playerOneSelect.disabled = true;
        this.playerTwoSelect.disabled = true;

        this.beginButton.onclick = () => this.resetGame();
        this.beginButton.textContent = 'Reset Game';

        resetAbort = false;

        this.moveController();
    }

    resetGame() {
        this.enable = false;
        this.empty();
        this.turn = 1;
        this.move = 0;
        
        this.computer.terminate();
        this.initComputer();

        this.playerOneSelect.disabled = false;
        this.playerTwoSelect.disabled = false;

        this.beginButton.onclick = () => this.beginGame();
        this.beginButton.textContent = 'Begin Game';

        this.loadingMessage.innerText = 'Loading...';
        this.gameOverText.style.setProperty('display', 'none');

        resetAbort = true;
        this.dispatchEvent(new Event('resetAbort'));
    }

    async moveController() {
        if(this.move !== this.computerValues.move) {
            await this.eventToPromise('finishedUpdatingMove');
            if(resetAbort) return;
        }
        if(this.computerValues.gameOver.status) {
            if(this.computerValues.gameOver.winner === 0) {
                this.gameOverText.innerText = "GAME OVER: Player Two Wins"
            } else if(this.computerValues.gameOver.winner === 1) {
                this.gameOverText.innerText = "GAME OVER: Player One Wins"
            } else {
                this.gameOverText.innerText = "GAME OVER: Draw"
            }

            this.gameOverText.style.setProperty('display', 'block');
            console.log("===GAME OVER===");
            return;
        }

        this.move++;
        // Here we have the respective party take their turn
        if(this.control[this.turn] === 'human') {
            // For a human we just want to activate their ability to click on the board
            this.enable = true;
        } else {
            let endThinking = this.thinkingIndicator();
            // For a computer we can call the function after a moment
            if(this.computerValues.notComputing || !computerDelay) {
                await new Promise(resolve => setTimeout(resolve, 500));
                if(resetAbort) {endThinking(); return;}

                this.computerMove(endThinking);
            } else {
                await this.eventToPromise('finishedComputing');
                if(resetAbort) {endThinking(); return;}

                this.computerMove(endThinking);
            }
        }
    }

    async computerMove(endThinking) {
        // Sorting the moves by how good they are
        let moveList = [];
        for(let key in this.computerValues.moveScores) {
            moveList.push([key, this.computerValues.moveScores[key]]);
        }
        moveList.sort((a, b) => b[1] - a[1]);
        
        // Choosing the move to be made
        let selectedMove = this.computerMoveSelect(moveList);

        endThinking();

        // Making the move
        console.log("Sending message to drop in column", selectedMove);
        this.computer.postMessage({type:'update', column:selectedMove});
        await this.dropPiece(selectedMove);

        // Final transition
        this.turn = 3 - this.turn;
        this.pieceIndicator.instantFill(this.turn);

        this.moveController();
    }

    async humanMove(column) {
        // Making sure a move is able to be made now
        if(!this.enable || this.pieces[column][0].filled()) return;
        this.enable = false;
        this.pieceIndicator.hide();

        // Making the move
        this.computer.postMessage({type:'update', column:column});
        await this.dropPiece(column);

        // Final transition
        this.turn = 3 - this.turn;
        this.pieceIndicator.instantFill(this.turn);

        this.moveController();
    }

    /*
     * Handling the updating of the visual board state
     */
    async dropPiece(column) {
        this.pieceIndicator.style.setProperty('transform', `translateX(calc(${column-3} * (var(--piece-size) + 2*var(--piece-margin))))`);
        this.pieceIndicator.style.removeProperty('transition');
        this.pieceIndicator.instantFill(this.turn);
        this.pieceIndicator.show();

        const piecesInCol = this.pieces[column];
        const turn = this.turn;

        // If no spaces are full, the promise rejects
        if(piecesInCol[0].filled()) return Promise.reject(`Column ${column} full`);

        // Find the last open space in the column
        let row = 5;
        while(row > 0 && piecesInCol[row].filled()) {
            row -= 1;
        }

        // index is where the currently animated piece is
        let index = 0;
        // transitionDown is a function which passes the animation from one piece to another
        function transitionDown() {
            piecesInCol[index].removeEventListener('transitionend', transitionDown);

            if(index < row) {
                piecesInCol[index].empty();

                index++;
                piecesInCol[index].addEventListener('transitionend', transitionDown);
                piecesInCol[index].fill(turn);
            }
        }

        return new Promise((resolve, reject) => {
            // Prepare the first piece to pass the transition on
            piecesInCol[0].addEventListener('transitionend', transitionDown);

            // Prepare the final piece to resolve the promise
            const final = () => {
                piecesInCol[row].removeEventListener('transitionend', final);
                resolve();
                this.pieceIndicator.hide();
                this.pieceIndicator.style.setProperty('transition', 'transform 300ms 0ms ease');
            };
            piecesInCol[row].addEventListener('transitionend', final);

            // Transition the first piece
            piecesInCol[0].fill(turn);
        });
    }

    thinkingIndicator() {
        this.pieceIndicator.style.removeProperty('transition');
        this.pieceIndicator.style.setProperty('transform', `translateX(calc(-3 * (var(--piece-size) + 2*var(--piece-margin))))`);
        this.pieceIndicator.instantFill(this.turn);
        this.pieceIndicator.show();

        let i = 0;
        const thinking = () => {
            i++;
            i = i % 12;
            let column = i >= 7 ? 12 - i : i;
            this.pieceIndicator.style.setProperty('transform', `translateX(calc(${column-3} * (var(--piece-size) + 2*var(--piece-margin))))`);
            this.pieceIndicator.style.setProperty('transition', 'transform 150ms 0ms linear');
        };

        this.pieceIndicator.addEventListener('transitionend', thinking);
        thinking();
        return () => {
            this.pieceIndicator.removeEventListener('transitionend', thinking);
            this.pieceIndicator.hide();
            this.pieceIndicator.style.setProperty('transition', 'transform 300ms 0ms ease');
        };
    }

    empty() {
        for(let i=0; i < 7; i++) {
            for(let j=0; j < 6; j++) {
                this.pieces[i][j].instantEmpty();
            }
        }
    }

    colOnMouseOver(column) {
        if(!this.enable) return;

        this.pieceIndicator.instantFill(this.turn);
        this.pieceIndicator.show();
        this.pieceIndicator.style.setProperty('transform', `translateX(calc(${column-3} * (var(--piece-size) + 2*var(--piece-margin))))`);
    }

    boardOnMouseOut() {
        if(!this.enable) return;

        this.pieceIndicator.hide();
    }

    openPopup() {
        this.settingsIcon.onclick = () => this.closePopup();
        this.settingsPopup.setAttribute('class', 'settings-popup popup-open');
    }

    closePopup() {
        this.settingsIcon.onclick = () => this.openPopup();
        this.settingsPopup.setAttribute('class', 'settings-popup popup-close');
    }

    changeColor(playerNum) {
        if(playerNum === 1) {
            this.boardParent.style.setProperty('--piece-color-one', this.playerOneColor.value);
        } else {
            this.boardParent.style.setProperty('--piece-color-two', this.playerTwoColor.value);
        }
    }

    changePlayers(playerNum) {
        if(playerNum === 1) {
            this.control[1] = this.playerOneSelect.value;
        } else {
            this.control[2] = this.playerTwoSelect.value;
        }
    }

    eventToPromise(eventName) {
        return new Promise(resolve => {
            const onEvent = () => {
                this.removeEventListener(eventName, onEvent);
                this.removeEventListener('resetAbort', onEvent);
                resolve();
            };
            this.addEventListener(eventName, onEvent);
            this.addEventListener('resetAbort', onEvent);
        });
    }

} customElements.define('game-board', Board);