import './Modals.css'

export default function RoomSettingsModal({
  isOpen,
  settings,
  error,
  isUpdating,
  onChange,
  onApply,
  onClose
}) {
  if (!isOpen) return null

  return (
    <div className="modal-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="modal-content">
        <h3>Room Settings</h3>
        {error && (
          <p className="modal-error" role="alert">
            {error}
          </p>
        )}
        <div className="room-settings-form">
          <div className="form-group">
            <label htmlFor="settingsName">Room Name</label>
            <input
              id="settingsName"
              type="text"
              placeholder="Room name"
              value={settings.name}
              onChange={(e) => onChange(prev => ({ ...prev, name: e.target.value }))}
            />
          </div>
          <div className="form-group form-group--horizontal">
            <label className="switch">
              <input
                type="checkbox"
                checked={settings.isPublic}
                onChange={(e) => onChange(prev => ({ ...prev, isPublic: e.target.checked }))}
              />
              Public
            </label>
          </div>
          <div className="form-group">
            <label htmlFor="settingsPassword">Password (optional)</label>
            <input
              id="settingsPassword"
              type="password"
              placeholder="Leave empty to remove password"
              value={settings.password}
              onChange={(e) => onChange(prev => ({ ...prev, password: e.target.value }))}
            />
          </div>
          <div className="form-group">
            <label htmlFor="settingsMaxPlayers">Max Players</label>
            <input
              id="settingsMaxPlayers"
              type="number"
              min={2}
              max={10}
              value={settings.maxPlayers}
              onChange={(e) => onChange(prev => ({ ...prev, maxPlayers: Number(e.target.value) }))}
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
