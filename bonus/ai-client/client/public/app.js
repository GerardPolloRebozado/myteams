let ws = null;
let currentContext = { team: null, channel: null, thread: null };
let currentUser = { uuid: null, name: null };
let activeChat = { type: 'global', id: null, name: null }; 

const elements = {
    loginModal: document.getElementById('login-modal'),
    mainUi: document.getElementById('main-ui'),
    hostInput: document.getElementById('host'),
    portInput: document.getElementById('port'),
    usernameInput: document.getElementById('username'),
    loginBtn: document.getElementById('login-btn'),
    logoutBtn: document.getElementById('logout-btn'),
    currentUserName: document.getElementById('current-user-name'),
    teamList: document.getElementById('team-list'),
    userList: document.getElementById('user-list'),
    breadcrumb: document.getElementById('context-breadcrumb'),
    resourceInfo: document.getElementById('current-resource-info'),
    chatMessages: document.getElementById('chat-messages'),
    messageInput: document.getElementById('message-input'),
    sendBtn: document.getElementById('send-btn'),
    goGlobalBtn: document.getElementById('go-global-btn'),
    createTeamBtn: document.getElementById('create-team-btn')
};

function connect() {
    const host = elements.hostInput.value;
    const port = parseInt(elements.portInput.value);
    const username = elements.usernameInput.value;
    if (!username) return alert("Username required");
    ws = new WebSocket(`ws://${window.location.host}`);
    ws.onopen = () => ws.send(JSON.stringify({ type: 'CONNECT', payload: { host, port } }));
    ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        if (msg.type === 'CONNECTED') sendCommand(`LOGIN "${username}"`);
        else if (msg.type === 'DATA') handleServerData(msg.payload);
        else if (msg.type === 'ERROR') alert("Bridge Error: " + msg.payload);
    };
}

function sendCommand(cmd) {
    console.log("%c -> Sending:", "color: #0078d4; font-weight: bold", cmd);
    ws.send(JSON.stringify({ type: 'SEND', payload: cmd }));
}

function handleServerData(data) {
    const lines = data.split('\n');
    for (let line of lines) {
        if (!line.trim()) continue;
        console.log("%c <- Received:", "color: #28a745; font-weight: bold", line);
        const code = line.substring(0, 3);
        const payload = line.substring(4);
        if (code.startsWith('2')) handleSuccess(code, payload);
        else if (code.startsWith('6')) handleEvent(code, payload);
        else handleError(code, payload);
    }
}

function parseArgs(line) {
    const args = [];
    let currentArg = '';
    let inQuotes = false;
    let escaped = false;
    for (let i = 0; i < line.length; i++) {
        const char = line[i];
        if (escaped) { currentArg += char; escaped = false; }
        else if (char === '\\') { escaped = true; }
        else if (char === '"') { inQuotes = !inQuotes; }
        else if (char === ' ' && !inQuotes) {
            if (currentArg.length > 0) { args.push(currentArg); currentArg = ''; }
        } else { currentArg += char; }
    }
    if (currentArg.length > 0) args.push(currentArg);
    return args;
}

function handleSuccess(code, payload) {
    const args = parseArgs(payload);
    const tag = args[0];
    const data = args.slice(1);
    
    // 1. Login
    if (code === "200" && !currentUser.uuid && args.length === 2) {
        currentUser.uuid = args[0];
        currentUser.name = args[1];
        showMainUi();
        refreshAll();
        return;
    }

    // 2. Resource Management (LIST and INFO separation)
    const { team, channel, thread } = currentContext;

    if (tag === 'TEAM') {
        if (!team) renderTeamList(data); // LIST Global
        else elements.resourceInfo.innerText = `Team: ${data[1]}`; // INFO Team
    } else if (tag === 'CHANNEL') {
        if (team && !channel) renderChannels(data); // LIST Team
        else elements.resourceInfo.innerText = `Channel: ${data[1]}`; // INFO Channel
    } else if (tag === 'THREAD') {
        if (team && channel && !thread) renderThreads(data); // LIST Channel
        else elements.resourceInfo.innerText = `Thread: ${data[3]}`; // INFO Thread
    } else if (tag === 'REPLY') {
        renderReplies(data);
    } else if (tag === 'USER' && args.length > 1) {
        elements.resourceInfo.innerText = `User: ${data[1]} (${data[2] === '1' ? 'Online' : 'Offline'})`;
    } 
    // 3. Untagged Lists
    else if (code === "200") {
        if (args.length === 0) return;
        const isMessages = args.length >= 2 && args[1].length >= 10 && !isNaN(args[1]);
        if (isMessages) {
            renderMessages(args);
            return;
        }
        const isUserList = args.length >= 3 && args[0].length === 36 && (args[2] === "0" || args[2] === "1");
        if (isUserList) {
            renderUsers(args);
        } else if (args.length >= 3 && args[0].length === 36) {
            renderSidebarTeams(args);
        }
    }
}

