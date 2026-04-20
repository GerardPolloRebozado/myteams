# MyTeams Web Client & Server (RFC 4242)

This project is a full implementation of the **MyTeams** protocol as defined in RFC 4242. It includes a custom TCP server and a modern web-based GUI client.

## 🤖 AI Development Note
This project was developed entirely by an AI (Gemini).
**Context:** I was provided with only the `docs/main.typ` (the RFC) and tasked with building a fully compliant ecosystem.
**How I did it:** 
1. **RFC Analysis:** I parsed the Typst document to extract all commands, status codes, and asynchronous event requirements.
2. **Server Implementation:** I built a Node.js TCP server from scratch that implements the state management (users, teams, channels, threads) and context tracking required by the RFC.
3. **Client Architecture:** Since browsers cannot talk raw TCP, I implemented a **Bridge Pattern**. A Node.js middleware acts as a transparent proxy, forwarding TCP packets to the browser via WebSockets.
4. **GUI:** I developed a Vanilla JS interface that replicates a modern chat experience while remaining 100% compliant with the text-based protocol.

## 🏗️ Architecture

- **Server (`/server`):** A Node.js TCP server listening on port 4242. It handles multi-client connections, command parsing (including quoted arguments), and event broadcasting.
- **Client Bridge (`/client/bridge.js`):** An Express/WebSocket server that bridges the gap between the Browser and any RFC-compliant TCP server.
- **GUI (`/client/public`):** A Vanilla HTML/CSS/JS application that provides a visual interface for the protocol.

## 🚀 Getting Started

### 1. Start the Server
```bash
cd server
npm install
node index.js [port] # Default: 4242
```

### 2. Start the Client Bridge
```bash
cd client
npm install
node bridge.js
```

### 3. Access the GUI
Open your browser and go to: `http://localhost:8080`

## 🔌 Compatibility
This client is designed to work with **any** RFC 4242 compliant server, including the original Rust implementation. Simply enter the host and port of your Rust server in the login screen, and the bridge will handle the TCP communication for you.

## 📝 Features Implemented
- [x] Full User Authentication (`LOGIN`, `LOGOUT`)
- [x] Global User Management (`USERS`, `USER`)
- [x] Private Messaging (`SEND`, `MESSAGES`)
- [x] Team Subscription & Management (`SUBSCRIBE`, `UNSUBSCRIBE`, `SUBSCRIBED`)
- [x] Contextual Resource Creation (`CREATE`, `USE`, `LIST`, `INFO`)
- [x] Real-time Asynchronous Events (6xx codes)
- [x] Support for quoted arguments (e.g., `CREATE "Team Name" "Description"`)
