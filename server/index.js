'use strict';

var stdin = process.openStdin();

// const readline = require('readline');
// readline.emitKeypressEvents(process.stdin);
// process.stdin.setRawMode(true);

const WebSocket = require('ws');
 
const wss = new WebSocket.Server({ port: 8080 });
 
wss.on('connection', function connection(ws) {
    console.log("connected...");
  
    ws.on('message', function incoming(message) {
        console.log('received: %s', message);
    });


    var size = 4;
    function pressCircle(arg) {
        ws.send(JSON.stringify({ "type": "press", "circle": size, "ring": 80}) );
        size = size + 4;
        if (size == 80) {
            size = 8;
        }  
        setTimeout(pressCircle, 60, 'funky');
    }
        
    setTimeout(pressCircle, 1500, 'funky');

    // process.stdin.on('keypress', (str, key) => {
    // if (key.ctrl && key.name === 'c') {
    //     process.exit();
    // } else {
        
    //     ws.send(JSON.stringify({ "type": "press", "circle": size, "ring": 80}) );
    //     size = size + 1;

    //     // console.log(`You pressed the "${str}" key`);
    //     // console.log();
    //     // console.log(key);
    //     // console.log();
//     }
// });

    // setup lister for stdin
    stdin.addListener("data", function(d) {
        // note:  d is an object, and when converted to a string it will
        // end with a linefeed.  so we (rather crudely) account for that  
        // with toString() and then trim() 
        //console.log("you entered: [" + d.toString().trim() + "]");
        ws.send(d.toString().trim());
    });
    
    ws.on('close', function() {
        console.log('disconnected');
    });
});


