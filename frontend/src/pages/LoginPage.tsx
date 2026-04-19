import { useState } from 'react';
import { useNavigate, useParams, Navigate } from 'react-router-dom';
import { Form, Input, Button, Card, Typography, Alert, Tabs, Divider } from 'antd';
import { UserOutlined, LockOutlined, KeyOutlined } from '@ant-design/icons';
import api from '../api/client';
import { getApiErrorMessage } from '../api/errors';
import { useAuth } from '../contexts/useAuth';

const { Title } = Typography;

function base64urlToBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
  const pad = base64.length % 4 === 0 ? '' : '='.repeat(4 - (base64.length % 4));
  const binary = atob(base64 + pad);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes.buffer;
}

function bufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

interface PasskeyAllowCredential {
  id: string;
  type?: PublicKeyCredentialType;
  transports?: AuthenticatorTransport[];
}

const TYPE_MAP: Record<string, { label: string; apiType: string; registerType: string }> = {
  member: { label: 'Member', apiType: 'member', registerType: 'member' },
  staff: { label: 'Community Staff', apiType: 'staff', registerType: 'community_staff' },
  admin: { label: 'Platform Staff', apiType: 'admin', registerType: 'platform_staff' },
};

export default function LoginPage() {
  const navigate = useNavigate();
  const { type = 'member' } = useParams<{ type: string }>();
  const { user, login } = useAuth();
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [mfaRequired, setMfaRequired] = useState(false);
  const [mfaToken, setMfaToken] = useState('');
  const [mfaCode, setMfaCode] = useState('');
  const [passkeyUsername, setPasskeyUsername] = useState('');
  const [passkeyLoading, setPasskeyLoading] = useState(false);

  const info = TYPE_MAP[type] || TYPE_MAP.member;

  const handleLogin = async (values: { username: string; password: string }) => {
    setError('');
    setLoading(true);
    try {
      const res = await api.post(`/auth/${info.apiType}/login`, values);
      const data = res.data.data;
      if (data.requires_mfa) {
        setMfaRequired(true);
        setMfaToken(data.mfa_token);
      } else {
        login(data.token);
        navigate('/dashboard');
      }
    } catch (err: unknown) {
      setError(getApiErrorMessage(err, 'Login failed'));
    } finally {
      setLoading(false);
    }
  };

  const handleMfa = async () => {
    setError('');
    setLoading(true);
    try {
      const res = await api.post('/auth/mfa/verify', {
        mfa_token: mfaToken,
        code: mfaCode,
      });
      login(res.data.data.token);
      navigate('/dashboard');
    } catch (err: unknown) {
      setError(getApiErrorMessage(err, 'MFA verification failed'));
    } finally {
      setLoading(false);
    }
  };

  const handleRegister = async (values: { username: string; password: string; display_name: string }) => {
    setError('');
    setLoading(true);
    try {
      await api.post('/subjects/register', {
        ...values,
        subject_type: info.registerType,
      });
      await handleLogin({ username: values.username, password: values.password });
    } catch (err: unknown) {
      setError(getApiErrorMessage(err, 'Registration failed'));
      setLoading(false);
    }
  };

  const handlePasskeyLogin = async () => {
    if (!passkeyUsername.trim()) {
      setError('Please enter your username for Passkey login');
      return;
    }
    setError('');
    setPasskeyLoading(true);
    try {
      const beginRes = await api.post('/auth/passkey/begin', {
        username: passkeyUsername,
        subject_type: info.registerType,
      });
      const { challenge_id, options } = beginRes.data.data;

      const publicKey = options.publicKey;
      if (publicKey.challenge) {
        publicKey.challenge = base64urlToBuffer(publicKey.challenge);
      }
      if (publicKey.allowCredentials) {
        publicKey.allowCredentials = publicKey.allowCredentials.map((credential: PasskeyAllowCredential) => ({
          ...credential,
          id: base64urlToBuffer(credential.id),
        }));
      }

      const assertion = await navigator.credentials.get({ publicKey });
      if (!assertion) throw new Error('Passkey authentication cancelled');

      const publicKeyCredential = assertion as PublicKeyCredential;
      const response = publicKeyCredential.response as AuthenticatorAssertionResponse;
      const credential = {
        id: publicKeyCredential.id,
        rawId: bufferToBase64url(publicKeyCredential.rawId),
        type: publicKeyCredential.type,
        response: {
          authenticatorData: bufferToBase64url(response.authenticatorData),
          clientDataJSON: bufferToBase64url(response.clientDataJSON),
          signature: bufferToBase64url(response.signature),
          userHandle: response.userHandle ? bufferToBase64url(response.userHandle) : null,
        },
      };

      const completeRes = await api.post('/auth/passkey/complete', {
        challenge_id,
        credential,
      });
      login(completeRes.data.data.token);
      navigate('/dashboard');
    } catch (err: unknown) {
      setError(getApiErrorMessage(err, 'Passkey login failed'));
    } finally {
      setPasskeyLoading(false);
    }
  };

  if (user) {
    return <Navigate to="/dashboard" replace />;
  }

  if (mfaRequired) {
    return (
      <Card style={{ maxWidth: 400, margin: '40px auto' }}>
        <Title level={3}>MFA Verification</Title>
        <p>Enter the code from your authenticator app.</p>
        {error && <Alert message={error} type="error" style={{ marginBottom: 16 }} />}
        <Input
          placeholder="6-digit code"
          value={mfaCode}
          onChange={(e) => setMfaCode(e.target.value)}
          maxLength={6}
          style={{ marginBottom: 16 }}
          data-testid="mfa-input-code"
        />
        <Button type="primary" block loading={loading} onClick={handleMfa} data-testid="mfa-btn-verify">
          Verify
        </Button>
      </Card>
    );
  }

  return (
    <Card style={{ maxWidth: 420, margin: '40px auto' }}>
      <Title level={3}>{info.label} Portal</Title>
      {error && <Alert title={error} type="error" style={{ marginBottom: 16 }} />}
      <Tabs
        defaultActiveKey="login"
        items={[
          {
            key: 'login',
            label: 'Login',
            children: (
              <Form onFinish={handleLogin} layout="vertical">
                <Form.Item name="username" rules={[{ required: true, message: 'Username is required' }]}>
                  <Input prefix={<UserOutlined />} placeholder="Username" data-testid="login-input-username" />
                </Form.Item>
                <Form.Item name="password" rules={[{ required: true, message: 'Password is required' }]}>
                  <Input.Password prefix={<LockOutlined />} placeholder="Password" data-testid="login-input-password" />
                </Form.Item>
                <Button type="primary" htmlType="submit" block loading={loading} data-testid="login-btn-submit">
                  Login
                </Button>
                <Divider plain>or</Divider>
                <Input
                  prefix={<UserOutlined />}
                  placeholder="Username for Passkey"
                  value={passkeyUsername}
                  onChange={(e) => setPasskeyUsername(e.target.value)}
                  style={{ marginBottom: 8 }}
                  data-testid="passkey-input-username"
                />
                <Button
                  icon={<KeyOutlined />}
                  block
                  loading={passkeyLoading}
                  onClick={handlePasskeyLogin}
                  data-testid="passkey-btn-login"
                >
                  Login with Passkey
                </Button>
              </Form>
            ),
          },
          {
            key: 'register',
            label: 'Register',
            children: (
              <Form onFinish={handleRegister} layout="vertical">
                <Form.Item name="username" rules={[{ required: true, message: 'Username is required' }]}>
                  <Input prefix={<UserOutlined />} placeholder="Username" data-testid="register-input-username" />
                </Form.Item>
                <Form.Item name="display_name" rules={[{ required: true, message: 'Display name is required' }]}>
                  <Input placeholder="Display Name" data-testid="register-input-displayname" />
                </Form.Item>
                <Form.Item name="password" rules={[{ required: true, min: 6, message: 'Min 6 characters' }]}>
                  <Input.Password prefix={<LockOutlined />} placeholder="Password" data-testid="register-input-password" />
                </Form.Item>
                <Button type="primary" htmlType="submit" block loading={loading} data-testid="register-btn-submit">
                  Register
                </Button>
              </Form>
            ),
          },
        ]}
      />
    </Card>
  );
}
