const net = require('net');
const http = require('http');
const WebSocket = require('ws');
const express = require('express');
const path = require('path');

const app = express();
const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

app.use(express.static(path.join(__dirname, 'public')));

wss.on('connection', (ws) => {
    let tcpClient = null;

    ws.on('message', (message) => {
        const data = JSON.parse(message);

        if (data.type === 'CONNECT') {
            const { host, port } = data.payload;
            
            if (tcpClient) tcpClient.destroy();
            
            tcpClient = new net.Socket();
            tcpClient.connect(port, host, () => {
                ws.send(JSON.stringify({ type: 'CONNECTED' }));
            });

            tcpClient.on('data', (tcpData) => {
                ws.send(JSON.stringify({ type: 'DATA', payload: tcpData.toString() }));
            });

            tcpClient.on('close', () => {
                ws.send(JSON.stringify({ type: 'DISCONNECTED' }));
            });

            tcpClient.on('error', (err) => {
                ws.send(JSON.stringify({ type: 'ERROR', payload: err.message }));
            });
        } else if (data.type === 'SEND') {
            if (tcpClient) {
                tcpClient.write(data.payload + '\n');
            }
        }
    });

    ws.on('close', () => {
        if (tcpClient) tcpClient.destroy();
    });
});

const PORT = 8080;
server.listen(PORT, () => {
    console.log(`Bridge/GUI server running at http://localhost:${PORT}`);
});
