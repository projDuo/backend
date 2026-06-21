import React, { createContext, useState, useContext, useMemo, useCallback, useRef } from 'react';
import { api } from '../services/api';

export const AuthContext = createContext();

const normalizeStoredToken = (value) => {
  if (typeof value !== 'string') return null;
  if (value === 'undefined' || value === 'null' || value.trim() === '') return null;
  return value;
};

export const AuthProvider = ({ children }) => {
  const [token, setToken] = useState(normalizeStoredToken(localStorage.getItem('access_token')));
  const [refreshToken, setRefreshToken] = useState(normalizeStoredToken(localStorage.getItem('refresh_token')));
  const refreshInFlightRef = useRef(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const updateTokens = useCallback((pair) => {
    if (!pair || !pair.access_token) return;
    console.debug('[AuthContext] updateTokens', {
      accessTokenLength: pair.access_token?.length,
      refreshTokenLength: pair.refresh_token?.length,
    });

    localStorage.setItem('access_token', pair.access_token);
    setToken(pair.access_token);

    const nextRefreshToken = normalizeStoredToken(pair.refresh_token);
    if (nextRefreshToken) {
      localStorage.setItem('refresh_token', nextRefreshToken);
      setRefreshToken(nextRefreshToken);
    } else if (pair.refresh_token === undefined) {
    } else {
      localStorage.removeItem('refresh_token');
      setRefreshToken(null);
    }
  }, []);

  const register = async (loginName, password) => {
    setLoading(true);
    setError(null);
    try {
      await api.register(loginName, password);
      const tokenPair = await api.login(loginName, password);
      updateTokens(tokenPair);
      return tokenPair;
    } catch (e) {
      setError(e.message || 'Registration failed');
      throw e;
    } finally {
      setLoading(false);
    }
  };

  const login = async (loginName, password) => {
    setLoading(true);
    setError(null);
    try {
      const tokenPair = await api.login(loginName, password);
      updateTokens(tokenPair);
      return tokenPair;
    } catch (e) {
      setError(e.message || 'Login failed');
      throw e;
    } finally {
      setLoading(false);
    }
  };

  const logout = async () => {
    setLoading(true);
    try {
      if (token) {
        await api.logout(token);
      }
    } catch (e) {
      console.error('Logout error:', e);
    } finally {
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      setToken(null);
      setRefreshToken(null);
      setLoading(false);
    }
  };

  const clearSession = useCallback(() => {
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    setToken(null);
    setRefreshToken(null);
  }, []);

  const refresh = useCallback(async () => {
    if (refreshInFlightRef.current) {
      console.debug('[AuthContext] refresh already in flight, reusing promise');
      return refreshInFlightRef.current;
    }

    const refreshPromise = (async () => {
      setLoading(true);
      try {
        const rt = normalizeStoredToken(localStorage.getItem('refresh_token')) || refreshToken;
        console.debug('[AuthContext] refresh start', {
          refreshTokenInState: !!refreshToken,
          refreshTokenInStorage: !!normalizeStoredToken(localStorage.getItem('refresh_token')),
        });
        if (!rt) throw new Error('No refresh token available');
        const pair = await api.refreshToken(rt);
        console.debug('[AuthContext] refresh success', {
          accessTokenLength: pair.access_token?.length,
          refreshTokenLength: pair.refresh_token?.length,
        });
        updateTokens(pair);
        return pair;
      } catch (e) {
        console.error('AuthContext refresh failed:', e);
        throw e;
      } finally {
        setLoading(false);
      }
    })();

    refreshInFlightRef.current = refreshPromise;

    try {
      return await refreshPromise;
    } finally {
      if (refreshInFlightRef.current === refreshPromise) {
        refreshInFlightRef.current = null;
      }
    }
  }, [refreshToken, updateTokens]);

  const value = useMemo(() => ({
    token,      
    refreshToken, 
    updateTokens, 
    loading,
    error,
    clearSession,
    refresh,
    clearError: () => setError(null),
    register,
    login,
    logout,
    isAuthenticated: !!token,
  }), [token, refreshToken, loading, error]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
};