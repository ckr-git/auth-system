import { Navigate } from 'react-router-dom';
import { Spin } from 'antd';
import { useAuth } from '../contexts/AuthContext';

export default function RequireAuth({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth();

  if (loading) {
    return <Spin size="large" style={{ display: 'block', margin: '40px auto' }} />;
  }

  if (!user) {
    return <Navigate to="/member/login" replace />;
  }

  return <>{children}</>;
}