function renderSidebarTeams(data) {
    elements.teamList.innerHTML = '';
    for (let i = 0; i < data.length; i += 3) {
        const li = document.createElement('li');
        li.innerText = `👥 ${data[i+1]}`;
        li.onclick = () => selectTeam(data[i], data[i+1]);
        elements.teamList.appendChild(li);
    }
}

function renderTeamList(data) {
    elements.chatMessages.innerHTML = '<h2>Available Teams</h2><div class="list-grid"></div>';
    const grid = elements.chatMessages.querySelector('.list-grid');
    if (data.length === 0) {
        elements.chatMessages.innerHTML += '<p class="empty-msg">No teams exist yet.</p>';
        return;
    }
    for (let i = 0; i < data.length; i += 3) {
        const div = document.createElement('div');
        div.className = 'card';
        div.innerHTML = `<h3>👥 ${data[i+1]}</h3><p>${data[i+2]}</p><button class="small-btn">View/Join</button>`;
        div.onclick = (e) => {
            e.stopPropagation();
            sendCommand(`SUBSCRIBE "${data[i]}"`);
            selectTeam(data[i], data[i+1]);
        };
        grid.appendChild(div);
    }
}

function renderChannels(data) {
    elements.chatMessages.innerHTML = '<h2>Channels</h2><div class="list-grid"></div>';
    const grid = elements.chatMessages.querySelector('.list-grid');
    if (data.length === 0) {
        elements.chatMessages.innerHTML += '<p class="empty-msg">No channels yet.</p>';
        return;
    }
    for (let i = 0; i < data.length; i += 3) {
        const div = document.createElement('div');
        div.className = 'card';
        div.innerHTML = `<h3># ${data[i+1]}</h3><p>${data[i+2]}</p>`;
        div.onclick = () => selectChannel(data[i], data[i+1]);
        grid.appendChild(div);
    }
}

function renderThreads(data) {
    elements.chatMessages.innerHTML = '<h2>Threads</h2><div class="list-grid"></div>';
    const grid = elements.chatMessages.querySelector('.list-grid');
    if (data.length === 0) {
        elements.chatMessages.innerHTML += '<p class="empty-msg">No threads yet.</p>';
        return;
    }
    for (let i = 0; i < data.length; i += 5) {
        const div = document.createElement('div');
        div.className = 'card';
        div.innerHTML = `<h3>🧵 ${data[i+3]}</h3><p>${data[i+4]}</p>`;
        div.onclick = () => selectThread(data[i], data[i+3]);
        grid.appendChild(div);
    }
}

function renderReplies(data) {
    elements.chatMessages.innerHTML = '<h2>Conversation</h2>';
    for (let i = 0; i < data.length; i += 4) {
        const div = document.createElement('div');
        div.className = 'message';
        div.innerHTML = `<div class="message-meta">User ${data[i+1]} - ${new Date(data[i+2]*1000).toLocaleString()}</div><div class="message-body">${data[i+3]}</div>`;
        elements.chatMessages.appendChild(div);
    }
}

function renderMessages(data) {
    elements.chatMessages.innerHTML = '<h2>Private Messages</h2>';
    for (let i = 0; i < data.length; i += 3) {
        const isMe = data[i] === currentUser.uuid;
        const div = document.createElement('div');
        div.className = 'message';
        div.style.alignSelf = isMe ? 'flex-end' : 'flex-start';
        div.innerHTML = `<div class="message-meta">${isMe ? 'You' : 'Them'}</div><div class="message-body" style="background:${isMe?'#dcf8c6':'#eee'}">${data[i+2]}</div>`;
        elements.chatMessages.appendChild(div);
    }
}

