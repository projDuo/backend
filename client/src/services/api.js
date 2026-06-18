const authHeaders = (token) => ({
  Authorization: `Bearer ${token}`,
});

const errorMessageFromResponse = async (res) => {
  const text = await res.text();
  if (!text) return `Request failed (${res.status})`;
  try {
    const data = JSON.parse(text);
    if (data?.message) return String(data.message);
    if (data?.error_code) return String(data.error_code);
  } catch {
    
  }
  return text;
};

const errorCodeFromResponse = async (res) => {
  try {
    const text = await res.clone().text();
    if (!text) return null;
    const data = JSON.parse(text);
    return data?.error_code ? String(data.error_code) : null;
  } catch {
    return null;
  }
};

const SESSION_REFRESHABLE_CODES = new Set(['EXPIRED', 'INVALID_TOKEN', 'NOT_YET_VALID']);
const SESSION_INVALID_CODES = new Set([
  'REVOKED',
]);

const AUTH_401_SKIP_REFRESH = new Set(['WRONG_PASSWORD', 'INVALID_CREDENTIALS']);

let refreshInFlight = null;

export const api = {
  login: async (login, password) => {
    const res = await fetch(`/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ login, password }),
    });
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return await res.json();
  },

  refreshToken: async (refreshToken) => {
    console.debug('[api] refreshToken request', {
      refreshTokenPresent: !!refreshToken,
      refreshTokenLength: refreshToken?.length,
    });
    const res = await fetch(`/api/v1/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
    if (!res.ok) {
      console.error('[api] refreshToken failed', { status: res.status });
      throw new Error(await errorMessageFromResponse(res));
    }
    return await res.json();
  },

  register: async (login, password) => {
    const res = await fetch(`/api/v1/accounts/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ login, password }),
    });
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  logout: async (token) => {
    const res = await fetch(`/api/v1/auth/logout`, {
      method: 'POST',
      headers: authHeaders(token),
    });
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res;
  },

  getAccount: async (id, token = null) => {
    const headers = token ? authHeaders(token) : {};
    const res = await fetch(`/api/v1/accounts/${id}`, { method: 'GET', headers });
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  updateAccount: async (tokenHandlers, accountId, login, displayName, password) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'PATCH',
      headers: {
        ...authHeaders(access_token),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        login: login !== undefined ? login : undefined,
        display_name: displayName !== undefined ? displayName : undefined,
        password: password !== undefined ? password : undefined,
      }),
    };
    const res = await request(`/api/v1/accounts/${accountId}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  getRoomsList: async (tokenHandlers, search = '', after = 0, limit = 100) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'GET',
      headers: access_token ? authHeaders(access_token) : {},
    };

    const params = new URLSearchParams();
    if (search) params.set('search', search);
    params.set('after', String(after));
    params.set('limit', String(limit));

    const res = await request(`/api/v1/rooms?${params.toString()}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  createRoom: async (tokenHandlers, name, isPublic, password, maxPlayers) => {
    const { access_token } = tokenHandlers.getToken();
    
    const options = {
      method: 'POST',
      headers: {
        ...authHeaders(access_token),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name,
        is_public: isPublic,
        password: password || null,
        max_players: maxPlayers,
      }),
    };

    const res = await request(`/api/v1/rooms`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

joinRoom: async (tokenHandlers, roomId, password = null) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: {
        ...authHeaders(access_token),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ password }),
    };
    const res = await request(`/api/v1/rooms/${roomId}/join`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  leaveRoom: async (tokenHandlers, roomId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/rooms/${roomId}/leave`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.status;
  },

  readyRoom: async (tokenHandlers, roomId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/rooms/${roomId}/ready`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.status;
  },

  kickRoomPlayer: async (tokenHandlers, roomId, playerId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/rooms/${roomId}/kick/${playerId}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.status;
  },

  updateRoom: async (tokenHandlers, roomId, name, isPublic, password, maxPlayers) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'PATCH',
      headers: {
        ...authHeaders(access_token),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        id: roomId,
        name: name || undefined,
        is_public: isPublic !== undefined ? isPublic : undefined,
        password: password !== undefined ? password : undefined,
        max_players: maxPlayers || undefined,
      }),
    };
    const res = await request(`/api/v1/rooms/${roomId}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  getGameState: async (tokenHandlers, roomId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'GET',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/rooms/${roomId}/game`, options, tokenHandlers);

    if (res.status === 204) return null;
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  getGameStateById: async (tokenHandlers, gameId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'GET',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/games/${gameId}`, options, tokenHandlers);

    if (res.status === 204) return null;
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  getGamesHistory: async (tokenHandlers, after = null, limit = 100) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'GET',
      headers: access_token ? authHeaders(access_token) : {},
    };

    const params = new URLSearchParams();
    if (after) params.set('after', String(after));
    if (limit) params.set('limit', String(limit));

    const res = await request(`/api/v1/games/history?${params.toString()}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  getRoomById: async (tokenHandlers, roomId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'GET',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/rooms/${roomId}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  drawCard: async (tokenHandlers, roomId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/rooms/${roomId}/game/play`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.status;
  },

  drawCardByGameId: async (tokenHandlers, gameId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/games/${gameId}/play`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  playCardByIndex: async (tokenHandlers, roomId, cardIndex) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/rooms/${roomId}/game/play/${cardIndex}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.status;
  },

  playCardByGameId: async (tokenHandlers, gameId, cardIndex) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: authHeaders(access_token),
    };
    const res = await request(`/api/v1/games/${gameId}/play/${cardIndex}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  getChatMessages: async (tokenHandlers, channelId, after = 0, limit = 100) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'GET',
      headers: access_token ? authHeaders(access_token) : {},
    };

    const url = `/api/v1/chat/${channelId}?after=${after}&limit=${limit}`;
    const res = await request(url, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  sendChatMessage: async (tokenHandlers, channelId, content) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'POST',
      headers: {
        ...authHeaders(access_token),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ content }),
    };

    const res = await request(`/api/v1/chat/${channelId}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  editChatMessage: async (tokenHandlers, channelId, messageId, content) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'PATCH',
      headers: {
        ...authHeaders(access_token),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ content }),
    };

    const res = await request(`/api/v1/chat/${channelId}/${messageId}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.json();
  },

  deleteChatMessage: async (tokenHandlers, channelId, messageId) => {
    const { access_token } = tokenHandlers.getToken();
    const options = {
      method: 'DELETE',
      headers: authHeaders(access_token),
    };

    const res = await request(`/api/v1/chat/${channelId}/${messageId}`, options, tokenHandlers);
    if (!res.ok) throw new Error(await errorMessageFromResponse(res));
    return res.status;
  },

  reportPlayer: async () => {
    throw new Error('Reporting is not implemented.');
  },
  mutePlayer: async () => {
    throw new Error('Mute is not implemented.');
  },

  getSavefile: async (id, token) => {
      const res = await fetch(`/api/v1/savefiles/${id}`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        }
      });
      if (!res.ok) throw new Error('Failed to fetch savefile');
      return res.json();
    },

  getRankings: async (token) => {
      const res = await fetch(`/api/v1/savefiles`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        }
      });
      if (!res.ok) throw new Error('Failed to fetch rankings');
      return res.json();
    },
};

