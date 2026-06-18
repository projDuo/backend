import { useEffect, useState } from 'react'
import { api } from '../services/api'
import {
  mergeGamePayload,
  handFromPayload,
  parsePlayResult,
  parseGameId,
  parseRoomId,
  playerDisplayName,
} from '../components/GameRoom/gameUtils'
import { WebSocketManager } from '../services/websocket'

export function useGatewaySession({
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
}) {
  const [ws, setWs] = useState(null)

  const flushPendingGameEvents = () => {
    const pendingEvents = [...pendingGameEventsRef.current]
    pendingGameEventsRef.current = []

    for (const event of pendingEvents) {
      switch (event.type) {
        case 'GameStarted':
        case 'GameNewTurn':
          applyGamePayload(event.payload)
          break
        case 'GameOver':
          setGameResults(event.payload?.leaderboard ?? event.payload)
          setGameState(null)
          setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev))
          break
        default:
          break
      }
    }
  }

  const applyGamePayload = (payload) => {
    if (!payload) return
    setGameState((prev) => {
      const next = mergeGamePayload(prev, payload)
      gameStateRef.current = next
      return next
    })
    setGameResults(null)
    setResumeError(null)
    const hand = handFromPayload(payload)
    if (hand) setMyCards(hand)
    setJoinedRoom((prev) => (prev ? { ...prev, game: true } : prev))
  }

  const applyPlayResult = (result) => {
    const parsed = parsePlayResult(result)
    if (!parsed) return
    if (parsed.kind === 'turn') {
      applyGamePayload(parsed.payload)
    } else if (parsed.kind === 'over') {
      setGameResults(parsed.payload.leaderboard ?? parsed.payload)
      setGameState(null)
      setMyCards([])
      setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev))
    }
  }

  const hydrateGameState = async (gameId) => {
    const id = String(gameId)
    isHydratingGameRef.current = true
    setIsHydratingGame(true)
    setResumeError(null)
    setGameState(null)
    gameStateRef.current = null
    setMyCards([])

    try {
      const h = handlers
      const game = await api.getGameStateById(h, id)
      if (game) {
        applyGamePayload(game)
        gameStateRef.current = game
      } else {
        setResumeError('Game session is no longer available.')
        setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev))
      }
    } catch (err) {
      console.error('Failed to hydrate game state:', err)
      setResumeError(err.message || 'Failed to load game')
      setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev))
    } finally {
      isHydratingGameRef.current = false
      setIsHydratingGame(false)
      flushPendingGameEvents()
    }
  }

  useEffect(() => {
    if (!isAuthenticated || !token) return

    const tokenString = typeof token === 'object' ? token.access_token : token
    const manager = new WebSocketManager(tokenString)

    manager.connect().catch((err) => console.error('WS Connection failed:', err))
    setWs(manager)

    return () => {
      manager.disconnect()
      setWs(null)
    }
  }, [isAuthenticated, token])

  useEffect(() => {
    if (!ws) return

    const ensureRoomForSession = async (inGame = false) => {
      const roomId = gatewaySessionRef.current?.room ?? joinedRoomRef.current?.id
      if (!roomId) return
      try {
        const room = await api.getRoomById(handlers, roomId)
        const normalized = normalizeRoomPayload(room)
        setJoinedRoom({
          ...normalized,
          game: inGame || Boolean(gatewaySessionRef.current?.game),
        })
        gatewaySessionRef.current = {
          ...(gatewaySessionRef.current || {}),
          room: normalized.id,
        }
      } catch (err) {
        console.error('Failed to load room:', err)
      }
    }

    const handleReady = async (authPayload) => {
      const account = authPayload?.account ?? authPayload
      const session = authPayload?.session ?? null
      gatewaySessionRef.current = session
      const accountId = account?.id || account?.uuid || myId
      const tokenString = typeof token === 'object' ? token.access_token : token

      cacheProfiles(account)
      setMyProfile(account || { login: 'Player' })

      try {
        const [fullProfile, savefileData] = await Promise.all([
          api.getAccount(accountId, tokenString).catch(() => account),
          api.getSavefile(accountId, tokenString).catch(() => null),
        ])
        cacheProfiles(fullProfile)
        setMyProfile(fullProfile || account)
        setMyStats(savefileData)
      } catch {
        setMyProfile(account || { login: 'Player' })
      }

      if (!session?.room && !session?.game) return

      try {
        if (session.room) {
          const room = await api.getRoomById(handlers, session.room)
          setJoinedRoom({
            ...normalizeRoomPayload(room),
            game: Boolean(session.game),
          })
        }

        if (session.game) {
          if (!gameStateRef.current) {
            await hydrateGameState(session.game)
          }
        }
      } catch (err) {
        console.error('Failed to resume session from gateway activity:', err)
        setResumeError(err.message || 'Failed to resume session')
      }
    }

    const handleRoomCreate = (roomData) => {
      cacheProfiles(roomData.players)
      const amIInThisRoom = roomData.players.some((p) => p.id === myId)
      if (amIInThisRoom) setJoinedRoom(roomData)
    }

    const handlePlayerNew = async (newPlayer) => {
      if (!newPlayer?.id) return

      cacheProfiles(newPlayer)
      let enrichedPlayer = newPlayer

      if (!playerDisplayName(newPlayer) && !newPlayer.login) {
        try {
          const tokenString = typeof token === 'object' ? token.access_token : token
          const account = await api.getAccount(newPlayer.id, tokenString)
          if (account) {
            cacheProfiles(account)
            enrichedPlayer = { ...newPlayer, ...account }
          }
        } catch (err) {
          console.warn('Failed to load profile for new room player:', err)
        }
      }

      setJoinedRoom((prev) => {
        if (!prev) return prev
        if (prev.players.some((p) => p.id === newPlayer.id)) return prev
        return { ...prev, players: [...prev.players, enrichedPlayer] }
      })
    }

    const handlePlayerLeft = (leftPlayerId) => {
      setJoinedRoom((prev) =>
        prev ? { ...prev, players: prev.players.filter((p) => p.id !== leftPlayerId) } : prev,
      )
    }

    const handleRoomUpdate = (updatedRoom) => {
      cacheProfiles(updatedRoom.players)
      setJoinedRoom((prev) =>
        prev && prev.id === updatedRoom.id ? { ...prev, ...updatedRoom } : prev,
      )
    }

    const handlePlayerUpdate = (updatedPlayer) => {
      cacheProfiles(updatedPlayer)
      setJoinedRoom((prev) => {
        if (!prev) return prev
        return {
          ...prev,
          players: prev.players.map((p) => (p.id === updatedPlayer.id ? { ...p, ...updatedPlayer } : p)),
        }
      })
    }

    const handleJoinedRoom = async (event) => {
      const roomId = parseRoomId(event)
      if (!roomId) return
      try {
        const room = await api.getRoomById(handlers, roomId)
        const normalized = normalizeRoomPayload(room)
        cacheProfiles(normalized.players)
        setJoinedRoom(normalized)
        gatewaySessionRef.current = {
          ...(gatewaySessionRef.current || {}),
          room: normalized.id,
          game: gatewaySessionRef.current?.game ?? null,
        }
      } catch (err) {
        console.error('Failed to refresh room after join event:', err)
      }
    }

    const handleLeftRoomActivity = () => {
      gatewaySessionRef.current = {
        ...(gatewaySessionRef.current || {}),
        room: null,
      }
      setJoinedRoom(null)
      setGameState(null)
      setGameResults(null)
      setMyCards([])
      pendingGameEventsRef.current = []
    }

    const handleJoinedGame = async (event) => {
      const gameId = parseGameId(event)
      if (!gameId) return

      gatewaySessionRef.current = {
        ...(gatewaySessionRef.current || {}),
        game: gameId,
      }

      await ensureRoomForSession(true)

      if (!gameStateRef.current) {
        await hydrateGameState(gameId)
      }
    }

    const handleLeftGame = () => {
      gatewaySessionRef.current = {
        ...(gatewaySessionRef.current || {}),
        game: null,
      }
      setGameState(null)
      setMyCards([])
      setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev))
      pendingGameEventsRef.current = []
    }

    const onGameStarted = async (data) => {
      if (isHydratingGameRef.current) {
        pendingGameEventsRef.current.push({ type: 'GameStarted', payload: data })
        return
      }
      gatewaySessionRef.current = {
        ...(gatewaySessionRef.current || {}),
        game: data?.id ?? gatewaySessionRef.current?.game,
      }
      await ensureRoomForSession(true)
      applyGamePayload(data)
    }

    const onGameUpdate = (data) => {
      if (isHydratingGameRef.current) {
        pendingGameEventsRef.current.push({ type: 'GameNewTurn', payload: data })
        return
      }
      applyGamePayload(data)
    }

    const onAlert = (data) => console.warn('[Gateway Alert]', data)

    const onGameOver = (data) => {
      if (isHydratingGameRef.current) {
        pendingGameEventsRef.current.push({ type: 'GameOver', payload: data })
        return
      }
      setGameResults(data?.leaderboard ?? data)
      setGameState(null)
      setJoinedRoom((prev) => (prev ? { ...prev, game: false } : prev))
    }

    const handleAuthError = async () => {
      console.warn('WS Auth Error: Attempting silent re-identification...')
      try {
        const newPair = await refresh()
        console.debug('[App] WS refresh returned new token pair', {
          accessTokenLength: newPair.access_token?.length,
          refreshTokenLength: newPair.refresh_token?.length,
        })
        if (ws?.isConnected()) {
          ws.token = newPair.access_token
          ws.send({ event: 'Identify', data: { token: newPair.access_token } })
          console.debug('[App] WS re-identify sent on existing socket')
        } else if (ws) {
          ws.token = newPair.access_token
          await ws.connect()
          console.debug('[App] WS reconnected with new access token')
        } else {
          throw new Error('WebSocket manager unavailable')
        }
      } catch (err) {
        console.warn('WS token refresh failed:', err)
        clearSession()
        if (ws) ws.disconnect()
      }
    }

    ws.on('Ready', handleReady)
    ws.on('RoomCreate', handleRoomCreate)
    ws.on('RoomPlayerNew', handlePlayerNew)
    ws.on('RoomPlayerLeft', handlePlayerLeft)
    ws.on('RoomUpdate', handleRoomUpdate)
    ws.on('RoomPlayerUpdate', handlePlayerUpdate)
    ws.on('JoinedRoom', handleJoinedRoom)
    ws.on('LeftRoom', handleLeftRoomActivity)
    ws.on('AuthError', handleAuthError)
    ws.on('JoinedGame', handleJoinedGame)
    ws.on('LeftGame', handleLeftGame)
    ws.on('GameStarted', onGameStarted)
    ws.on('GameNewTurn', onGameUpdate)
    ws.on('ReportAlert', onAlert)
    ws.on('GameOver', onGameOver)

    return () => {
      ws.off('Ready', handleReady)
      ws.off('RoomCreate', handleRoomCreate)
      ws.off('RoomPlayerNew', handlePlayerNew)
      ws.off('RoomPlayerLeft', handlePlayerLeft)
      ws.off('RoomUpdate', handleRoomUpdate)
      ws.off('RoomPlayerUpdate', handlePlayerUpdate)
      ws.off('JoinedRoom', handleJoinedRoom)
      ws.off('LeftRoom', handleLeftRoomActivity)
      ws.off('AuthError', handleAuthError)
      ws.off('JoinedGame', handleJoinedGame)
      ws.off('LeftGame', handleLeftGame)
      ws.off('GameStarted', onGameStarted)
      ws.off('GameNewTurn', onGameUpdate)
      ws.off('ReportAlert', onAlert)
      ws.off('GameOver', onGameOver)
    }
  }, [
    ws,
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
  ])

  return { ws, applyPlayResult }
}