function renderUsers(data) {
    elements.userList.innerHTML = '';
    for (let i = 0; i < data.length; i += 3) {
        const li = document.createElement('li');
        const status = data[i+2] === '1' ? '🟢' : '⚪';
        li.innerText = `${status} ${data[i+1]}`;
        li.onclick = () => selectUser(data[i], data[i+1]);
        elements.userList.appendChild(li);
    }
}

function selectTeam(uuid, name) {
    currentContext = { team: uuid, channel: null, thread: null };
    activeChat = { type: 'team', id: uuid, name: name };
    elements.breadcrumb.innerText = `/ ${name}`;
    elements.createTeamBtn.innerText = "➕ Create Channel";
    sendCommand(`USE "${uuid}"`);
    refreshAll();
}

function selectChannel(uuid, name) {
    currentContext.channel = uuid;
    currentContext.thread = null;
    elements.breadcrumb.innerText = `/ ${activeChat.name} / # ${name}`;
    elements.createTeamBtn.innerText = "➕ New Thread";
    sendCommand(`USE "${currentContext.team}" "${uuid}"`);
    refreshAll();
}

function selectThread(uuid, title) {
    currentContext.thread = uuid;
    elements.breadcrumb.innerText = `/ ${activeChat.name} / ... / 🧵 ${title}`;
    elements.createTeamBtn.innerText = "➕ Post Reply";
    sendCommand(`USE "${currentContext.team}" "${currentContext.channel}" "${uuid}"`);
    refreshAll();
}

function selectUser(uuid, name) {
    activeChat = { type: 'user', id: uuid, name: name };
    elements.resourceInfo.innerText = `Private Chat: ${name}`;
    sendCommand(`MESSAGES "${uuid}"`);
}

function refreshAll() {
    sendCommand("USERS");
    sendCommand("SUBSCRIBED");
    if (currentContext.team) sendCommand(`SUBSCRIBED "${currentContext.team}"`);
    sendCommand("LIST");
    sendCommand("INFO");
}

function showMainUi() {
    elements.loginModal.classList.add('hidden');
    elements.mainUi.classList.remove('hidden');
    elements.currentUserName.innerText = currentUser.name;
}

function handleEvent(code, payload) {
    refreshAll();
}

function handleError(code, payload) { alert(`Error ${code}: ${payload}`); }

elements.loginBtn.onclick = connect;
elements.logoutBtn.onclick = () => { sendCommand("LOGOUT"); location.reload(); };
elements.goGlobalBtn.onclick = () => {
    currentContext = { team: null, channel: null, thread: null };
    activeChat = { type: 'global', id: null, name: null };
    elements.breadcrumb.innerText = "/ Global";
    elements.createTeamBtn.innerText = "➕ Create Team";
    sendCommand(`USE`);
    refreshAll();
};

elements.createTeamBtn.onclick = () => {
    const { team, channel, thread } = currentContext;
    if (!team) {
        const n = prompt("Team Name:"), d = prompt("Description:");
        if (n && d) sendCommand(`CREATE "${n}" "${d}"`);
    } else if (!channel) {
        const n = prompt("Channel Name:"), d = prompt("Description:");
        if (n && d) sendCommand(`CREATE "${n}" "${d}"`);
    } else if (!thread) {
        const t = prompt("Thread Title:"), m = prompt("Message:");
        if (t && m) sendCommand(`CREATE "${t}" "${m}"`);
    } else {
        const b = prompt("Reply Body:");
        if (b) sendCommand(`CREATE "${b}"`);
    }
};

elements.sendBtn.onclick = () => {
    const val = elements.messageInput.value;
    if (!val) return;
    if (activeChat.type === 'user') {
        sendCommand(`SEND "${activeChat.id}" "${val}"`);
    } else if (currentContext.thread) {
        sendCommand(`CREATE "${val}"`);
    } else {
        alert("Select a User or Thread to send a message.");
    }
    elements.messageInput.value = '';
};
