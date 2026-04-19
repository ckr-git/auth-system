import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (res) => res,
  (err) => {
    if (err.response?.status === 401) {
      const url = err.config?.url || '';
      const isLogoutEndpoint = url.includes('/auth/logout');
      const isAuthEndpoint = url.includes('/auth/') || url.includes('/subjects/register');
      if (!isAuthEndpoint || isLogoutEndpoint) {
        localStorage.removeItem('token');
        delete api.defaults.headers.common.Authorization;
        window.location.href = '/member/login';
      }
    }
    return Promise.reject(err);
  }
);

export default api;
