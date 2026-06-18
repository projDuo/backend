import { playerDisplayName } from './GameRoom/gameUtils'
import './PlayerSlots.css'

export default function PlayerSlots({ room, myId, playerNames, onPlayerClick, onKickPlayer }) {
  if (!room) return null

  const isRoomOwner = room.owner === myId
  const max = room.max_players || 4
  const players = room.players || []
  const slots = Array.from({ length: max }, (_, i) => players[i] ?? null)

  return (
    <div className="room-players-bar">
      {slots.map((p, index) =>
        p ? (
          <div
            key={p.id}
            className={`room-player-slot room-player-slot--filled${p.id === room.owner ? ' room-player-slot--owner' : ''}`}
          >
            <div className="room-player-slot__info">
              <span className="room-player-slot__name">
                <button
                  type="button"
                  onClick={() => onPlayerClick?.(p.id)}
                  style={{
                    border: 'none',
                    background: 'transparent',
                    padding: 0,
                    margin: 0,
                    cursor: onPlayerClick ? 'pointer' : 'default',
                    color: 'inherit',
                    font: 'inherit',
                    textDecoration: onPlayerClick ? 'underline' : 'none',
                  }}
                >
                  {p.id === myId
                    ? 'You'
                    : playerDisplayName(p) || playerNames[p.id] || String(p.id).slice(0, 8)}
                </button>
              </span>
              <span className="room-player-slot__ready">
                {p.is_ready ? 'Ready' : 'Not ready'}
              </span>
            </div>
            {isRoomOwner && p.id !== myId && onKickPlayer && (
              <button
                type="button"
                aria-label={`Kick ${p.id}`}
                title="Kick player"
                className="room-player-slot__kick"
                onClick={() => onKickPlayer(p.id)}
              >
                ✕
              </button>
            )}
          </div>
        ) : (
          <div key={`empty-${index}`} className="room-player-slot room-player-slot--empty">
            <span>Empty</span>
          </div>
        )
      )}
    </div>
  )
}
