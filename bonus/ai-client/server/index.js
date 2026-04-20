const net = require('net');
const { v4: uuidv4 } = require('uuid');

const PORT = process.argv[2] ? parseInt(process.argv[2]) : 4242;

if (isNaN(PORT)) {
    console.error("Invalid port provided.");
    process.exit(1);
}

// Data Store
const state = {
    users: new Map(), // uuid -> { uuid, name, status }
    teams: new Map(), // uuid -> { uuid, name, description, channels: Map, subscribers: Set }
    privateMessages: [], // { senderUuid, receiverUuid, timestamp, body }
};

// Helper to parse commands with quoted arguments and escaped characters
function parseCommand(line) {
    const args = [];
    let currentArg = '';
    let inQuotes = false;
    let escaped = false;

    for (let i = 0; i < line.length; i++) {
        const char = line[i];

        if (escaped) {
            currentArg += char;
            escaped = false;
        } else if (char === '\\') {
            escaped = true;
        } else if (char === '"') {
            inQuotes = !inQuotes;
        } else if (char === ' ' && !inQuotes) {
            if (currentArg.length > 0) {
                args.push(currentArg);
                currentArg = '';
            }
        } else {
            currentArg += char;
        }
    }
    if (currentArg.length > 0) args.push(currentArg);
    return args;
}

const clients = new Set();

class ClientConnection {
    constructor(socket) {
        this.socket = socket;
        this.socket.setNoDelay(true);
        this.user = null;
        this.context = {
            team: null,
            channel: null,
            thread: null
        };
        this.buffer = '';

        socket.on('data', (data) => this.handleData(data));
        socket.on('end', () => this.handleDisconnect());
        socket.on('error', () => this.handleDisconnect());
    }

    send(code, ...args) {
        let message = `${code}`;
        if (args.length > 0) {
            const payload = args.map(arg => {
                const s = String(arg);
                // Escape backslashes and quotes as per RFC
                const escaped = s.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
                return `"${escaped}"`;
            }).join(' ');
            message += ` ${payload}`;
        }
        this.socket.write(`${message}\n`);
    }

    broadcast(code, ...args) {
        for (const client of clients) {
            if (client.user) {
                client.send(code, ...args);
            }
        }
    }

    handleData(data) {
        this.buffer += data.toString();
        let lines = this.buffer.split('\n');
        this.buffer = lines.pop();

        for (let line of lines) {
            line = line.trim();
            if (!line) continue;
            const args = parseCommand(line);
            if (args.length === 0) continue;
            const command = args.shift().toUpperCase();
            this.executeCommand(command, args);
        }
    }

    handleDisconnect() {
        if (this.user) {
            this.user.status = "0";
            this.broadcast(601, this.user.uuid, this.user.name);
        }
        clients.delete(this);
    }

    executeCommand(command, args) {
        if (command !== 'LOGIN' && !this.user) {
            return this.send(401, "Unauthorized");
        }

        switch (command) {
            case 'LOGIN': return this.login(args);
            case 'LOGOUT': return this.logout();
            case 'USERS': return this.listUsers();
            case 'USER': return this.getUser(args);
            case 'SEND': return this.sendMessage(args);
            case 'MESSAGES': return this.listMessages(args);
            case 'SUBSCRIBE': return this.subscribe(args);
            case 'SUBSCRIBED': return this.subscribed(args);
            case 'UNSUBSCRIBE': return this.unsubscribe(args);
            case 'USE': return this.use(args);
            case 'CREATE': return this.create(args);
            case 'LIST': return this.list();
            case 'INFO': return this.info();
            default: return this.send(400, "Bad Request");
        }
    }

    validate(str, max, field) {
        if (str.length > max) {
            this.send(400, `Field ${field} is too long (max ${max})`);
            return false;
        }
        return true;
    }

    login(args) {
        if (args.length !== 1) return this.send(400, "Invalid arguments");
        const name = args[0];
        if (!this.validate(name, 32, "name")) return;
        
        let user = Array.from(state.users.values()).find(u => u.name === name);
        if (!user) {
            user = { uuid: uuidv4(), name, status: "1" };
            state.users.set(user.uuid, user);
        } else {
            user.status = "1";
        }
        
        this.user = user;
        this.send(200, user.uuid, user.name);
        this.broadcast(600, user.uuid, user.name);
    }

