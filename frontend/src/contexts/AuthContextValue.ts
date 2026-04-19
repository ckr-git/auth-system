import { createContext } from 'react';

type User = {
  id: string;
  username: string;
  display_name: string;
  subject_type: string;
};

type AuthContextType = {
  user: User | null;
  token: string | null;
  login: (token: string) => void;
  logout: () => Promise<void>;
  loading: boolean;
  clearAuth: () => void;
};

export const AuthContext = createContext<AuthContextType>(null!);
export type { AuthContextType, User };
