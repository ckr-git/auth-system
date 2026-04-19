import { useState, useEffect, useCallback, type ReactNode } from 'react';
import api from '../api/client';
import { AuthContext, type User } from './AuthContextValue';

function clearAuthorizationHeader() {
  delete api.defaults.headers.common.Authorization;
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
  const [loading, setLoading] = useState(true);

  const clearAuth = useCallback(() => {
    localStorage.removeItem('token');
    clearAuthorizationHeader();
    setToken(null);
    setUser(null);
  }, []);

  useEffect(() => {
    if (token) {
      api.get('/subjects/me')
        .then((res) => setUser(res.data.data))
        .catch(() => clearAuth())
        .finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
  }, [token, clearAuth]);

  useEffect(() => {
    if (token) {
      api.defaults.headers.common.Authorization = `Bearer ${token}`;
    } else {
      clearAuthorizationHeader();
    }
  }, [token]);

  const login = (newToken: string) => {
    localStorage.setItem('token', newToken);
    setToken(newToken);
  };

  const logout = async () => {
    try {
      await api.post('/auth/logout');
    } finally {
      clearAuth();
      window.location.href = '/member/login';
    }
  };

  return (
    <AuthContext.Provider value={{ user, token, login, logout, loading, clearAuth }}>
      {children}
    </AuthContext.Provider>
  );
}
