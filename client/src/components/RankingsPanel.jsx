export default function RankingsPanel({ rankings, playerNames, myId, onOpenPlayerStats }) {
  return (
    <div className="panel-nav__body" id="rankingsView">
      <h2>Global Leaderboard</h2>
      <div className="leaderboard-list leaderboard-list--nav">
        {rankings.map((rank, index) => (
          <div
            key={rank.id}
            className={`leaderboard-row ${rank.id === myId ? 'leaderboard-me' : ''}`}
          >
            <div className="leaderboard-rank">#{index + 1}</div>
            <div className="leaderboard-info">
              <span className="leaderboard-name">
                <button
                  type="button"
                  onClick={() => onOpenPlayerStats?.(rank.id)}
                  style={{
                    border: 'none',
                    background: 'transparent',
                    padding: 0,
                    margin: 0,
                    cursor: onOpenPlayerStats ? 'pointer' : 'default',
                    color: 'inherit',
                    font: 'inherit',
                    textDecoration: 'underline',
                  }}
                >
                  {playerNames[rank.id] || String(rank.id).slice(0, 8)}
                </button>
                {rank.id === myId && ' (You)'}
              </span>
              <span className="leaderboard-stats">
                Wins: {rank.wins} | Played: {rank.games_played} | Max Pts: {rank.max_points}
              </span>
            </div>
            <div className="leaderboard-points">{rank.points} Pts</div>
          </div>
        ))}
        {rankings.length === 0 && (
          <p style={{ color: 'var(--text-muted)' }}>No rankings available yet.</p>
        )}
      </div>
    </div>
  )
}
