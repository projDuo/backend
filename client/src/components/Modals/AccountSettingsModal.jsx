import './Modals.css'

export default function AccountSettingsModal({
  isOpen,
  settings,
  error,
  isUpdating,
  isPasswordUnlocked,
  onChange,
  onUnlockPassword,
  onApply,
  onClose
}) {
  if (!isOpen) return null

  return (
    <div className="modal-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="modal-content">
        <h3>Account Settings</h3>
        {error && (
          <p className="modal-error" role="alert">
            {error}
          </p>
        )}
        <div className="room-settings-form">
          <div className="form-group">
            <label htmlFor="accountLogin">Login</label>
            <input
              id="accountLogin"
              type="text"
              placeholder="Login"
              value={settings.login}
              onChange={(e) => onChange(prev => ({ ...prev, login: e.target.value }))}
            />
          </div>
          <div className="form-group">
            <label htmlFor="accountDisplayName">Display Name</label>
            <input
              id="accountDisplayName"
              type="text"
              placeholder="Display name"
              value={settings.displayName}
              onChange={(e) => onChange(prev => ({ ...prev, displayName: e.target.value }))}
            />
          </div>
          <div className="form-group">
            <label htmlFor="accountPassword">Password</label>
            <input
              id="accountPassword"
              type="password"
              placeholder={isPasswordUnlocked ? "Leave empty to keep current password" : "Double click to change password"}
              value={settings.password}
              onChange={(e) => onChange(prev => ({ ...prev, password: e.target.value }))}
              onDoubleClick={onUnlockPassword}
              readOnly={!isPasswordUnlocked}
              style={{
                opacity: isPasswordUnlocked ? 1 : 0.6,
                cursor: isPasswordUnlocked ? 'text' : 'pointer',
              }}
            />
          </div>
        </div>
        <div className="modal-buttons">
          <button
            type="button"
            className="join"
            onClick={onApply}
            disabled={isUpdating}
          >
            {isUpdating ? 'Applying...' : 'Apply'}
          </button>
          <button
            type="button"
            onClick={onClose}
            disabled={isUpdating}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}
