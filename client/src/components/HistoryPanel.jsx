const enumNameFromValue = (value) => {
  if (typeof value === 'string') return value
  if (typeof value === 'object' && value) {
    const keys = Object.keys(value)
    return keys.length ? keys[0] : null
  }
  return null
}

export default function HistoryPanel({ gameState, playerNames, onOpenPlayerStats }) {
  if (!gameState?.history || !gameState.history.length) {
    return (
      <div className="panel-nav__body" id="historyView">
        <h2>Game History</h2>
        <p style={{ color: 'var(--text-muted)' }}>No history available yet.</p>
      </div>
    )
  }

  return (
    <div className="panel-nav__body" id="historyView">
      <h2>Game History</h2>
      <div className="history-list">
        {gameState.history.map((turn, index) => {
          const playerName = turn.player
            ? playerNames[String(turn.player)] || String(turn.player).slice(0, 8)
            : 'Start'
          const element = enumNameFromValue(turn.card?.element)
          const effect =
            turn.card?.effect && typeof turn.card.effect === 'object'
              ? Object.entries(turn.card.effect)
                  .map(([key, value]) => (value != null ? `${key} ${value}` : key))
                  .join(' ')
              : String(turn.card?.effect || '')
          return (
            <div className="history-item" key={`${index}-${playerName}`}>
              <div className="history-item__meta">
                <span className="history-item__turn">#{index + 1}</span>
                <span className="history-item__player">
                  {turn.player ? (
                    <button
                      type="button"
                      onClick={() => onOpenPlayerStats?.(turn.player)}
                      style={{
                        border: 'none',
                        background: 'transparent',
                        padding: 0,
                        margin: 0,
                        cursor: onOpenPlayerStats ? 'pointer' : 'default',
                        color: 'inherit',
                        font: 'inherit',
                        textDecoration: onOpenPlayerStats ? 'underline' : 'none',
                      }}
                    >
                      {playerName}
                    </button>
                  ) : (
                    playerName
                  )}
                </span>
              </div>
              <div className="history-item__card">
                <div className="history-card">
                  {element && (
                    <img
                      className="history-card__icon"
                      src={`/textures/elements/${element.toLowerCase()}.svg`}
                      alt=""
                      aria-hidden="true"
                    />
                  )}
                  <div className="history-card__label">
                    <span>Element: {element || 'Unknown'}</span>
                    {effect && <span>Effect: {effect}</span>}
                  </div>
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
