import React from 'react'

export default function MatchesPanel({ matches = [], getCachedPlayerName, onOpenPlayerStats }) {
  
  const groups = {};
  (Array.isArray(matches) ? matches : []).forEach((m) => {
    const gid = String(m.game_id);
    if (!groups[gid]) groups[gid] = [];
    groups[gid].push(m);
  });

  const summaries = Object.values(groups).map((group) => {
    
    const participantsSet = new Set();
    group.forEach((g) => (Array.isArray(g.participants) ? g.participants.forEach((p) => participantsSet.add(p)) : null));
    const participants = Array.from(participantsSet);
    const total = participants.length || (group[0]?.participants?.length || 0);

    
    const normalized = group.map((g) => ({ ...g, displayPlacement: total && g.placement != null ? (total - g.placement + 1) : g.placement }));
    
    const winner = normalized.find((n) => n.displayPlacement === 1) || normalized[0];

    return {
      game_id: group[0].game_id,
      finished_at: group[0].finished_at,
      participants,
      winnerPlacement: winner ? winner.displayPlacement : null,
      winnerPoints: winner ? winner.points : null,
    };
  });

  return (
    <div className="panel-nav__body" id="matchesView">
      <h2>Matches</h2>
      <div className="history-list">
        {summaries.length > 0 ? (
          summaries.map((m) => {
            const date = m.finished_at ? new Date(m.finished_at).toLocaleString() : '';
            const participants = m.participants || [];
            return (
              <div className="history-item" key={String(m.game_id)}>
                <div className="history-item__meta">
                  <span className="history-item__player">{date}</span>
                </div>
                <div className="history-item__card">
                  <div className="history-card history-card--match-stats">
                    <div className="history-card__label">
                      <div className="match-stats">
                        <div className="match-stat">
                          <strong>Placement</strong>
                          <div>{m.winnerPlacement ?? '-'}</div>
                        </div>
                        <div className="match-stat">
                          <strong>Points</strong>
                          <div>{m.winnerPoints ?? '-'}</div>
                        </div>
                        <div className="match-stat">
                          <strong>Players</strong>
                          <div>{participants.length}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <div className="participants-section">
                  <div className="participants-label">Participants</div>
                  <div className="participants-container">
                    {participants.map((p) => (
                      <button
                        key={p}
                        type="button"
                        className="participant-chip"
                        onClick={() => onOpenPlayerStats?.(p)}
                        style={{
                          border: 'none',
                          background: 'transparent',
                          padding: '0.4rem 0.6rem',
                          margin: '0.1rem',
                          cursor: onOpenPlayerStats ? 'pointer' : 'default',
                          color: 'inherit',
                          font: 'inherit',
                          textDecoration: 'underline',
                        }}
                      >
                        {(getCachedPlayerName && getCachedPlayerName(p)) || String(p).slice(0, 8)}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )
          })
        ) : (
          <p style={{ color: 'var(--text-muted)' }}>No matches found.</p>
        )}
      </div>
    </div>
  )
}
