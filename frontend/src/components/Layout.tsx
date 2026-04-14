import { Layout, Menu, Button, Space, Tag } from 'antd';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import {
  UserOutlined, TeamOutlined, CrownOutlined,
  DesktopOutlined, DashboardOutlined, LogoutOutlined,
} from '@ant-design/icons';
import { useAuth } from '../contexts/AuthContext';

const { Header, Content, Sider } = Layout;

export default function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const { user, logout } = useAuth();

  const menuItems = user
    ? [
        { key: '/dashboard', icon: <DashboardOutlined />, label: 'Dashboard' },
        { key: '/sessions', icon: <DesktopOutlined />, label: 'Sessions' },
      ]
    : [
        { key: '/member/login', icon: <UserOutlined />, label: 'Member' },
        { key: '/staff/login', icon: <TeamOutlined />, label: 'Staff' },
        { key: '/admin/login', icon: <CrownOutlined />, label: 'Admin' },
      ];

  const handleLogout = () => {
    logout();
    navigate('/member/login');
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider breakpoint="lg" collapsedWidth="0">
        <div style={{ height: 32, margin: 16, color: '#fff', fontWeight: 'bold', textAlign: 'center' }}>
          Auth System
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[location.pathname]}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
        />
      </Sider>
      <Layout>
        <Header style={{ padding: '0 24px', background: '#fff', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <h2 style={{ margin: 0 }}>Multi-Subject Auth System</h2>
          {user && (
            <Space>
              <span>{user.display_name}</span>
              <Tag color="blue">{user.subject_type}</Tag>
              <Button icon={<LogoutOutlined />} onClick={handleLogout} data-testid="layout-btn-logout">
                Logout
              </Button>
            </Space>
          )}
        </Header>
        <Content style={{ margin: '24px 16px', padding: 24, background: '#fff', borderRadius: 8 }}>
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
}
