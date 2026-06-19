import { useMemo, useState, useEffect, useRef, useCallback } from 'react'
import { useAuth } from './context/AuthContext'
import HallOfRooms from './components/HallOfRooms/HallOfRooms'
import AuthPage from './pages/AuthPage'
import { api } from './services/api'
import GameSession from './components/GameRoom/GameSession'
import Chat from './components/Chat/Chat'
import { useGatewaySession } from './hooks/useGatewaySession'
import {
  mergeGamePayload,
  handFromPayload,
  parsePlayResult,
  playerDisplayName,
} from './components/GameRoom/gameUtils'
import StatisticsPanel from './components/StatisticsPanel'
import RankingsPanel from './components/RankingsPanel'
import HistoryPanel from './components/HistoryPanel'
import MatchesPanel from './components/MatchesPanel'
import RoomLobby from './components/RoomLobby/RoomLobby'
import GameOverPanel from './components/GameOver/GameOverPanel'
import RoomSettingsModal from './components/Modals/RoomSettingsModal'
import AccountSettingsModal from './components/Modals/AccountSettingsModal'
import './App.css'

function App() {
  const { isAuthenticated, logout, token, refreshToken, updateTokens, clearSession, refresh } = useAuth()
  const [joinedRoom, setJoinedRoom] = useState(null)
  const [myProfile, setMyProfile] = useState(null)
  const [myStats, setMyStats] = useState(null) 
  const [selectedPlayer, setSelectedPlayer] = useState({
    id: null,
    name: '',
    stats: null,
    loading: false,
    error: null,
  })
  const [activeTab, setActiveTab] = useState('rooms')
  const [selectedRoomId, setSelectedRoomId] = useState(null)
  const [navWidthPercent, setNavWidthPercent] = useState(33)
  const isResizingRef = useRef(false)
  const startXRef = useRef(0)
  const startWidthRef = useRef(33)
  const bodyRef = useRef(null)

  const [gameState, setGameState] = useState(null)
  const [myCards, setMyCards] = useState([])
  const [playerNames, setPlayerNames] = useState({}) 
  const [isHydratingGame, setIsHydratingGame] = useState(false)
  const [resumeError, setResumeError] = useState(null)
  const gatewaySessionRef = useRef(null)
  const joinedRoomRef = useRef(null)
  const gameStateRef = useRef(null)
  const isHydratingGameRef = useRef(false)
  const pendingGameEventsRef = useRef([])
  
  const [gameResults, setGameResults] = useState(null)
  const [rankings, setRankings] = useState([])
  const [matchesList, setMatchesList] = useState([])

  const [isRoomSettingsOpen, setIsRoomSettingsOpen] = useState(false)
  const [roomSettings, setRoomSettings] = useState({
    name: '',
    isPublic: true,
    password: '',
    maxPlayers: 2,
  })
  const [roomSettingsError, setRoomSettingsError] = useState(null)
  const [isUpdatingRoom, setIsUpdatingRoom] = useState(false)

  const [isAccountSettingsOpen, setIsAccountSettingsOpen] = useState(false)
  const [accountSettings, setAccountSettings] = useState({
    login: '',
    displayName: '',
    password: '',
  })
  const [accountSettingsError, setAccountSettingsError] = useState(null)
  const [isUpdatingAccount, setIsUpdatingAccount] = useState(false)
  const [isPasswordUnlocked, setIsPasswordUnlocked] = useState(false)
  const [mutedUsers, setMutedUsers] = useState([])
  const [isMuting, setIsMuting] = useState(false)

  const handlers = useMemo(() => ({
    getToken: () => {
      const storedRefresh = localStorage.getItem('refresh_token');
      const normalizedRefresh = storedRefresh === 'undefined' || storedRefresh === 'null' ? null : storedRefresh;
      return {
        access_token: typeof token === 'object' ? token.access_token : token,
        refresh_token: refreshToken || normalizedRefresh,
      };
    },
    updateTokens: (newPair) => {
      if (updateTokens) updateTokens(newPair)
    },
    logout,
    clearSession,
    refresh,
  }), [token, refreshToken, updateTokens, logout, clearSession, refresh])

  const myId = useMemo(() => {
    if (!token) return null;
    try {
      const tokenString = typeof token === 'object' ? token.access_token : token;
      const [, payload] = tokenString.split('.');
      if (!payload) return null;
      return JSON.parse(atob(payload.replace(/-/g, '+').replace(/_/g, '/'))).sub || null;
    } catch { return null; }
  }, [token])

  const profileCacheRef = useRef({});

  const cacheProfiles = useCallback((profiles) => {
    if (!profiles) return
    const list = Array.isArray(profiles) ? profiles : [profiles]
    list.forEach((profile) => {
      if (!profile || profile.id == null) return
      profileCacheRef.current[String(profile.id)] = profile
    })
  }, [])

  const getCachedPlayerName = useCallback((id) => {
    const cached = profileCacheRef.current[String(id)]
    return cached ? playerDisplayName(cached) || String(id).slice(0, 8) : null
  }, [])

  const normalizeRoomPayload = useCallback((room) => {
    if (!room) return room
    if (room.room && Array.isArray(room.players)) {
      return { ...room.room, players: room.players }
    }
    return room
  }, [])

  const { ws, applyPlayResult } = useGatewaySession({
    isAuthenticated,
    token,
    myId,
    handlers,
    refresh,
    clearSession,
    normalizeRoomPayload,
    cacheProfiles,
    setMyProfile,
    setMyStats,
    setJoinedRoom,
    setGameState,
    setMyCards,
    setGameResults,
    setResumeError,
    setIsHydratingGame,
    joinedRoomRef,
    gatewaySessionRef,
    gameStateRef,
    pendingGameEventsRef,
    isHydratingGameRef,
  })

  const clampNavWidthPercent = (value) => {
    const bodyWidth = bodyRef.current?.clientWidth || window.innerWidth;
    const minPercent = Math.min(100, (240 / bodyWidth) * 100);
    const maxPercent = Math.min(100, (540 / bodyWidth) * 100);
    return Math.min(maxPercent, Math.max(minPercent, value));
  };

  const startResize = (event) => {
    event.preventDefault();
    isResizingRef.current = true;
    startXRef.current = event.clientX;
    startWidthRef.current = navWidthPercent;
    window.addEventListener('mousemove', handleResize);
    window.addEventListener('mouseup', stopResize);
  };

  const handleResize = (event) => {
    if (!isResizingRef.current) return;
    const bodyWidth = bodyRef.current?.clientWidth || window.innerWidth;
    const delta = event.clientX - startXRef.current;
    const deltaPercent = (delta / bodyWidth) * 100;
    setNavWidthPercent((prev) => clampNavWidthPercent(startWidthRef.current + deltaPercent));
  };

  const stopResize = () => {
    if (!isResizingRef.current) return;
    isResizingRef.current = false;
    window.removeEventListener('mousemove', handleResize);
    window.removeEventListener('mouseup', stopResize);
  };

  useEffect(() => {
    return () => {
      window.removeEventListener('mousemove', handleResize);
      window.removeEventListener('mouseup', stopResize);
    };
  }, []);

  useEffect(() => {
    joinedRoomRef.current = joinedRoom;
  }, [joinedRoom]);

  useEffect(() => {
    gameStateRef.current = gameState;
  }, [gameState]);

  useEffect(() => {
    if (joinedRoom?.id) {
      setSelectedRoomId(joinedRoom.id);
    }
  }, [joinedRoom?.id]);

  useEffect(() => {
    if (activeTab === 'chat' && !joinedRoom) {
      setActiveTab('rooms');
    }
    if (activeTab === 'history' && !joinedRoom?.game) {
      setActiveTab('rooms');
    }
  }, [activeTab, joinedRoom]);

  useEffect(() => {
    if (!joinedRoom || !token) return;

    const idsToResolve = new Set();
    for (const player of joinedRoom.players || []) {
      idsToResolve.add(String(player.id));
    }
    for (const id of gameState?.players || []) {
      const playerId = typeof id === 'string' ? id : id?.id;
      if (playerId) idsToResolve.add(String(playerId));
    }

    cacheProfiles(joinedRoom.players);

    const fetchPlayerNames = async () => {
      const tokenString = typeof token === 'object' ? token.access_token : token;
      const fetched = {};
      const idsToFetch = [];

      for (const id of idsToResolve) {
        const cachedName = getCachedPlayerName(id);
        if (cachedName) {
          fetched[id] = cachedName;
        } else {
          idsToFetch.push(id);
        }
      }

      if (idsToFetch.length > 0) {
        const results = await Promise.allSettled(
          idsToFetch.map(async (id) => {
            const account = await api.getAccount(id, tokenString);
            cacheProfiles(account);
            return { id, name: playerDisplayName(account) || String(id).slice(0, 8) };
          })
        );

        for (let index = 0; index < results.length; index += 1) {
          const result = results[index];
          const id = idsToFetch[index];
          if (result.status === 'fulfilled') {
            fetched[result.value.id] = result.value.name;
          } else {
            fetched[id] = String(id).slice(0, 8);
          }
        }
      }

      setPlayerNames((prev) => {
        const next = { ...prev };
        let updated = false;

        for (const player of joinedRoom.players || []) {
          const fromRoom = playerDisplayName(player);
          const key = String(player.id);
          if (fromRoom && next[key] !== fromRoom) {
            next[key] = fromRoom;
            updated = true;
          }
        }

        for (const [id, name] of Object.entries(fetched)) {
          if (!next[id] || next[id] !== name) {
            next[id] = name;
            updated = true;
          }
        }

        return updated ? next : prev;
      });
    };
    fetchPlayerNames();
  }, [joinedRoom, gameState?.players, token]);

  useEffect(() => {
    if (activeTab === 'rankings' && token) {
      const fetchRankingsData = async () => {
        try {
          const tokenString = typeof token === 'object' ? token.access_token : token;
          const data = await api.getRankings(tokenString);
          setRankings(data);

          const next = { ...playerNames };
          let updated = false;
          const idsToFetch = [];

          for (const rank of data) {
            const key = String(rank.id);
            if (!next[key]) {
              const cachedName = getCachedPlayerName(rank.id);
              if (cachedName) {
                next[key] = cachedName;
                updated = true;
              } else {
                idsToFetch.push(rank.id);
              }
            }
          }

          if (idsToFetch.length > 0) {
            const results = await Promise.allSettled(
              idsToFetch.map(async (id) => {
                const account = await api.getAccount(id, tokenString);
                cacheProfiles(account);
                return { id, name: playerDisplayName(account) || String(id).slice(0, 8) };
              })
            );

            for (let index = 0; index < results.length; index += 1) {
              const result = results[index];
              const id = idsToFetch[index];
              if (result.status === 'fulfilled') {
                next[String(result.value.id)] = result.value.name;
              } else {
                next[String(id)] = String(id).slice(0, 8);
              }
              updated = true;
            }
          }

          if (updated) setPlayerNames(next);
        } catch (e) {
          console.error("Failed to fetch rankings:", e);
        }
      };
      fetchRankingsData();
    }
    if (activeTab === 'matches' && token && !joinedRoom?.game) {
      const fetchMatches = async () => {
        try {
          const data = await api.getGamesHistory(handlers, null, 100);
          setMatchesList(Array.isArray(data) ? data : []);
        } catch (e) {
          console.error('Failed to fetch matches list:', e);
          setMatchesList([]);
        }
      };
      fetchMatches();
    }
  }, [activeTab, token]);

  useEffect(() => {
    if (isAuthenticated && token) {
      const fetchMuted = async () => {
        try {
          const list = await api.getMutedUsers(handlers);
          setMutedUsers((list || []).map(u => String(u.blocked_id)));
        } catch (e) {
          console.error("Failed to fetch muted users:", e);
        }
      };
      fetchMuted();
    } else {
      setMutedUsers([]);
    }
  }, [isAuthenticated, token, handlers]);

  const handleToggleMute = async (userId) => {
    if (!userId) return;
    const userIdStr = String(userId);
    const isMuted = mutedUsers.includes(userIdStr);
    setIsMuting(true);
    try {
      if (isMuted) {
        await api.unmutePlayer(handlers, userIdStr);
        setMutedUsers(prev => prev.filter(id => id !== userIdStr));
      } else {
        await api.mutePlayer(handlers, userIdStr);
        setMutedUsers(prev => [...prev, userIdStr]);
      }
    } catch (e) {
      console.error("Failed to toggle mute:", e);
      alert(`Failed to ${isMuted ? 'unmute' : 'mute'} player: ${e.message}`);
    } finally {
      setIsMuting(false);
    }
  };

  const openSavedGame = async (gameId) => {
    if (!gameId) return;
    try {
      const game = await api.getGameStateById(handlers, gameId);
      if (game) {
        setGameState(game);
        setActiveTab('history');
      } else {
        alert('Failed to load selected game.');
      }
    } catch (e) {
      console.error('Failed to open saved game:', e);
      alert('Failed to open saved game.');
    }
  };

  

  const handleLeaveRoom = async () => {
    if (!joinedRoom) return;
    try {
      await api.leaveRoom(handlers, joinedRoom.id);
      setJoinedRoom(null);
      setSelectedRoomId(null);
      setGameState(null);
      setGameResults(null);
      setMyCards([]);
      pendingGameEventsRef.current = [];
      setActiveTab('rooms');
    } catch (e) { console.error("Failed to leave:", e); }
  };

  const handleKickPlayer = async (playerId) => {
    if (!joinedRoom || !playerId) return;
    try {
      await api.kickRoomPlayer(handlers, joinedRoom.id, playerId);
      setJoinedRoom((prev) =>
        prev ? { ...prev, players: prev.players.filter((player) => player.id !== playerId) } : prev,
      );
    } catch (e) {
      console.error('Failed to kick player:', e);
      alert('Could not remove player from room.');
    }
  };

  const openPlayerStatistics = useCallback(async (playerId) => {
    if (!playerId) return;
    const idString = String(playerId);
    setSelectedPlayer({
      id: idString,
      name: playerNames[idString] || '',
      stats: null,
      loading: true,
      error: null,
    });
    setActiveTab('statistics');

    const tokenString = typeof token === 'object' ? token.access_token : token;
    let displayName = playerNames[idString] || '';

    if (!displayName) {
      try {
        const account = await api.getAccount(idString, tokenString);
        cacheProfiles(account);
        displayName = playerDisplayName(account) || account.login || idString.slice(0, 8);
        setPlayerNames((prev) => ({ ...prev, [idString]: displayName }));
      } catch (err) {
        console.warn('Failed to resolve player display name for stats:', err);
        displayName = idString.slice(0, 8);
      }
    }
    setSelectedPlayer((prev) => ({ ...prev, name: displayName }));

    if (idString === String(myId) && myStats) {
      setSelectedPlayer((prev) => ({ ...prev, stats: myStats, loading: false }));
      return;
    }

    try {
      const savefile = await api.getSavefile(idString, tokenString);
      setSelectedPlayer((prev) => ({ ...prev, stats: savefile }));
    } catch (err) {
      const errorText = err?.message || 'Failed to load statistics';
      if (idString === String(myId) && myStats) {
        setSelectedPlayer((prev) => ({ ...prev, stats: myStats }));
      } else {
        setSelectedPlayer((prev) => ({ ...prev, error: errorText }));
      }
    } finally {
      setSelectedPlayer((prev) => ({ ...prev, loading: false }));
    }
  }, [token, playerNames, myStats, myId, cacheProfiles]);

  const openAccountSettings = () => {
    setAccountSettings({
      login: myProfile?.login || '',
      displayName: myProfile?.display_name || '',
      password: '',
    });
    setAccountSettingsError(null);
    setIsPasswordUnlocked(false);
    setIsAccountSettingsOpen(true);
  };

  const closeAccountSettings = () => {
    setIsAccountSettingsOpen(false);
    setAccountSettings({
      login: '',
      displayName: '',
      password: '',
    });
    setAccountSettingsError(null);
    setIsPasswordUnlocked(false);
  };

  const handleUpdateAccount = async () => {
    if (!myProfile?.id) return;
    setIsUpdatingAccount(true);
    setAccountSettingsError(null);
    try {
      const updated = await api.updateAccount(
        handlers,
        myProfile.id,
        accountSettings.login || undefined,
        accountSettings.displayName || undefined,
        accountSettings.password || undefined,
      );
      setMyProfile(updated);
      closeAccountSettings();
    } catch (e) {
      let errorMsg = e.message || 'Failed to update account';
      if (errorMsg && (errorMsg.includes('{') || errorMsg.includes('['))) {
        try {
          const parsed = JSON.parse(errorMsg);
          if (Array.isArray(parsed) && parsed.length > 0) {
            errorMsg = parsed[0].message || parsed[0].error_code || 'Failed to update account';
          } else if (typeof parsed === 'object') {
            errorMsg = parsed.message || parsed.error_code || errorMsg;
          }
        } catch {
          
        }
      }
      setAccountSettingsError(errorMsg);
    } finally {
      setIsUpdatingAccount(false);
    }
  };

  const openRoomSettings = () => {
    if (joinedRoom) {
      setRoomSettings({
        name: joinedRoom.name || '',
        isPublic: joinedRoom.is_public !== undefined ? joinedRoom.is_public : true,
        password: '',
        maxPlayers: joinedRoom.max_players || 2,
      });
      setRoomSettingsError(null);
      setIsRoomSettingsOpen(true);
    }
  };

  const closeRoomSettings = () => {
    setIsRoomSettingsOpen(false);
    setRoomSettings({
      name: '',
      isPublic: true,
      password: '',
      maxPlayers: 2,
    });
    setRoomSettingsError(null);
  };

  const handleUpdateRoom = async () => {
    if (!joinedRoom) return;
    setIsUpdatingRoom(true);
    setRoomSettingsError(null);
    try {
      const updated = await api.updateRoom(
        handlers,
        joinedRoom.id,
        roomSettings.name,
        roomSettings.isPublic,
        roomSettings.password || undefined,
        roomSettings.maxPlayers,
      );
      setJoinedRoom((prev) =>
        prev ? { ...prev, ...updated } : prev,
      );
      closeRoomSettings();
    } catch (e) {
      let errorMsg = e.message || 'Failed to update room';
      
      if (errorMsg && (errorMsg.includes('{') || errorMsg.includes('['))) {
        try {
          const parsed = JSON.parse(errorMsg);
          
          if (Array.isArray(parsed) && parsed.length > 0) {
            errorMsg = parsed[0].message || parsed[0].error_code || 'Failed to update room';
          } else if (typeof parsed === 'object') {
            
            errorMsg = parsed.message || parsed.error_code || errorMsg;
          }
        } catch {
          
        }
      }
      setRoomSettingsError(errorMsg);
    } finally {
      setIsUpdatingRoom(false);
    }
  };



  if (!isAuthenticated) return <AuthPage />;

  const displayName = myProfile
    ? (myProfile.display_name || myProfile.login)
    : (isAuthenticated ? 'Player' : 'Loading...');

  const handleRoomJoin = (room) => {
    const normalized = normalizeRoomPayload(room);
    setJoinedRoom(normalized);
    setSelectedRoomId(normalized.id);
    gatewaySessionRef.current = {
      ...(gatewaySessionRef.current || {}),
      room: normalized.id,
    };
  };

  return (
    <div id="menuUser" className="app-container">
      <header className="app-topbar">
        <h1 id="user" onClick={() => { setSelectedPlayer(prev => ({ ...prev, id: null })); setActiveTab('statistics'); }}>
          {displayName}
        </h1>
        <nav id="headers" className="app-topbar__nav" aria-label="Main">
          <div
            className={`position ${activeTab === 'statistics' ? 'active' : ''}`}
            onClick={() => { setSelectedPlayer(prev => ({ ...prev, id: null })); setActiveTab('statistics'); }}
          >
            <p>Statistics</p>
          </div>
          <div
            className={`position ${activeTab === 'rooms' ? 'active' : ''}`}
            onClick={() => setActiveTab('rooms')}
          >
            <p id="bthRoom">Rooms</p>
          </div>
          {joinedRoom && (
            <div
              className={`position ${activeTab === 'chat' ? 'active' : ''}`}
              onClick={() => setActiveTab('chat')}
            >
              <p>Chat</p>
            </div>
          )}
          {joinedRoom?.game && (
            <div
              className={`position ${activeTab === 'history' ? 'active' : ''}`}
              onClick={() => setActiveTab('history')}
            >
              <p>History</p>
            </div>
          )}
          <div
            className={`position ${activeTab === 'rankings' ? 'active' : ''}`}
            onClick={() => setActiveTab('rankings')}
          >
            <p>Rankings</p>
          </div>
          <div
            className={`position ${activeTab === 'matches' ? 'active' : ''}`}
            onClick={() => setActiveTab('matches')}
          >
            <p>Matches</p>
          </div>
        </nav>
        <div className="position app-topbar__exit" onClick={logout}>
          <p id="bthExit">Exit</p>
        </div>
      </header>

      <div className="app-body" ref={bodyRef}>
        <aside className="panel-nav" aria-label="Navigation panel" style={{ width: `${navWidthPercent}%`, minWidth: '240px', maxWidth: '540px' }}>
          {activeTab === 'rooms' && (
            <HallOfRooms
              onJoin={handleRoomJoin}
              onSelectRoom={setSelectedRoomId}
              selectedRoomId={selectedRoomId}
              joinedRoomId={joinedRoom?.id ?? null}
              ws={ws}
              myId={myId}
              handlers={handlers}
            />
          )}

          {activeTab === 'statistics' && (
            <StatisticsPanel
              displayName={selectedPlayer.id ? selectedPlayer.name : displayName}
              stats={selectedPlayer.id ? selectedPlayer.stats : myStats}
              isLoading={selectedPlayer.loading}
              error={selectedPlayer.error}
              isOwnProfile={!selectedPlayer.id || String(selectedPlayer.id) === String(myId)}
              onOpenAccountSettings={openAccountSettings}
              isMuted={selectedPlayer.id ? mutedUsers.includes(String(selectedPlayer.id)) : false}
              onToggleMute={() => handleToggleMute(selectedPlayer.id)}
              isMuting={isMuting}
            />
          )}

          {activeTab === 'rankings' && (
            <RankingsPanel
              rankings={rankings}
              playerNames={playerNames}
              myId={myId}
              onOpenPlayerStats={openPlayerStatistics}
            />
          )}

          {activeTab === 'chat' && joinedRoom && (
            <div className="panel-nav__body panel-nav__body--chat">
              <Chat
                ws={ws}
                handlers={handlers}
                myId={myId}
                myProfile={myProfile}
                roomId={joinedRoom.id}
                token={token}
                getCachedPlayerName={getCachedPlayerName}
                cacheProfiles={cacheProfiles}
                onOpenPlayerStats={openPlayerStatistics}
              />
            </div>
          )}

          {activeTab === 'history' && joinedRoom?.game && (
            <HistoryPanel
              gameState={gameState}
              playerNames={playerNames}
              onOpenPlayerStats={openPlayerStatistics}
            />
          )}

          {activeTab === 'matches' && !joinedRoom?.game && (
            <MatchesPanel
              matches={matchesList}
              getCachedPlayerName={getCachedPlayerName}
              onOpenPlayerStats={openPlayerStatistics}
            />
          )}

        </aside>

        <div
          className="panel-resizer"
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize navigation panel"
          onMouseDown={startResize}
        />

        <div id="readyMenu" className="panel-main">
          {!joinedRoom && (
            <p className="room-preview-hint">Select a room from the list on the left and join it to begin playing!</p>
          )}

          {joinedRoom && (
            <>
              {gameResults ? (
                <GameOverPanel
                  gameResults={gameResults}
                  myId={myId}
                  playerNames={playerNames}
                  onOpenPlayerStats={openPlayerStatistics}
                  onBackToLobby={() => setGameResults(null)}
                />
              ) : !joinedRoom.game ? (
                <RoomLobby
                  room={joinedRoom}
                  myId={myId}
                  playerNames={playerNames}
                  handlers={handlers}
                  onLeaveRoom={handleLeaveRoom}
                  onOpenRoomSettings={openRoomSettings}
                  onOpenPlayerStats={openPlayerStatistics}
                  onKickPlayer={handleKickPlayer}
                />
              ) : isHydratingGame || !gameState ? (
                <div id="gameBlock">
                  <p>{resumeError || 'Loading game...'}</p>
                  {resumeError && (
                    <button
                      type="button"
                      className="draw-btn"
                      style={{ marginTop: '10px' }}
                      onClick={() => {
                        setResumeError(null);
                        setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev));
                      }}
                    >
                      Back to lobby
                    </button>
                  )}
                </div>
              ) : (
                <div id="gameBlock" className="game-session-panel">
                  <GameSession
                    handlers={handlers}
                    gameId={gameState?.id}
                    myId={myId}
                    game={gameState}
                    myCards={myCards}
                    playerNames={playerNames}
                    roomPlayers={joinedRoom.players || []}
                    onPlayResult={applyPlayResult}
                    onOpenPlayerStats={openPlayerStatistics}
                        onOpenChat={() => setActiveTab('chat')}
                    onOpenHistory={() => setActiveTab('history')}
                    onLeaveRoom={handleLeaveRoom}
                    onForceLobby={() => {
                      setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev));
                      setGameState(null);
                      setMyCards([]);
                    }}
                  />
                </div>
              )}
            </>
          )}
        </div>

        <RoomSettingsModal
          isOpen={isRoomSettingsOpen}
          settings={roomSettings}
          error={roomSettingsError}
          isUpdating={isUpdatingRoom}
          onChange={setRoomSettings}
          onApply={handleUpdateRoom}
          onClose={closeRoomSettings}
        />

        <AccountSettingsModal
          isOpen={isAccountSettingsOpen}
          settings={accountSettings}
          error={accountSettingsError}
          isUpdating={isUpdatingAccount}
          isPasswordUnlocked={isPasswordUnlocked}
          onChange={setAccountSettings}
          onUnlockPassword={() => setIsPasswordUnlocked(true)}
          onApply={handleUpdateAccount}
          onClose={closeAccountSettings}
        />
      </div>
    </div>
  )
}

export default App