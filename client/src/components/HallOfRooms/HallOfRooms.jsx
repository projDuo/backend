import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useAuth } from '../../context/AuthContext';
import { api } from '../../services/api';
import './HallOfRooms.css';

export default function HallOfRooms({
  onJoin,
  onSelectRoom,
  selectedRoomId,
  joinedRoomId,
  ws,
  myId,
}) {
  const { token, refreshToken, updateTokens, logout, refresh, clearSession } = useAuth();
  const [rooms, setRooms] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [passwordRoom, setPasswordRoom] = useState(null);
  const [password, setPassword] = useState('');
  const [passwordError, setPasswordError] = useState(null);

  const handlers = useMemo(() => ({
    getToken: () => ({
      access_token: typeof token === 'object' ? token.access_token : token,
      refresh_token: refreshToken || localStorage.getItem('refresh_token'),
    }),
    updateTokens: (newPair) => {
      localStorage.setItem('access_token', newPair.access_token);
      localStorage.setItem('refresh_token', newPair.refresh_token);
      if (updateTokens) updateTokens(newPair);
    },
    refresh,
    clearSession,
    logout,
  }), [token, refreshToken, updateTokens, refresh, clearSession, logout]);

  const [createForm, setCreateForm] = useState({
    name: 'Duo Room',
    isPublic: true,
    password: '',
    maxPlayers: 2,
  });
  const [searchInput, setSearchInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [joinRoomId, setJoinRoomId] = useState('');
  const [isCreateOpen, setIsCreateOpen] = useState(true);
  const [isFindOpen, setIsFindOpen] = useState(true);

  useEffect(() => {
    const updateVisibility = () => {
      const isWide = window.innerWidth > 900;
      setIsCreateOpen(isWide);
      setIsFindOpen(isWide);
    };

    updateVisibility();
    window.addEventListener('resize', updateVisibility);
    return () => window.removeEventListener('resize', updateVisibility);
  }, []);

  const fetchRooms = useCallback(async (search = '') => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.getRoomsList(handlers, search, 0, 100);
      setRooms(data);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  }, [handlers]);

  
  useEffect(() => {
    fetchRooms(searchQuery);
  }, [token, refreshToken]);

  
  useEffect(() => {
    const interval = setInterval(() => {
      fetchRooms(searchQuery);
    }, 15000);

    return () => clearInterval(interval);
  }, [searchQuery, token, refreshToken]);

  
  useEffect(() => {
    if (!ws) return;

    
    const handleRoomCreate = (newRoom) => {
      setRooms(prev => {
        if (prev.some(r => r.id === newRoom.id)) return prev; 
        return [...prev, newRoom];
      });
    };

    
    const handleRoomUpdate = (updatedRoom) => {
      setRooms(prev => prev.map(r => r.id === updatedRoom.id ? updatedRoom : r));
    };

    
    const handleRoomDelete = (deletedRoomId) => {
      setRooms(prev => prev.filter(r => r.id !== deletedRoomId));
      if (selectedRoomId === deletedRoomId && onSelectRoom) {
        onSelectRoom(null);
      }
    };

    ws.on('RoomCreate', handleRoomCreate);
    ws.on('RoomUpdate', handleRoomUpdate);
    ws.on('RoomDelete', handleRoomDelete);

    return () => {
      ws.off('RoomCreate', handleRoomCreate);
      ws.off('RoomUpdate', handleRoomUpdate);
      ws.off('RoomDelete', handleRoomDelete);
    };
  }, [ws, selectedRoomId, onSelectRoom]);

  const selectRoom = (room) => {
    if (onSelectRoom) onSelectRoom(room.id);
  };

  const closePasswordModal = () => {
    setPasswordRoom(null);
    setPassword('');
    setPasswordError(null);
  };

  const handleJoinClick = async (room) => {
    if (room.password && joinedRoomId !== room.id) {
      setPasswordRoom(room.id);
      setPassword('');
      setPasswordError(null);
      return;
    }
    await doJoin(room.id, null);
  };

  const handleJoinById = async () => {
    setError(null);
    const roomId = joinRoomId.trim();
    if (!roomId) {
      setError('Enter a room ID to join');
      return;
    }

    try {
      const response = await api.getRoomById(handlers, roomId);
      const room = response.room ? response.room : response;
      const targetRoomId = room.id || roomId;

      if (room.password) {
        setPasswordRoom(targetRoomId);
        setPassword('');
        setPasswordError(null);
      } else {
        await doJoin(targetRoomId, null);
      }
    } catch (e) {
      setError(e.message || 'Failed to join room');
    }
  };

  const handleSearch = async () => {
    const trimmed = searchInput.trim();
    setSearchQuery(trimmed);
    await fetchRooms(trimmed);
  };

  const doJoin = async (roomId, pass) => {
    const fromPasswordModal = passwordRoom === roomId;
    try {
      if (fromPasswordModal) {
        setPasswordError(null);
      } else {
        setError(null);
      }
      const room = await api.joinRoom(handlers, roomId, pass);
      if (fromPasswordModal) {
        closePasswordModal();
      }
      if (onJoin) onJoin(room);
    } catch (e) {
      const message = e.message || 'Failed to join room';
      if (fromPasswordModal) {
        setPasswordError(message);
      } else {
        setError(message);
      }
    }
  };

  const handleCreate = async () => {
      try {
        setError(null);
        
        const room = await api.createRoom(
          handlers,
          createForm.name,
          createForm.isPublic,
          createForm.password,
          createForm.maxPlayers
        );
        await fetchRooms(searchQuery);
        if (onJoin) onJoin(room);
      } catch (e) {
        setError(e.message);
      }
    };

  return (
    <div className="hall-of-rooms">
      <header className="hall-header">
        <h3>Rooms</h3>
        <button className="refresh" onClick={() => fetchRooms(searchQuery)}>Refresh</button>
      </header>

      <div className="create-room-toggle">
        <button type="button" onClick={() => setIsCreateOpen((open) => !open)} aria-expanded={isCreateOpen}>
          {isCreateOpen ? 'Hide create room' : 'Show create room'}
        </button>
      </div>
      <div className="create-room-toggle find-room-toggle">
        <button type="button" onClick={() => setIsFindOpen((open) => !open)} aria-expanded={isFindOpen}>
          {isFindOpen ? 'Hide search/join' : 'Show search/join'}
        </button>
      </div>

      <div className={`create-room ${isCreateOpen ? 'create-room--open' : 'create-room--collapsed'}`}>
        <h4>Create room</h4>
        <div className="create-row">
          <input
            type="text"
            value={createForm.name}
            onChange={(e) => setCreateForm(prev => ({ ...prev, name: e.target.value }))}
            placeholder="Room name"
          />
          <label className="switch">
            <input
              type="checkbox"
              checked={createForm.isPublic}
              onChange={(e) => setCreateForm(prev => ({ ...prev, isPublic: e.target.checked }))}
            />
            Public
          </label>
        </div>
        <div className="create-row">
          <input
            type="password"
            value={createForm.password}
            onChange={(e) => setCreateForm(prev => ({ ...prev, password: e.target.value }))}
            placeholder="Room password (optional)"
          />
          <input
            type="number"
            min={2}
            max={10}
            value={createForm.maxPlayers}
            onChange={(e) => setCreateForm(prev => ({ ...prev, maxPlayers: Number(e.target.value) }))}
            placeholder="Max players"
          />
          <button onClick={handleCreate} className="join">Create</button>
        </div>
      </div>

      <div className={`find-room ${isFindOpen ? 'find-room--open' : 'find-room--collapsed'}`}>
        <h4>Search & join</h4>
        <div className="create-row">
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder="Search by name"
          />
          <button type="button" className="search" onClick={handleSearch}>
            Search
          </button>
        </div>
        <div className="create-row">
          <input
            type="text"
            value={joinRoomId}
            onChange={(e) => setJoinRoomId(e.target.value)}
            placeholder="Join by room ID"
          />
          <button type="button" className="join" onClick={handleJoinById}>
            Join
          </button>
        </div>
      </div>

      {loading && <div className="hall-empty">Loading…</div>}
      {error && <div className="hall-empty hall-error">{error}</div>}

      {!loading && rooms.length === 0 && (
        <div className="hall-empty">
          {searchQuery ? 'No rooms match search' : 'No rooms'}
        </div>
      )}

      <ul className="rooms-list">
        {rooms.map((r) => {
          return (
            <li key={r.id} className="room-item">
              <button
                type="button"
                className={`room-card${selectedRoomId === r.id ? ' room-card--selected' : ''}`}
                onClick={() => selectRoom(r)}
              >
              <div className="room-card__top">
                <span className="room-name">{r.name}</span>
                <span className="room-count">
                  {r.players || 0}/{r.max_players}
                </span>
              </div>
              <div className="room-meta">
                {r.is_public ? 'Public' : 'Private'}
                {!r.is_public && ' 🔒'}
              </div>
            </button>
            {selectedRoomId === r.id && joinedRoomId !== r.id && (
              <div className="room-right">
                <button type="button" className="join" onClick={() => handleJoinClick(r)}>
                  Join
                </button>
              </div>
            )}
          </li>
          );
        })}
      </ul>

      {passwordRoom && (
        <div className="modal-overlay" onClick={closePasswordModal}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>Enter Room Password</h3>
            <input
              type="password"
              placeholder="Password"
              value={password}
              onChange={(e) => {
                setPassword(e.target.value);
                if (passwordError) setPasswordError(null);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  doJoin(passwordRoom, password);
                }
              }}
              autoFocus
            />
            {passwordError && (
              <p className="modal-password-error" role="alert">
                {passwordError}
              </p>
            )}
            <div className="modal-buttons">
              <button type="button" className="join" onClick={() => doJoin(passwordRoom, password)}>
                Join
              </button>
              <button type="button" onClick={closePasswordModal}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
