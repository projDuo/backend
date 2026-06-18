import React, { useState, useEffect, useRef } from 'react';
import { api } from '../../services/api';
import './Chat.css';

export default function Chat({ ws, handlers, myId, myProfile, roomId, token, getCachedPlayerName, cacheProfiles, onOpenPlayerStats }) {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [playerNames, setPlayerNames] = useState({});
  const scrollRef = useRef();
  const [busy, setBusy] = useState(false);

  const getCachedName = (playerId) => {
    return playerNames[playerId] || getCachedPlayerName?.(playerId) || null;
  };

  const fetchPlayerName = async (playerId) => {
    if (!playerId) return String(playerId).slice(0, 8);
    const cached = getCachedName(playerId);
    if (cached) return cached;

    try {
      const tokenString = typeof token === 'object' ? token.access_token : token;
      const account = await api.getAccount(playerId, tokenString);
      const login = account.login || account.display_name || playerId.slice(0, 8);
      cacheProfiles?.(account);
      setPlayerNames((prev) => ({ ...prev, [playerId]: login }));
      return login;
    } catch (err) {
      const fallback = playerId.slice(0, 8);
      setPlayerNames((prev) => ({ ...prev, [playerId]: fallback }));
      return fallback;
    }
  };

  useEffect(() => {
    if (!ws) return;

    const onMessagePosted = async (msg) => {
      if (!msg?.author) return;
      if (!getCachedName(msg.author)) {
        await fetchPlayerName(msg.author);
      }

      setMessages((prev) => {
        if (prev.some((m) => m.id === msg.id)) return prev;
        return [...prev, msg];
      });
    };

    const onMessageEdited = async (msg) => {
      if (!msg?.author) return;
      if (!getCachedName(msg.author)) {
        await fetchPlayerName(msg.author);
      }

      setMessages((prev) => prev.map((m) => (m.id === msg.id ? msg : m)));
    };

    const onMessageDeleted = (deletedId) => {
      setMessages((prev) => prev.filter((m) => m.id !== deletedId));
    };

    ws.on('MessagePosted', onMessagePosted);
    ws.on('MessageEdited', onMessageEdited);
    ws.on('MessageDeleted', onMessageDeleted);
    return () => {
      ws.off('MessagePosted', onMessagePosted);
      ws.off('MessageEdited', onMessageEdited);
      ws.off('MessageDeleted', onMessageDeleted);
    };
  }, [ws, getCachedPlayerName, playerNames]);

  useEffect(() => {
    let cancelled = false;
    if (!roomId || !handlers) return;

    (async () => {
      setBusy(true);
      try {
        const history = await api.getChatMessages(handlers, roomId, 0, 100);
        if (cancelled) return;
        const chatMessages = Array.isArray(history) ? history : [];
        setMessages(chatMessages);

        const authors = Array.from(
          new Set(chatMessages.map((msg) => String(msg.author)).filter(Boolean))
        );
        const missingAuthors = authors.filter((id) => !getCachedName(id));

        if (missingAuthors.length > 0) {
          await Promise.allSettled(missingAuthors.map((id) => fetchPlayerName(id)));
        }
      } catch (e) {
        console.error('Failed to load chat history:', e);
      } finally {
        if (!cancelled) setBusy(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [roomId, handlers, getCachedPlayerName, playerNames]);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [messages]);

  const sendMessage = async (e) => {
    e.preventDefault();
    if (!input.trim()) return;
    try {
      const msg = await api.sendChatMessage(handlers, roomId, input);
      setMessages((prev) => {
        if (!msg?.id) return [...prev];
        if (prev.some((m) => m.id === msg.id)) return prev;
        return [...prev, msg];
      });
      setInput("");
    } catch (e) {
      alert("Message failed to send. You might be unauthorized or channel is closed.");
    }
  };

  return (
    <div className="chat-container">
      <h2>Chat</h2>
      <div className="chat-messages" ref={scrollRef}>
        {busy && messages.length === 0 ? <p>Loading chat...</p> : null}
        {messages.map((m) => (
          <div key={m.id} className="chat-msg">
            <div className="chat-message-body">
              <span className="chat-author">
                <button
                  type="button"
                  onClick={() => onOpenPlayerStats?.(m.author)}
                  style={{
                    border: 'none',
                    background: 'transparent',
                    padding: 0,
                    margin: 0,
                    cursor: onOpenPlayerStats ? 'pointer' : 'default',
                    color: 'inherit',
                    font: 'inherit',
                    textDecoration: 'underline',
                  }}
                >
                  {playerNames[m.author] || getCachedPlayerName?.(m.author) || String(m.author).slice(0, 8)}
                </button>
                :
              </span>
              <span className="chat-content">{m.content}</span>
              {m.edited_at ? <span className="chat-edited">(edited)</span> : null}
            </div>
            {String(m.author) === String(myId) && (
              <div className="chat-actions">
                <button
                  className="chat-edit-btn"
                  type="button"
                  onClick={async () => {
                    const next = window.prompt('Edit your message:', m.content);
                    if (next == null) return;
                    try {
                      const updated = await api.editChatMessage(handlers, roomId, m.id, next);
                      setMessages((prev) => prev.map((x) => (x.id === m.id ? updated : x)));
                    } catch (err) {
                      alert('Failed to edit message.');
                    }
                  }}
                >
                  Edit
                </button>
                <button
                  className="chat-delete-btn"
                  type="button"
                  onClick={async () => {
                    const ok = window.confirm('Delete this message?');
                    if (!ok) return;
                    try {
                      await api.deleteChatMessage(handlers, roomId, m.id);
                      setMessages((prev) => prev.filter((x) => x.id !== m.id));
                    } catch (err) {
                      alert('Failed to delete message.');
                    }
                  }}
                >
                  Delete
                </button>
              </div>
            )}
          </div>
        ))}
      </div>
      <form onSubmit={sendMessage} className="chat-input-area">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type a message..."
          disabled={busy}
        />
        <button type="submit" disabled={busy}>
          Send
        </button>
      </form>
    </div>
  );
}