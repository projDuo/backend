
export function parseGameId(event) {
  if (event == null) return null;
  if (typeof event === 'string') return event;
  if (typeof event !== 'object') return null;
  return event.game_id ?? event.gameId ?? event.id ?? null;
}

export function parseRoomId(event) {
  if (event == null) return null;
  if (typeof event === 'string') return event;
  if (typeof event !== 'object') return null;
  return event.room_id ?? event.roomId ?? event.id ?? null;
}

export function playerIdAtTurn(game) {
  if (!game || game.turn == null) return null;
  const slot = game.players?.[game.turn];
  if (slot == null) return null;
  return typeof slot === 'string' ? slot : slot?.id ?? null;
}

export function playersForDisplay(game) {
  if (!game) return [];
  const activeById = Object.fromEntries(
    (game.players_active || []).map((p) => [String(p.id), p])
  );

  if (Array.isArray(game.players) && game.players.length > 0) {
    return game.players.map((entry) => {
      const id = typeof entry === 'string' ? entry : entry?.id;
      return activeById[String(id)] || { id, cards: 0 };
    });
  }

  return game.players_active || [];
}

export function mergeGamePayload(prev, incoming) {
  if (!incoming) return prev;
  const players =
    incoming.players ??
    prev?.players ??
    (incoming.players_active?.map((p) => p.id) ?? []);

  const hasIncomingHistory = Array.isArray(incoming.history);
  const incomingHistory = hasIncomingHistory ? incoming.history : undefined;
  const preservedHistory = !hasIncomingHistory && Array.isArray(prev?.history) ? prev.history : undefined;

  const history = (() => {
    if (incomingHistory) return incomingHistory;
    if (!preservedHistory || !incoming.card) return preservedHistory;

    if (
      prev &&
      incoming.turn_enforced_at != null &&
      prev.turn_enforced_at === incoming.turn_enforced_at
    ) {
      return preservedHistory;
    }

    const rawPreviousPlayer = Array.isArray(prev.players)
      ? prev.players[prev.turn]
      : undefined;

    const previousPlayer = rawPreviousPlayer == null
      ? undefined
      : typeof rawPreviousPlayer === 'object'
      ? rawPreviousPlayer.id ?? rawPreviousPlayer
      : rawPreviousPlayer;

    if (previousPlayer == null) return preservedHistory;

    return [
      ...preservedHistory,
      {
        player: previousPlayer,
        card: incoming.card,
      },
    ];
  })();

  return {
    ...(prev || {}),
    ...incoming,
    players,
    ...(history ? { history } : {}),
  };
}

export function handFromPayload(payload) {
  return Array.isArray(payload?.hand) ? payload.hand : null;
}

export function playerDisplayName(playerOrAccount) {
  if (!playerOrAccount) return null;
  const displayName = playerOrAccount.display_name;
  if (displayName != null && String(displayName).trim()) {
    return String(displayName).trim();
  }
  const login = playerOrAccount.login;
  if (login != null && String(login).trim()) {
    return String(login).trim();
  }
  return null;
}

export function playerNameFromSources(id, namesById = {}, roomPlayers = []) {
  const key = String(id);
  const roomPlayer = roomPlayers.find((p) => String(p.id) === key);
  const fromRoom = playerDisplayName(roomPlayer);
  if (fromRoom) return fromRoom;
  const fromMap = namesById[key] ?? namesById[id];
  if (fromMap) return fromMap;
  return key ? key.slice(0, 8) : '?';
}

export const ELEMENT_COLORS = {
  Fire: '#ff4e18',
  Water: '#2563eb',
  Air: '#93c5fd',
  Wood: '#2d6a4f',
  Earth: '#a16207',
  Energy: '#1a1a1a',
};

export function elementColor(element) {
  const name = typeof element === 'string' ? element : element?.element ?? null;
  if (!name) return 'var(--bg-pane-light)';
  return ELEMENT_COLORS[name] ?? 'var(--bg-pane-light)';
}

export function seatsAroundTable(game, myId) {
  const players = playersForDisplay(game);
  if (!players.length) return { bottom: null, top: null, left: null, right: null };

  const myIndex = players.findIndex((p) => String(p.id) === String(myId));
  const start = myIndex >= 0 ? myIndex : 0;
  const rotated = [...players.slice(start), ...players.slice(0, start)];

  if (rotated.length === 1) {
    return { bottom: rotated[0], top: null, left: null, right: null };
  }
  if (rotated.length === 2) {
    return { bottom: rotated[0], top: rotated[1], left: null, right: null };
  }
  if (rotated.length === 3) {
    return { bottom: rotated[0], left: rotated[1], right: rotated[2], top: null };
  }
  return {
    bottom: rotated[0],
    left: rotated[1],
    top: rotated[2],
    right: rotated[3],
  };
}

export function parsePlayResult(result) {
  if (!result || typeof result !== 'object') return null;
  if (result.NextTurn) return { kind: 'turn', payload: result.NextTurn };
  if (result.GameOver) return { kind: 'over', payload: result.GameOver };
  if (result.id != null && result.turn != null && result.card != null) {
    return { kind: 'turn', payload: result };
  }
  if (result.leaderboard != null && result.id != null) {
    return { kind: 'over', payload: result };
  }
  return null;
}
