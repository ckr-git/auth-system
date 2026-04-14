import { useEffect, useState } from 'react';
import { Card, Descriptions, Button, Tag, Spin, Alert, Typography, Input } from 'antd';
import { useAuth } from '../contexts/AuthContext';
import api from '../api/client';

const { Title } = Typography;

interface CredentialStatus {
  has_password: boolean;
  has_totp: boolean;
  passkey_count: number;
}

export default function DashboardPage() {
  const { user, loading: authLoading } = useAuth();
  const [credentials, setCredentials] = useState<CredentialStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [totpSetup, setTotpSetup] = useState<{ qr_code_base64: string; secret: string } | null>(null);
  const [totpConfirmCode, setTotpConfirmCode] = useState('');
  const [totpConfirming, setTotpConfirming] = useState(false);

  useEffect(() => {
    if (!user) return;
    api.get('/credentials/status')
      .then((res) => setCredentials(res.data.data))
      .catch((err) => setError(err.response?.data?.error || 'Failed to load credentials'))
      .finally(() => setLoading(false));
  }, [user]);

  const handleTotpSetup = async () => {
    setError('');
    try {
      const res = await api.post('/credentials/totp/setup');
      setTotpSetup(res.data.data);
      setTotpConfirmCode('');
    } catch (err: unknown) {
      const msg = (err as any).response?.data?.error || 'Failed to setup TOTP';
      setError(msg);
    }
  };

  const handleTotpConfirm = async () => {
    setError('');
    setTotpConfirming(true);
    try {
      await api.post('/credentials/totp/confirm', { code: totpConfirmCode });
      setTotpSetup(null);
      setTotpConfirmCode('');
      const credRes = await api.get('/credentials/status');
      setCredentials(credRes.data.data);
    } catch (err: unknown) {
      const msg = (err as any).response?.data?.error || 'Invalid code, please try again';
      setError(msg);
    } finally {
      setTotpConfirming(false);
    }
  };

  if (authLoading || loading) return <Spin size="large" style={{ display: 'block', margin: '40px auto' }} />;
  if (!user) return <Alert message="Not authenticated" type="warning" />;

  return (
    <div style={{ maxWidth: 600, margin: '0 auto' }}>
      <Title level={3}>Dashboard</Title>
      <Card style={{ marginBottom: 16 }}>
        <Descriptions title="Profile" column={1}>
          <Descriptions.Item label="Username">{user.username}</Descriptions.Item>
          <Descriptions.Item label="Display Name">{user.display_name}</Descriptions.Item>
          <Descriptions.Item label="Type">
            <Tag color="blue">{user.subject_type}</Tag>
          </Descriptions.Item>
        </Descriptions>
      </Card>

      {error && <Alert message={error} type="error" style={{ marginBottom: 16 }} />}

      <Card title="Credentials">
        <p>Password: {credentials?.has_password ? <Tag color="green">Set</Tag> : <Tag>Not set</Tag>}</p>
        <p>
          TOTP: {credentials?.has_totp ? <Tag color="green">Enabled</Tag> : (
            <Button size="small" onClick={handleTotpSetup} data-testid="dashboard-btn-totp-setup">
              Setup TOTP
            </Button>
          )}
        </p>
        <p>Passkeys: {credentials?.passkey_count ?? 0} registered</p>
      </Card>

      {totpSetup && (
        <Card title="Scan QR Code with Authenticator App" style={{ marginTop: 16 }}>
          <img src={`data:image/png;base64,${totpSetup.qr_code_base64}`} alt="TOTP QR Code" style={{ maxWidth: 200 }} />
          <p style={{ marginTop: 8 }}>Secret: <code>{totpSetup.secret}</code></p>
          <p style={{ marginTop: 16 }}>Enter the 6-digit code from your authenticator app to confirm:</p>
          <Input
            placeholder="6-digit code"
            value={totpConfirmCode}
            onChange={(e) => setTotpConfirmCode(e.target.value)}
            maxLength={6}
            style={{ width: 200, marginRight: 8 }}
            data-testid="totp-confirm-input"
          />
          <Button
            type="primary"
            loading={totpConfirming}
            onClick={handleTotpConfirm}
            disabled={totpConfirmCode.length !== 6}
            data-testid="totp-confirm-btn"
          >
            Confirm
          </Button>
        </Card>
      )}
    </div>
  );
}
