import React, { useEffect, useState } from 'react';
import { useAuth } from '../context/AuthContext';
import './AuthPage.css';

export default function AuthPage() {
  const { login, register, loading, error, clearError } = useAuth();
  const [isRegister, setIsRegister] = useState(false);
  const [popup, setPopup] = useState({ open: false, message: '' });
  const [formData, setFormData] = useState({
    login: '',
    password: '',
  });

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData(prev => ({ ...prev, [name]: value }));
  };

  const handleSubmit = async (e) => {
    e.preventDefault();

    try {
      if (isRegister) {
        await register(formData.login, formData.password);
      } else {
        await login(formData.login, formData.password);
      }
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    if (!error) return;
    setPopup({ open: true, message: error });
    const t = setTimeout(() => setPopup((p) => ({ ...p, open: false })), 5200);
    return () => clearTimeout(t);
  }, [error]);

  const hint = (() => {
    const msg = String(popup.message || '').toLowerCase();
    if (!msg) return '';
    if (msg.includes('invalid') || msg.includes('forbidden') || msg.includes('unauthorized')) {
      return 'Check your login/password and try again.';
    }
    if (msg.includes('not found')) {
      return 'Account not found. Switch to Register.';
    }
    if (msg.includes('password') && (msg.includes('long') || msg.includes('short'))) {
      return 'Check password requirements.';
    }
    return 'Please try again.';
  })();

  const dismissPopup = () => {
    setPopup({ open: false, message: '' });
    if (clearError) clearError();
  };

  return (
    <div className="auth-container">
      {popup.open && (
        <div className="auth-popup-overlay" onClick={dismissPopup} role="alert" aria-live="assertive">
          <div className="auth-popup" onClick={(e) => e.stopPropagation()}>
            <div className="auth-popup-title">Authentication error</div>
            <div className="auth-popup-message">{popup.message}</div>
            {hint ? <div className="auth-popup-hint">{hint}</div> : null}
            <div className="auth-popup-actions">
              <button type="button" onClick={dismissPopup}>
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="auth-card">
        <h2>{isRegister ? 'Create Account' : 'Sign In'}</h2>

        <div className="auth-toggle">
          <span>{isRegister ? 'Already have an account? ' : "Don't have an account? "}</span>
          <button
            type="button"
            className="toggle-btn"
            onClick={() => {
              setIsRegister((v) => !v);
              if (clearError) clearError();
              setPopup({ open: false, message: '' });
            }}
            disabled={loading}
          >
            {isRegister ? 'Sign In' : 'Register'}
          </button>
        </div>

        <form onSubmit={handleSubmit} className="auth-form">
          <input
            type="text"
            name="login"
            placeholder="Login"
            value={formData.login}
            onChange={handleChange}
            required
            disabled={loading}
          />
          <input
            type="password"
            name="password"
            placeholder="Password"
            value={formData.password}
            onChange={handleChange}
            required
            disabled={loading}
          />

          <button type="submit" disabled={loading}>
            {loading ? 'Loading...' : isRegister ? 'Register' : 'Login'}
          </button>
        </form>
      </div>
    </div>
  );
}