    logout() {
        this.send(200, "Goodbye");
        this.socket.end();
    }

    listUsers() {
        const users = Array.from(state.users.values());
        const flatArgs = users.flatMap(u => [u.uuid, u.name, u.status]);
        this.send(200, ...flatArgs);
    }

    getUser(args) {
        if (args.length !== 1) return this.send(400, "Invalid arguments");
        const uuid = args[0];
        const user = state.users.get(uuid);
        if (!user) return this.send(404, "User not found");
        this.send(200, user.uuid, user.name, user.status);
    }

    sendMessage(args) {
        if (args.length !== 2) return this.send(400, "Invalid arguments");
        const [uuid, body] = args;
        if (!this.validate(body, 512, "body")) return;
        if (!state.users.has(uuid)) return this.send(404, "User not found");
        
        const timestamp = Math.floor(Date.now() / 1000);
        state.privateMessages.push({
            senderUuid: this.user.uuid,
            receiverUuid: uuid,
            timestamp,
            body
        });

        this.send(200);

        for (const client of clients) {
            if (client.user && client.user.uuid === uuid) {
                client.send(602, this.user.uuid, body);
            }
        }
    }

    listMessages(args) {
        if (args.length !== 1) return this.send(400, "Invalid arguments");
        const uuid = args[0];
        const msgs = state.privateMessages.filter(m => 
            (m.senderUuid === this.user.uuid && m.receiverUuid === uuid) ||
            (m.senderUuid === uuid && m.receiverUuid === this.user.uuid)
        );
        const flatArgs = msgs.flatMap(m => [m.senderUuid, String(m.timestamp), m.body]);
        this.send(200, ...flatArgs);
    }

    subscribe(args) {
        if (args.length !== 1) return this.send(400, "Invalid arguments");
        const teamUuid = args[0];
        const team = state.teams.get(teamUuid);
        if (!team) return this.send(404, "Team not found");
        team.subscribers.add(this.user.uuid);
        this.send(200, this.user.uuid, teamUuid);
    }

    subscribed(args) {
        if (args.length === 0) {
            const myTeams = Array.from(state.teams.values())
                .filter(t => t.subscribers.has(this.user.uuid));
            const flatArgs = myTeams.flatMap(t => [t.uuid, t.name, t.description]);
            this.send(200, ...flatArgs);
        } else {
            const teamUuid = args[0];
            const team = state.teams.get(teamUuid);
            if (!team) return this.send(404, "Team not found");
            const flatArgs = Array.from(team.subscribers).flatMap(uuid => {
                const u = state.users.get(uuid);
                return [u.uuid, u.name, u.status];
            });
            this.send(200, ...flatArgs);
        }
    }

    unsubscribe(args) {
        if (args.length !== 1) return this.send(400, "Invalid arguments");
        const teamUuid = args[0];
        const team = state.teams.get(teamUuid);
        if (!team) return this.send(404, "Team not found");
        team.subscribers.delete(this.user.uuid);
        this.send(200, this.user.uuid, teamUuid);
    }

    use(args) {
        this.context.team = args[0] || null;
        this.context.channel = args[1] || null;
        this.context.thread = args[2] || null;
        this.send(200);
    }

