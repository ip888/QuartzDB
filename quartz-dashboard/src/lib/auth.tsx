'use client';

import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import api from './api';

interface AuthContextType {
  apiKey: string;
  setApiKey: (key: string) => void;
  isAuthenticated: boolean;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [apiKey, setApiKeyState] = useState<string>('');

  useEffect(() => {
    // Load from localStorage on mount
    const saved = localStorage.getItem('quartz_api_key');
    if (saved) {
      setApiKeyState(saved);
      api.setApiKey(saved);
    }
  }, []);

  const setApiKey = (key: string) => {
    setApiKeyState(key);
    api.setApiKey(key);
    if (key) {
      localStorage.setItem('quartz_api_key', key);
    } else {
      localStorage.removeItem('quartz_api_key');
    }
  };

  const logout = () => {
    setApiKey('');
  };

  return (
    <AuthContext.Provider value={{ apiKey, setApiKey, isAuthenticated: !!apiKey, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}