async function request(url, options, tokenHandlers) {
  const { getToken, updateTokens, logout } = tokenHandlers;
  const { clearSession } = tokenHandlers;

  let res = await fetch(url, options);
  const statusCode = res.status;
  const errorCode = statusCode === 403 ? await errorCodeFromResponse(res) : null;
  const isRefreshable403 = statusCode === 403 && errorCode && SESSION_REFRESHABLE_CODES.has(errorCode);

  if (statusCode === 403 && !isRefreshable403) {
    if (errorCode && SESSION_INVALID_CODES.has(errorCode)) {
      (clearSession || logout)();
      throw new Error(await errorMessageFromResponse(res));
    }
    return res;
  }

  if (statusCode === 401 || isRefreshable403) {
    const code = await errorCodeFromResponse(res);
    console.warn('[api] auth failure detected, attempting refresh', {
      url,
      statusCode,
      errorCode: code,
      isRefreshable403,
    });
    if (code && AUTH_401_SKIP_REFRESH.has(code)) {
      return res;
    }

    try {
      const tokens = getToken();
      console.debug('[api] starting token refresh', {
        url,
        currentAccessTokenLength: tokens.access_token?.length,
        currentRefreshTokenLength: tokens.refresh_token?.length,
        refreshInFlight: !!refreshInFlight,
      });

      if (!tokens.refresh_token) {
        console.error('[api] no refresh token available for retry', { url, statusCode });
        throw new Error('No refresh token');
      }

      if (refreshInFlight) {
        console.debug('[api] waiting for in-flight refresh', { url });
        const newPair = await refreshInFlight;
        options.headers = { ...options.headers, Authorization: `Bearer ${newPair.access_token}` };
        res = await fetch(url, options);
      } else {
        refreshInFlight = (async () => {
          try {
            const newPair = tokenHandlers.refresh
              ? await tokenHandlers.refresh()
              : await api.refreshToken(tokens.refresh_token);
            if (!tokenHandlers.refresh) updateTokens(newPair);
            return newPair;
          } finally {
            refreshInFlight = null;
          }
        })();

        const newPair = await refreshInFlight;
        console.debug('[api] refresh succeeded, retrying request', {
          url,
          retryWithAccessTokenLength: newPair.access_token?.length,
          retryWithRefreshTokenLength: newPair.refresh_token?.length,
        });
        options.headers = { ...options.headers, Authorization: `Bearer ${newPair.access_token}` };
        res = await fetch(url, options);
      }

      if (res.status === 403) {
        const retryCode = await errorCodeFromResponse(res);
        if (retryCode && SESSION_INVALID_CODES.has(retryCode)) {
          (clearSession || logout)();
          throw new Error(await errorMessageFromResponse(res));
        }
      }
    } catch (err) {
      console.error('Token refresh failed:', err);
      try {
        (clearSession || logout)();
      } catch (e) {
        console.error('Clearing session during token refresh failure also failed:', e);
      }
      throw err;
    }
  }

  return res;
}