    create(args) {
        const { team, channel, thread } = this.context;
        if (!team) {
            if (args.length !== 2) return this.send(400, "Invalid arguments");
            const [name, desc] = args;
            if (!this.validate(name, 32, "name")) return;
            if (!this.validate(desc, 255, "description")) return;

            const newTeam = { uuid: uuidv4(), name, description: desc, channels: new Map(), subscribers: new Set() };
            state.teams.set(newTeam.uuid, newTeam);
            this.send(200, "TEAM", newTeam.uuid, newTeam.name, newTeam.description);
            this.broadcast(605, newTeam.uuid, newTeam.name, newTeam.description);
        } else if (!channel) {
            const t = state.teams.get(team);
            if (!t) return this.send(404, "Team not found");
            if (args.length !== 2) return this.send(400, "Invalid arguments");
            const [name, desc] = args;
            if (!this.validate(name, 32, "name")) return;
            if (!this.validate(desc, 255, "description")) return;

            const newChannel = { uuid: uuidv4(), name, description: desc, threads: new Map() };
            t.channels.set(newChannel.uuid, newChannel);
            this.send(200, "CHANNEL", newChannel.uuid, newChannel.name, newChannel.description);
            this.broadcast(606, newChannel.uuid, newChannel.name, newChannel.description);
        } else if (!thread) {
            const t = state.teams.get(team);
            const c = t?.channels.get(channel);
            if (!c) return this.send(404, "Channel not found");
            if (args.length !== 2) return this.send(400, "Invalid arguments");
            const [title, body] = args;
            if (!this.validate(title, 32, "title")) return;
            if (!this.validate(body, 512, "body")) return;

            const timestamp = Math.floor(Date.now() / 1000);
            const newThread = { uuid: uuidv4(), userUuid: this.user.uuid, timestamp, title, body, replies: [] };
            c.threads.set(newThread.uuid, newThread);
            this.send(200, "THREAD", newThread.uuid, newThread.userUuid, String(timestamp), newThread.title, newThread.body);
            this.broadcast(603, newThread.uuid, newThread.userUuid, String(timestamp), newThread.title, newThread.body);
        } else {
            const t = state.teams.get(team);
            const c = t?.channels.get(channel);
            const th = c?.threads.get(thread);
            if (!th) return this.send(404, "Thread not found");
            if (args.length !== 1) return this.send(400, "Invalid arguments");
            const body = args[0];
            if (!this.validate(body, 512, "body")) return;

            const timestamp = Math.floor(Date.now() / 1000);
            const reply = { threadUuid: th.uuid, userUuid: this.user.uuid, timestamp, body };
            th.replies.push(reply);
            this.send(200, "REPLY", th.uuid, this.user.uuid, String(timestamp), body);
            this.broadcast(604, team, th.uuid, this.user.uuid, body);
        }
    }

    list() {
        const { team, channel, thread } = this.context;
        if (!team) {
            const teams = Array.from(state.teams.values());
            const flatArgs = teams.flatMap(t => [t.uuid, t.name, t.description]);
            this.send(200, "TEAM", ...flatArgs);
        } else if (!channel) {
            const t = state.teams.get(team);
            if (!t) return this.send(404, "Team not found");
            const channels = Array.from(t.channels.values());
            const flatArgs = channels.flatMap(c => [c.uuid, c.name, c.description]);
            this.send(200, "CHANNEL", ...flatArgs);
        } else if (!thread) {
            const t = state.teams.get(team);
            const c = t?.channels.get(channel);
            if (!c) return this.send(404, "Channel not found");
            const threads = Array.from(c.threads.values());
            const flatArgs = threads.flatMap(th => [th.uuid, th.userUuid, String(th.timestamp), th.title, th.body]);
            this.send(200, "THREAD", ...flatArgs);
        } else {
            const t = state.teams.get(team);
            const c = t?.channels.get(channel);
            const th = c?.threads.get(thread);
            if (!th) return this.send(404, "Thread not found");
            const flatArgs = th.replies.flatMap(r => [th.uuid, r.userUuid, String(r.timestamp), r.body]);
            this.send(200, "REPLY", ...flatArgs);
        }
    }

    info() {
        const { team, channel, thread } = this.context;
        if (!team) {
            this.send(200, "USER", this.user.uuid, this.user.name, this.user.status);
        } else if (!channel) {
            const t = state.teams.get(team);
            if (!t) return this.send(404, "Team not found");
            this.send(200, "TEAM", t.uuid, t.name, t.description);
        } else if (!thread) {
            const t = state.teams.get(team);
            const c = t?.channels.get(channel);
            if (!c) return this.send(404, "Channel not found");
            this.send(200, "CHANNEL", c.uuid, c.name, c.description);
        } else {
            const t = state.teams.get(team);
            const c = t?.channels.get(channel);
            const th = c?.threads.get(thread);
            if (!th) return this.send(404, "Thread not found");
            this.send(200, "THREAD", th.uuid, th.userUuid, String(th.timestamp), th.title, th.body);
        }
    }
}

const server = net.createServer((socket) => {
    console.log('Client connected');
    const connection = new ClientConnection(socket);
    clients.add(connection);
});

server.listen(PORT, '0.0.0.0', () => {
    console.log(`Server listening on port ${PORT}`);
});
