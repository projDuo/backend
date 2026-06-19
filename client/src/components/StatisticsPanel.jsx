export default function StatisticsPanel({
  displayName,
  stats,
  isLoading,
  error,
  isOwnProfile,
  onOpenAccountSettings,
  isMuted,
  onToggleMute,
  isMuting,
}) {
  const statsData = stats || {};

  return (
    <div className="panel-nav__body" id="playerStatistics">
      <div className="player-header">
        <h2 id="userStatistics">{displayName ? `${displayName}'s Profile` : 'Player Statistics'}</h2>
        {isOwnProfile ? (
          <button
            type="button"
            className="player-header__settings"
            onClick={onOpenAccountSettings}
          >
            Settings
          </button>
        ) : (
          <button
            type="button"
            className={`player-header__settings ${isMuted ? 'player-header__settings--muted' : ''}`}
            onClick={onToggleMute}
            disabled={isMuting}
          >
            {isMuted ? 'Unmute' : 'Mute'}
          </button>
        )}
      </div>
      {isLoading ? (
        <p style={{ color: 'var(--text-muted)' }}>Loading statistics…</p>
      ) : error ? (
        <p style={{ color: 'var(--danger)' }}>{error}</p>
      ) : (
        <div id="containerPlayerStatistics" className="stats-grid stats-grid--nav">
          <div className="stat-card">
            <span className="stat-label">Total Points</span>
            <span className="stat-value">{statsData.points || 0}</span>
          </div>
          <div className="stat-card">
            <span className="stat-label">Games Played</span>
            <span className="stat-value">{statsData.games_played || 0}</span>
          </div>
          <div className="stat-card">
            <span className="stat-label">Wins</span>
            <span className="stat-value stat-value--wins">{statsData.wins || 0}</span>
          </div>
          <div className="stat-card">
            <span className="stat-label">Losses</span>
            <span className="stat-value stat-value--losses">{statsData.loses || 0}</span>
          </div>
          <div className="stat-card">
            <span className="stat-label">Max Points</span>
            <span className="stat-value highlight">{statsData.max_points || 0}</span>
          </div>
          <div className="stat-card">
            <span className="stat-label">Cards Played</span>
            <span className="stat-value">{statsData.cards_had || 0}</span>
          </div>
        </div>
      )}
    </div>
  )
}
