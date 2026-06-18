import PlayerSlots from '../PlayerSlots'
import { api } from '../../services/api'
import './RoomLobby.css'

export default function RoomLobby({
  room,
  myId,
  playerNames,
  handlers,
  onLeaveRoom,
  onOpenRoomSettings,
  onOpenPlayerStats,
  onKickPlayer
}) {
  if (!room) return null

  return (
    <div className="room-lobby-container">
      <div id="nameAndIdRoom">
        <button
          type="button"
          id="leaveRoom"
          className="room-header__leave"
          title="Leave room"
          onClick={onLeaveRoom}
        >
          ✕
        </button>
        <button
          type="button"
          id="settingsRoom"
          className="room-header__settings"
          title="Room settings"
          onClick={onOpenRoomSettings}
        >
          Settings
        </button>
        <h2 id="roomName">{room.name}</h2>
        <p id="idRoom">ID: {room.id}</p>
      </div>

      <h3 id="roomLobbyTitle">
        Players ({room.players?.length || 0}/{room.max_players})
      </h3>
      
      <PlayerSlots
        room={room}
        myId={myId}
        playerNames={playerNames}
        onPlayerClick={onOpenPlayerStats}
        onKickPlayer={onKickPlayer}
      />

      <div id="buttonBlock">
        <input
          type="button"
          value="Toggle Ready"
          id="buttonReady"
          onClick={() => api.readyRoom(handlers, room.id)}
        />
      </div>
    </div>
  )
}
