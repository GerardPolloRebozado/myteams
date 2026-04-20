import socket
import threading
from tkinter import messagebox

class MyTeamsClient:
    def __init__(self, host, port, on_event_received):
        self.host = host
        self.port = port
        self.socket = None
        self.on_event_received = on_event_received
        self.running = False
        self.buffer = ""

    def connect(self):
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.host, self.port))
            self.running = True
            threading.Thread(target=self._receive_loop, daemon=True).start()
            return True
        except Exception as e:
            messagebox.showerror("Connection Error", str(e))
            return False

    def _receive_loop(self):
        while self.running:
            try:
                data = self.socket.recv(16384).decode('utf-8')
                if not data:
                    break
                self.buffer += data
                while "\n" in self.buffer:
                    line, self.buffer = self.buffer.split("\n", 1)
                    if line.strip():
                        self.on_event_received(line.strip())
            except Exception:
                break
        self.running = False
        self.on_event_received("DISCONNECTED")

    def send(self, cmd):
        if self.socket:
            try:
                self.socket.sendall((cmd + "\n").encode('utf-8'))
            except Exception as e:
                print(f"Send error: {e}")

    def escape(self, s):
        return s.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n")
