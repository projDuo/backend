import './GameOverPanel.css'

export default function GameOverPanel({
  gameResults,
  myId,
  playerNames,
  onOpenPlayerStats,
  onBackToLobby
}) {
  if (!gameResults) return null

  return (
    <div id="gameOverBlock" className="game-over-container">
      <h2 className="game-over-title" style={{ fontSize: '2rem', textAlign: 'center' }}>
        Game Over!
      </h2>
      <div className="leaderboard-list">
        {Array.isArray(gameResults) &&
          gameResults.map((loser, index) => (
            <div
              key={loser.id}
              className={`leaderboard-row ${loser.id === myId ? 'leaderboard-me' : ''}`}
            >
              <div className="leaderboard-rank">#{index + 1}</div>
              <div className="leaderboard-info">
                <span className="leaderboard-name">
                  <button
                    type="button"
                    onClick={() => onOpenPlayerStats(loser.id)}
                    style={{
                      border: 'none',
                      background: 'transparent',
                      padding: 0,
                      margin: 0,
                      cursor: 'pointer',
                      color: 'inherit',
                      font: 'inherit',
                      textDecoration: 'underline',
                    }}
                  >
                    {playerNames[String(loser.id)] || loser.id.slice(0, 8)}
                  </button>
                  {loser.id === myId && ' (You)'}
                </span>
                <span className="leaderboard-stats">
                  Cards Played: {loser.cards_had}
                </span>
              </div>
              <div className="leaderboard-points">+{loser.points} Pts</div>
            </div>
          ))}
      </div>
      <button
        type="button"
        className="game-over-back leaderboard-row leaderboard-back-btn"
        onClick={onBackToLobby}
      >
        Back to Lobby
      </button>
    </div>
  )
}
