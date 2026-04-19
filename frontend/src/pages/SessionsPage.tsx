import { useEffect, useState } from 'react';
import { Table, Button, Tag, Alert, Typography, Popconfirm } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { DeleteOutlined } from '@ant-design/icons';
import api from '../api/client';
import { getApiErrorMessage } from '../api/errors';
import { useAuth } from '../contexts/useAuth';

const { Title } = Typography;

interface Session {
  session_id: string;
  device_name: string | null;
  device_ip: string | null;
  created_at: string;
  last_active_at: string;
  is_current: boolean;
}

export default function SessionsPage() {
  const { user } = useAuth();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const fetchSessions = async () => {
    setLoading(true);
    try {
      const res = await api.get('/sessions');
      setSessions(res.data.data || []);
    } catch (err: unknown) {
      setError(getApiErrorMessage(err, 'Failed to load sessions'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!user) {
      return;
    }

    void fetchSessions();
  }, [user]);

  const handleRevoke = async (sessionId: string) => {
    try {
      await api.delete(`/sessions/${sessionId}`);
      await fetchSessions();
    } catch (err: unknown) {
      setError(getApiErrorMessage(err, 'Failed to revoke session'));
    }
  };

  if (!user) return <Alert message="Not authenticated" type="warning" />;

  const columns: ColumnsType<Session> = [
    {
      title: 'Device',
      dataIndex: 'device_name',
      render: (name: string | null, record: Session) => (
        <>
          {name || 'Unknown Device'}
          {record.is_current && <Tag color="green" style={{ marginLeft: 8 }}>Current</Tag>}
        </>
      ),
    },
    { title: 'IP', dataIndex: 'device_ip', render: (ip: string | null) => ip || '-' },
    { title: 'Created', dataIndex: 'created_at', render: (d: string) => new Date(d).toLocaleString() },
    { title: 'Last Active', dataIndex: 'last_active_at', render: (d: string) => new Date(d).toLocaleString() },
    {
      title: 'Action',
      render: (_: unknown, record: Session) => (
        <Popconfirm
          title="Revoke this session?"
          onConfirm={() => handleRevoke(record.session_id)}
          disabled={record.is_current}
        >
          <Button
            danger
            size="small"
            icon={<DeleteOutlined />}
            disabled={record.is_current}
            data-testid={`session-btn-revoke-${record.session_id}`}
          >
            Revoke
          </Button>
        </Popconfirm>
      ),
    },
  ];

  return (
    <div>
      <Title level={3}>Active Sessions</Title>
      {error && <Alert message={error} type="error" style={{ marginBottom: 16 }} />}
      <Table
        dataSource={sessions}
        columns={columns}
        rowKey="session_id"
        loading={loading}
        pagination={false}
        locale={{ emptyText: 'No active sessions' }}
      />
    </div>
  );
}
