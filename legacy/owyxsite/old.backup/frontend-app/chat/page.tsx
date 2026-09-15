"use client";

import { useEffect, useRef, useState } from "react";
import { io, Socket } from "socket.io-client";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";
import Link from "next/link";

interface Room {
  id: number;
  name: string;
  slug: string;
  description?: string;
}

interface ChatMessage {
  id: number;
  room_id: number;
  user_id: number;
  content: string;
  created_at: string;
  nickname?: string;
}

export default function ChatPage() {
  const { isAuth, loading: authLoading, user } = useAuth({ requireAuth: false });
  const [rooms, setRooms] = useState<Room[]>([]);
  const [roomId, setRoomId] = useState<number | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [text, setText] = useState("");
  const [connected, setConnected] = useState(false);
  const [typingUser, setTypingUser] = useState<string | null>(null);
  const socketRef = useRef<Socket | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  const socketUrl =
    process.env.NEXT_PUBLIC_SOCKET_URL ||
    (typeof window !== "undefined" ? window.location.origin : "");

  useEffect(() => {
    if (!isAuth) return;
    async function loadRooms() {
      const res = await fetch("/api/chat/rooms", {
        headers: { Authorization: `Bearer ${localStorage.getItem("auth_token")}` },
      });
      if (res.ok) {
        const data = await res.json();
        setRooms(data.rooms ?? []);
        if (data.rooms?.[0]) setRoomId(data.rooms[0].id);
      }
    }
    loadRooms();
  }, [isAuth]);

  useEffect(() => {
    if (!isAuth || !roomId) return;
    async function loadMessages() {
      const res = await fetch(`/api/chat/rooms/${roomId}/messages`, {
        headers: { Authorization: `Bearer ${localStorage.getItem("auth_token")}` },
      });
      if (res.ok) {
        const data = await res.json();
        setMessages(data.messages ?? []);
      }
    }
    loadMessages();
  }, [isAuth, roomId]);

  useEffect(() => {
    if (!isAuth) return;
    const token = localStorage.getItem("auth_token");
    if (!token) return;

    const socket = io(socketUrl, {
      path: "/socket.io",
      auth: { token },
      transports: ["websocket", "polling"],
    });
    socketRef.current = socket;

    socket.on("connect", () => setConnected(true));
    socket.on("disconnect", () => setConnected(false));
    socket.on("new_message", (msg: ChatMessage) => {
      setMessages((prev) => (prev.some((m) => m.id === msg.id) ? prev : [...prev, msg]));
    });
    socket.on("typing", (payload: { user?: { nickname?: string }; isTyping?: boolean; roomId?: number }) => {
      if (payload.roomId !== roomId) return;
      setTypingUser(payload.isTyping ? payload.user?.nickname || "Кто-то" : null);
    });

    return () => {
      socket.disconnect();
      socketRef.current = null;
    };
  }, [isAuth, socketUrl, roomId]);

  useEffect(() => {
    if (!socketRef.current || !roomId || !connected) return;
    socketRef.current.emit("join_room", roomId);
  }, [roomId, connected]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  function send() {
    if (!text.trim() || !roomId || !socketRef.current) return;
    const content = text.trim();
    setText("");
    socketRef.current.emit("send_message", { roomId, content }, () => {});
    socketRef.current.emit("typing", { roomId, isTyping: false });
  }

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center text-gray-400">
        Загрузка...
      </div>
    );
  }

  if (!isAuth) {
    return (
      <>
        <Header />
        <main className="min-h-[calc(100vh-64px)] flex items-center justify-center px-4">
          <div className="glass-effect rounded-xl p-8 text-center max-w-md">
            <h1 className="text-2xl font-bold text-[#FFAA00] mb-3">Чат</h1>
            <p className="text-gray-400 mb-6">Войдите, чтобы подключиться к чату</p>
            <Link href="/login" className="btn-minecraft px-6 py-3 rounded-lg font-bold inline-block">
              Войти
            </Link>
          </div>
        </main>
        <Footer />
      </>
    );
  }

  return (
    <>
      <Header />
      <main className="min-h-[calc(100vh-64px)] py-8 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="flex items-center justify-between mb-4">
            <h1 className="text-2xl font-bold text-[#FFFF55] text-shadow">Чат</h1>
            <span className={`text-xs px-2 py-1 rounded ${connected ? "bg-green-500/20 text-green-400" : "bg-red-500/20 text-red-400"}`}>
              {connected ? "Online" : "Offline"}
            </span>
          </div>

          <div className="flex gap-2 mb-4 flex-wrap">
            {rooms.map((r) => (
              <button
                key={r.id}
                onClick={() => setRoomId(r.id)}
                className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${
                  roomId === r.id
                    ? "border-[#FFAA00] text-[#FFAA00] bg-[#FFAA00]/10"
                    : "border-[#FFAA00]/20 text-gray-400 hover:border-[#FFAA00]/40"
                }`}
              >
                {r.name}
              </button>
            ))}
          </div>

          <div className="glass-effect rounded-xl border border-[#FFAA00]/20 flex flex-col h-[60vh]">
            <div className="flex-1 overflow-y-auto p-4 space-y-3">
              {messages.map((m) => (
                <div
                  key={m.id}
                  className={`text-sm ${m.user_id === user?.id ? "text-right" : ""}`}
                >
                  <div className="text-xs text-gray-500 mb-0.5">
                    {m.nickname || "Игрок"} · {new Date(m.created_at).toLocaleTimeString("ru-RU")}
                  </div>
                  <div
                    className={`inline-block px-3 py-2 rounded-lg max-w-[80%] ${
                      m.user_id === user?.id
                        ? "bg-[#FFAA00]/20 text-[#FFFF55]"
                        : "bg-[#2a2a2a] text-gray-200"
                    }`}
                  >
                    {m.content}
                  </div>
                </div>
              ))}
              <div ref={bottomRef} />
            </div>
            {typingUser && (
              <div className="px-4 text-xs text-gray-500 pb-1">{typingUser} печатает...</div>
            )}
            <div className="p-3 border-t border-[#FFAA00]/20 flex gap-2">
              <input
                value={text}
                onChange={(e) => {
                  setText(e.target.value);
                  socketRef.current?.emit("typing", { roomId, isTyping: true });
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    send();
                  }
                }}
                placeholder="Сообщение..."
                className="flex-1 px-3 py-2 bg-[#2a2a2a] border border-[#FFAA00]/20 rounded-lg text-white focus:outline-none focus:border-[#FFAA00]"
              />
              <button
                onClick={send}
                disabled={!text.trim() || !connected}
                className="btn-minecraft px-5 py-2 rounded-lg font-bold disabled:opacity-50"
              >
                →
              </button>
            </div>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}
