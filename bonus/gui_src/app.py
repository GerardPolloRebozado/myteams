import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext, simpledialog
import shlex
import time
from collections import deque
from .network import MyTeamsClient

class App(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("MyTeams GUI Client")
        self.geometry("1100x800")
        self.dark_mode = True
        self.palettes = {
            "dark": {
                "bg": "#1e1e1e",
                "sidebar": "#252526",
                "header": "#2d2d2d",
                "text": "#d4d4d4",
                "accent": "#007acc",
                "input_bg": "#3c3c3c",
                "selected": "#37373d",
                "me": "#4fc1ff",
                "them": "#4ec9b0",
                "time": "#808080",
                "system": "#ce9178",
                "border": "#3c3c3c"
            },
            "light": {
                "bg": "#ffffff",
                "sidebar": "#f1f3f4",
                "header": "#e8eaed",
                "text": "#202124",
                "accent": "#1a73e8",
                "input_bg": "#ffffff",
                "selected": "#e8f0fe",
                "me": "#1a73e8",
                "them": "#188038",
                "time": "#5f6368",
                "system": "#d93025",
                "border": "#dadce0"
            }
        }
        self.colors = self.palettes["dark"]
        self.client = None
        self.current_user_uuid = None
        self.current_user_name = None
        self.users = {} 
        self.direct_messages = {} 
        self.teams = {} 
        self.channels = {} 
        self.threads = {} 
        self.replies = {} 
        self.selected_dm_user_uuid = None
        self.active_team_uuid = None
        self.active_channel_uuid = None
        self.active_thread_uuid = None
        self.pending_commands = deque()
        self._setup_login_ui()

    def _apply_theme(self):
        self.colors = self.palettes["dark"] if self.dark_mode else self.palettes["light"]
        self.configure(bg=self.colors["bg"])
        style = ttk.Style()
        style.theme_use('default')
        style.configure("TFrame", background=self.colors["bg"])
        style.configure("TLabel", background=self.colors["bg"], foreground=self.colors["text"])
        style.configure("TPanedwindow", background=self.colors["bg"])
        style.configure("Sidebar.TFrame", background=self.colors["sidebar"])
        style.configure("UserLabel.TLabel", font=("Helvetica", 10, "bold"), background=self.colors["sidebar"], foreground=self.colors["accent"])
        style.configure("TNotebook", background=self.colors["sidebar"], borderwidth=0)
        style.configure("TNotebook.Tab", background=self.colors["header"], foreground=self.colors["text"], padding=[10, 2])
        style.map("TNotebook.Tab", background=[("selected", self.colors["bg"])])
        style.configure("Treeview", background=self.colors["sidebar"], foreground=self.colors["text"], fieldbackground=self.colors["sidebar"], borderwidth=0)
        style.map("Treeview", background=[('selected', self.colors["accent"])])
        style.configure("Header.TLabel", font=("Helvetica", 12, "bold"), background=self.colors["bg"], foreground=self.colors["text"])
        if hasattr(self, 'main_area'):
            self.main_area.configure(style="TFrame")
            self.sidebar_frame.configure(style="Sidebar.TFrame")
            self.user_info_frame.configure(style="Sidebar.TFrame")
            self.dm_listbox.configure(bg=self.colors["sidebar"], fg=self.colors["text"], selectbackground=self.colors["accent"])
            self.chat_display.configure(bg=self.colors["sidebar"], fg=self.colors["text"], insertbackground=self.colors["text"])
            self.msg_entry.configure(bg=self.colors["input_bg"], fg=self.colors["text"], insertbackground=self.colors["text"])
            self.chat_display.tag_configure("me", foreground=self.colors["me"])
            self.chat_display.tag_configure("them", foreground=self.colors["them"])
            self.chat_display.tag_configure("time", foreground=self.colors["time"])
            self.chat_display.tag_configure("system", foreground=self.colors["system"])
            self.chat_display.tag_configure("bold", foreground=self.colors["text"])
            self.user_info_label.configure(background=self.colors["sidebar"])
            self.user_name_label.configure(style="UserLabel.TLabel")

    def _toggle_theme(self):
        self.dark_mode = not self.dark_mode
        self._apply_theme()

    def _setup_login_ui(self):
        self._apply_theme()
        self.login_frame = ttk.Frame(self)
        self.login_frame.pack(expand=True)
        ttk.Label(self.login_frame, text="MyTeams Login", font=("Helvetica", 18, "bold")).grid(row=0, column=0, columnspan=2, pady=(0, 20))
        ttk.Label(self.login_frame, text="Host:").grid(row=1, column=0, padx=5, pady=5, sticky="e")
        self.host_entry = tk.Entry(self.login_frame, bg=self.colors["input_bg"], fg=self.colors["text"], insertbackground=self.colors["text"], borderwidth=1)
        self.host_entry.insert(0, "127.0.0.1")
        self.host_entry.grid(row=1, column=1, padx=5, pady=5)
        ttk.Label(self.login_frame, text="Port:").grid(row=2, column=0, padx=5, pady=5, sticky="e")
        self.port_entry = tk.Entry(self.login_frame, bg=self.colors["input_bg"], fg=self.colors["text"], insertbackground=self.colors["text"], borderwidth=1)
        self.port_entry.insert(0, "8080")
        self.port_entry.grid(row=2, column=1, padx=5, pady=5)
        ttk.Label(self.login_frame, text="Username:").grid(row=3, column=0, padx=5, pady=5, sticky="e")
        self.user_entry = tk.Entry(self.login_frame, bg=self.colors["input_bg"], fg=self.colors["text"], insertbackground=self.colors["text"], borderwidth=1)
        self.user_entry.grid(row=3, column=1, padx=5, pady=5)
        self.user_entry.bind('<Return>', lambda e: self._login())
        ttk.Button(self.login_frame, text="Login", command=self._login).grid(row=4, column=0, columnspan=2, pady=20)

    def _login(self):
        host = self.host_entry.get()
        try:
            port = int(self.port_entry.get())
        except ValueError:
            messagebox.showerror("Error", "Invalid port")
            return
        username = self.user_entry.get()
        if not username:
            messagebox.showerror("Error", "Username is required")
            return
        self.current_user_name = username
        self.client = MyTeamsClient(host, port, self._handle_protocol)
        if self.client.connect():
            self.pending_commands.append(("LOGIN", None, None))
            self.client.send(f'LOGIN "{self.client.escape(username)}"')

    def _setup_main_ui(self):
        self.login_frame.pack_forget()
        self._apply_theme()

        self.main_paned = ttk.PanedWindow(self, orient=tk.HORIZONTAL)
        self.main_paned.pack(fill=tk.BOTH, expand=True)
        self.sidebar_frame = ttk.Frame(self.main_paned, style="Sidebar.TFrame")
        self.main_paned.add(self.sidebar_frame, weight=1)
        self.user_info_frame = ttk.Frame(self.sidebar_frame, style="Sidebar.TFrame", padding=10)
        self.user_info_frame.pack(fill=tk.X)
        self.user_info_label = ttk.Label(self.user_info_frame, text="Logged in as:", background=self.colors["sidebar"])
        self.user_info_label.pack(anchor="w")
        self.user_name_label = ttk.Label(self.user_info_frame, text=self.current_user_name, style="UserLabel.TLabel")
        self.user_name_label.pack(anchor="w")
        self.sidebar_tabs = ttk.Notebook(self.sidebar_frame)
        self.sidebar_tabs.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)
        self.dm_tab = ttk.Frame(self.sidebar_tabs, style="Sidebar.TFrame")
        self.sidebar_tabs.add(self.dm_tab, text="DMs")
        self.dm_listbox = tk.Listbox(self.dm_tab, font=("Helvetica", 11), borderwidth=0, highlightthickness=0, 
                                     bg=self.colors["sidebar"], fg=self.colors["text"], selectbackground=self.colors["accent"])
        self.dm_listbox.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)
        self.dm_listbox.bind('<<ListboxSelect>>', self._on_dm_user_select)
        self.teams_tab = ttk.Frame(self.sidebar_tabs, style="Sidebar.TFrame")
        self.sidebar_tabs.add(self.teams_tab, text="Teams")
        self.teams_tree = ttk.Treeview(self.teams_tab, show="tree", selectmode="browse")
        self.teams_tree.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)
        self.teams_tree.bind('<<TreeviewSelect>>', self._on_team_select)
        self.btn_frame = ttk.Frame(self.sidebar_frame, style="Sidebar.TFrame", padding=5)
        self.btn_frame.pack(fill=tk.X)
        ttk.Button(self.btn_frame, text="Refresh All", command=self._refresh_all).pack(fill=tk.X, pady=2)
        ttk.Button(self.btn_frame, text="Toggle Theme", command=self._toggle_theme).pack(fill=tk.X, pady=2)
        self.main_area = ttk.Frame(self.main_paned, padding=10, style="TFrame")
        self.main_paned.add(self.main_area, weight=3)
        self.nav_frame = ttk.Frame(self.main_area, style="TFrame")
        self.nav_frame.pack(fill=tk.X, pady=(0, 10))
        self.header_label = ttk.Label(self.nav_frame, text="Select a conversation", style="Header.TLabel")
        self.header_label.pack(side=tk.LEFT)
        self.action_btn = ttk.Button(self.nav_frame, text="Action", command=self._on_action_click)
        self.action_btn.pack(side=tk.RIGHT)
        self.action_btn.pack_forget()
        self.subscribe_btn = ttk.Button(self.nav_frame, text="Subscribe", command=self._on_subscribe_click)
        self.subscribe_btn.pack(side=tk.RIGHT, padx=5)
        self.subscribe_btn.pack_forget()
        self.chat_display = scrolledtext.ScrolledText(
            self.main_area, 
            state='disabled', 
            font=("Helvetica", 11),
            wrap=tk.WORD,
            padx=15,
            pady=15,
            bg=self.colors["sidebar"],
            fg=self.colors["text"],
            insertbackground=self.colors["text"],
            borderwidth=1,
            relief="flat"
        )
        self.chat_display.pack(fill=tk.BOTH, expand=True)
        self.chat_display.tag_configure("me", foreground=self.colors["me"], font=("Helvetica", 11, "bold"))
        self.chat_display.tag_configure("them", foreground=self.colors["them"], font=("Helvetica", 11, "bold"))
        self.chat_display.tag_configure("time", foreground=self.colors["time"], font=("Helvetica", 10))
        self.chat_display.tag_configure("system", foreground=self.colors["system"], font=("Helvetica", 11, "italic"))
        self.chat_display.tag_configure("bold", font=("Helvetica", 11, "bold"), foreground=self.colors["text"])
        self.input_frame = ttk.Frame(self.main_area, padding=(0, 10, 0, 0), style="TFrame")
        self.input_frame.pack(fill=tk.X)
        self.msg_entry = tk.Entry(self.input_frame, font=("Helvetica", 11), bg=self.colors["input_bg"], fg=self.colors["text"], 
                                  insertbackground=self.colors["text"], borderwidth=0)
        self.msg_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, ipady=8)
        self.msg_entry.bind('<Return>', lambda e: self._send_msg())
        self.send_btn = ttk.Button(self.input_frame, text="Send", command=self._send_msg)
        self.send_btn.pack(side=tk.RIGHT, padx=(10, 0))
        self._refresh_all()

    def _send_use(self, team=None, channel=None, thread=None):
        cmd = "USE"
        if team: cmd += f' "{team}"'
        if channel: cmd += f' "{channel}"'
        if thread: cmd += f' "{thread}"'
        self.pending_commands.append(("USE", None, None))
        self.client.send(cmd)

    def _refresh_all(self):
        self._refresh_users()
        self._refresh_teams()

    def _refresh_users(self):
        self.pending_commands.append(("USERS", None, None))
        self.client.send("USERS")

    def _refresh_teams(self):
        self.pending_commands.append(("LIST_TEAMS", None, None))
        self.client.send("LIST")

    def _on_dm_user_select(self, event):
        selection = self.dm_listbox.curselection()
        if selection:
            index = selection[0]
            user_str = self.dm_listbox.get(index)
            for uuid, name in self.users.items():
                if name in user_str:
                    self.selected_dm_user_uuid = uuid
                    self.active_team_uuid = None
                    self.active_channel_uuid = None
                    self.active_thread_uuid = None
                    self.header_label.config(text=f"Direct Message with {name}")
                    self.action_btn.pack_forget()
                    self.subscribe_btn.pack_forget()
                    self._load_dm_messages()
                    break

    def _on_team_select(self, event):
        selection = self.teams_tree.selection()
        if not selection: return
        item = selection[0]
        tags = self.teams_tree.item(item, "tags")
        uuid = item
        self.selected_dm_user_uuid = None
        if "team" in tags:
            self.active_team_uuid = uuid
            self.active_channel_uuid = None
            self.active_thread_uuid = None
            team_name = self.teams[uuid]["name"]
            self.header_label.config(text=f"Team: {team_name}")
            self.action_btn.config(text="Create Channel")
            self.action_btn.pack(side=tk.RIGHT)
            sub_text = "Unsubscribe" if self.teams[uuid]["subscribed"] else "Subscribe"
            self.subscribe_btn.config(text=sub_text)
            self.subscribe_btn.pack(side=tk.RIGHT, padx=5)
            self._send_use(uuid)
            self.pending_commands.append(("LIST_CHANNELS", uuid, None))
            self.client.send("LIST")
            self._display_info("team", uuid)
        elif "channel" in tags:
            parent_team = self.teams_tree.parent(item)
            self.active_team_uuid = parent_team
            self.active_channel_uuid = uuid
            self.active_thread_uuid = None
            chan_name = self.channels[parent_team][uuid]["name"]
            self.header_label.config(text=f"Channel: {chan_name}")
            self.action_btn.config(text="Create Thread")
            self.action_btn.pack(side=tk.RIGHT)
            self.subscribe_btn.pack_forget()
            self._send_use(parent_team, uuid)
            self.pending_commands.append(("LIST_THREADS", uuid, None))
            self.client.send("LIST")
            self._display_info("channel", uuid)
        elif "thread" in tags:
            chan_item = self.teams_tree.parent(item)
            team_item = self.teams_tree.parent(chan_item)
            self.active_team_uuid = team_item
            self.active_channel_uuid = chan_item
            self.active_thread_uuid = uuid
            thread_title = self.threads[chan_item][uuid]["title"]
            self.header_label.config(text=f"Thread: {thread_title}")
            self.action_btn.config(text="Reply")
            self.action_btn.pack(side=tk.RIGHT)
            self.subscribe_btn.pack_forget()
            self._send_use(team_item, chan_item, uuid)
            self.pending_commands.append(("LIST_REPLIES", uuid, None))
            self.client.send("LIST")

    def _display_info(self, type, uuid):
        self.chat_display.config(state='normal')
        self.chat_display.delete('1.0', tk.END)
        if type == "team":
            t = self.teams[uuid]
            self.chat_display.insert(tk.END, f"Team: {t['name']}\n", "bold")
            self.chat_display.insert(tk.END, f"Description: {t['desc']}\n\n")
            self.chat_display.insert(tk.END, "Select a channel in the sidebar to view messages.\n", "system")
        elif type == "channel":
            c = self.channels[self.active_team_uuid][uuid]
            self.chat_display.insert(tk.END, f"Channel: {c['name']}\n", "bold")
            self.chat_display.insert(tk.END, f"Description: {c['desc']}\n\n")
            self.chat_display.insert(tk.END, "Select a thread in the sidebar to view messages.\n", "system")
        self.chat_display.config(state='disabled')

    def _on_action_click(self):
        if self.active_thread_uuid:
            body = simpledialog.askstring("Reply", "Enter your message:")
            if body:
                self._send_use(self.active_team_uuid, self.active_channel_uuid, self.active_thread_uuid)
                self.pending_commands.append(("CREATE_REPLY", self.active_thread_uuid, None))
                self.client.send(f'CREATE "{self.client.escape(body)}"')
        elif self.active_channel_uuid:
            title = simpledialog.askstring("New Thread", "Enter thread title:")
            if title:
                msg = simpledialog.askstring("New Thread", "Enter initial message:")
                if msg:
                    self._send_use(self.active_team_uuid, self.active_channel_uuid)
                    self.pending_commands.append(("CREATE_THREAD", self.active_channel_uuid, None))
                    self.client.send(f'CREATE "{self.client.escape(title)}" "{self.client.escape(msg)}"')
        elif self.active_team_uuid:
            name = simpledialog.askstring("New Channel", "Enter channel name:")
            if name:
                desc = simpledialog.askstring("New Channel", "Enter description:")
                if desc:
                    self._send_use(self.active_team_uuid)
                    self.pending_commands.append(("CREATE_CHANNEL", self.active_team_uuid, None))
                    self.client.send(f'CREATE "{self.client.escape(name)}" "{self.client.escape(desc)}"')
        else:
            name = simpledialog.askstring("New Team", "Enter team name:")
            if name:
                desc = simpledialog.askstring("New Team", "Enter description:")
                if desc:
                    self._send_use()
                    self.pending_commands.append(("CREATE_TEAM", None, None))
                    self.client.send(f'CREATE "{self.client.escape(name)}" "{self.client.escape(desc)}"')

    def _on_subscribe_click(self):
        if self.active_team_uuid:
            t = self.teams[self.active_team_uuid]
            if t["subscribed"]:
                self.pending_commands.append(("UNSUBSCRIBE", self.active_team_uuid, None))
                self.client.send(f'UNSUBSCRIBE "{self.active_team_uuid}"')
            else:
                self.pending_commands.append(("SUBSCRIBE", self.active_team_uuid, None))
                self.client.send(f'SUBSCRIBE "{self.active_team_uuid}"')

    def _send_msg(self):
        msg = self.msg_entry.get()
        if not msg: return
        if self.selected_dm_user_uuid:
            self.pending_commands.append(("SEND", self.selected_dm_user_uuid, None))
            self.client.send(f'SEND "{self.selected_dm_user_uuid}" "{self.client.escape(msg)}"')
            timestamp = int(time.time())
            if self.selected_dm_user_uuid not in self.direct_messages:
                self.direct_messages[self.selected_dm_user_uuid] = []
            self.direct_messages[self.selected_dm_user_uuid].append((self.current_user_uuid, timestamp, msg))
            self._display_chat_history()
            self.msg_entry.delete(0, tk.END)
        elif self.active_thread_uuid:
            self._send_use(self.active_team_uuid, self.active_channel_uuid, self.active_thread_uuid)
            self.pending_commands.append(("CREATE_REPLY", self.active_thread_uuid, None))
            self.client.send(f'CREATE "{self.client.escape(msg)}"')
            self.msg_entry.delete(0, tk.END)
        else:
            messagebox.showinfo("Info", "Select a user or a thread to send messages")

    def _load_dm_messages(self, uuid=None):
        target = uuid or self.selected_dm_user_uuid
        if target:
            self.pending_commands.append(("DM_MESSAGES", target, None))
            self.client.send(f'MESSAGES "{target}"')

    def _display_chat_history(self):
        self.chat_display.config(state='normal')
        self.chat_display.delete('1.0', tk.END)
        history = []
        if self.selected_dm_user_uuid:
            history = self.direct_messages.get(self.selected_dm_user_uuid, [])
        elif self.active_thread_uuid:
            history = self.replies.get(self.active_thread_uuid, [])
        history.sort(key=lambda x: int(x[1]))
        if not history:
            self.chat_display.insert(tk.END, "No messages yet.\n", "system")
        else:
            for sender_uuid, timestamp, msg in history:
                is_me = (sender_uuid == self.current_user_uuid)
                sender_name = "You" if is_me else self.users.get(sender_uuid, "Unknown")
                tag = "me" if is_me else "them"
                time_str = time.strftime('%H:%M', time.localtime(int(timestamp)))

                self.chat_display.insert(tk.END, f"[{time_str}] ", "time")
                self.chat_display.insert(tk.END, f"{sender_name}: ", tag)
                self.chat_display.insert(tk.END, f"{msg}\n")
        self.chat_display.see(tk.END)
        self.chat_display.config(state='disabled')

    def _handle_protocol(self, line):
        self.after(0, self._process_line, line)

    def _process_line(self, line):
        if line == "DISCONNECTED":
            messagebox.showwarning("Connection Lost", "Disconnected from server")
            self.quit()
            return
        parts = shlex.split(line)
        if not parts: return
        code = parts[0]
        data = parts[1:]
        if not code.startswith("6") and self.pending_commands:
            cmd_type, target_uuid, extra = self.pending_commands.popleft()
            if code == "200" or code == "201":
                if cmd_type == "USE": return
                if cmd_type == "LOGIN":
                    self.current_user_uuid = data[0]
                    self._setup_main_ui()
                elif cmd_type == "USERS":
                    self.dm_listbox.delete(0, tk.END)
                    for i in range(0, len(data), 3):
                        uuid, name, status = data[i], data[i+1], data[i+2]
                        self.users[uuid] = name
                        if uuid == self.current_user_uuid: continue
                        status_str = "● Online" if status == "1" else "○ Offline"
                        self.dm_listbox.insert(tk.END, f"{name} ({status_str})")
                        self._load_dm_messages(uuid)
                elif cmd_type == "LIST_TEAMS":
                    if len(data) > 0 and data[0] == "TEAM":
                        items = data[1:]
                        for i in range(0, len(items), 3):
                            u, n, d = items[i], items[i+1], items[i+2]
                            self.teams[u] = {"name": n, "desc": d, "subscribed": False}
                            if not self.teams_tree.exists(u):
                                self.teams_tree.insert("", "end", iid=u, text=n, tags=("team",))
                        self.pending_commands.append(("SUBSCRIBED_TEAMS", None, None))
                        self.client.send("SUBSCRIBED")
                elif cmd_type == "SUBSCRIBED_TEAMS":
                    for uuid in data:
                        if uuid in self.teams:
                            self.teams[uuid]["subscribed"] = True
                            self.teams_tree.item(uuid, text=f"★ {self.teams[uuid]['name']}")
                elif cmd_type == "LIST_CHANNELS":
                    if len(data) > 0 and data[0] == "CHANNEL":
                        items = data[1:]
                        if target_uuid not in self.channels: self.channels[target_uuid] = {}
                        for i in range(0, len(items), 3):
                            u, n, d = items[i], items[i+1], items[i+2]
                            self.channels[target_uuid][u] = {"name": n, "desc": d}
                            if not self.teams_tree.exists(u):
                                self.teams_tree.insert(target_uuid, "end", iid=u, text=n, tags=("channel",))
                        self.teams_tree.item(target_uuid, open=True)
                elif cmd_type == "LIST_THREADS":
                    if len(data) > 0 and data[0] == "THREAD":
                        items = data[1:]
                        if target_uuid not in self.threads: self.threads[target_uuid] = {}
                        for i in range(0, len(items), 5):
                            u, a, t, title, body = items[i], items[i+1], items[i+2], items[i+3], items[i+4]
                            self.threads[target_uuid][u] = {"title": title, "body": body, "author": a, "time": t}
                            if not self.teams_tree.exists(u):
                                self.teams_tree.insert(target_uuid, "end", iid=u, text=title, tags=("thread",))
                        self.teams_tree.item(target_uuid, open=True)
                elif cmd_type == "LIST_REPLIES":
                    if len(data) > 0 and data[0] == "REPLY":
                        items = data[1:]
                        new_history = []
                        for i in range(0, len(items), 4):
                            t_uuid, a_uuid, ts, msg = items[i], items[i+1], items[i+2], items[i+3]
                            new_history.append((a_uuid, ts, msg))
                        self.replies[target_uuid] = new_history
                        if target_uuid == self.active_thread_uuid:
                            self._display_chat_history()
                elif cmd_type == "DM_MESSAGES":
                    if target_uuid:
                        new_history = []
                        for i in range(0, len(data), 3):
                            new_history.append((data[i], data[i+1], data[i+2]))
                        self.direct_messages[target_uuid] = new_history
                        if target_uuid == self.selected_dm_user_uuid:
                            self._display_chat_history()
                elif cmd_type == "SUBSCRIBE":
                    if target_uuid in self.teams:
                        self.teams[target_uuid]["subscribed"] = True
                        self.teams_tree.item(target_uuid, text=f"★ {self.teams[target_uuid]['name']}")
                        self.subscribe_btn.config(text="Unsubscribe")
                elif cmd_type == "UNSUBSCRIBE":
                    if target_uuid in self.teams:
                        self.teams[target_uuid]["subscribed"] = False
                        self.teams_tree.item(target_uuid, text=self.teams[target_uuid]['name'])
                        self.subscribe_btn.config(text="Subscribe")
                elif cmd_type.startswith("CREATE_"):
                    messagebox.showinfo("Success", f"{cmd_type[7:]} created successfully!")
                    self._refresh_all()

            elif code.startswith("4") or code.startswith("5"):
                if cmd_type != "USE":
                    messagebox.showerror("Server Error", f"Error {code} on {cmd_type}: {' '.join(data)}")
        elif code == "600": # LoggedIn
            if len(data) >= 2:
                u, n = data[0], data[1]
                self.users[u] = n
                if not self.current_user_uuid and n == self.current_user_name:
                    self.current_user_uuid = u
                    self._setup_main_ui()
                else: self._refresh_users()
        elif code == "602": # MessageReceived
            if len(data) >= 2:
                s_uuid, msg = data[0], data[1]
                ts = int(time.time())
                if s_uuid not in self.direct_messages: self.direct_messages[s_uuid] = []
                if not any(m[2] == msg and abs(int(m[1]) - ts) < 2 for m in self.direct_messages[s_uuid]):
                    self.direct_messages[s_uuid].append((s_uuid, ts, msg))
                if s_uuid == self.selected_dm_user_uuid: self._display_chat_history()
        elif code == "604": # CommentCreated
            if len(data) >= 4:
                t_uuid, thread_uuid, a_uuid, msg = data[0], data[1], data[2], data[3]
                ts = int(time.time())
                if thread_uuid not in self.replies: self.replies[thread_uuid] = []
                self.replies[thread_uuid].append((a_uuid, ts, msg))
                if thread_uuid == self.active_thread_uuid: self._display_chat_history()
        elif code in ["603", "605", "606"]: # Thread, Team, Channel created
            self._refresh_all()
