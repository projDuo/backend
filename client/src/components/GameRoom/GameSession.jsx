import React, { useMemo, useState, useEffect, useRef } from 'react';
import { api } from '../../services/api';
import {
  playerIdAtTurn,
  seatsAroundTable,
  elementColor,
  playerNameFromSources,
} from './gameUtils';
import './GameSession.css';

const enumName = (value) => {
  if (value == null) return null;
  if (typeof value === 'string') return value;
  if (typeof value === 'object') {
    const keys = Object.keys(value);
    return keys.length ? keys[0] : null;
  }
  return null;
};

const formatEffect = (effect) => {
  if (effect == null) return '';
  if (typeof effect === 'string') return effect;
  if (typeof effect === 'object') {
    if (effect.Atk != null) return `Atk ${effect.Atk}`;
    if (effect.Add != null) return `Add ${effect.Add}`;
    if (effect.Flow != null) return 'Flow';
    if (effect.Stun != null) return 'Stun';
  }
  return String(effect);
};

const elementIndex = (elem) => {
  const map = { Water: 0, Fire: 1, Wood: 2, Earth: 3, Air: 4, Energy: 5 };
  return map[elem] ?? -1;
};

const elementCoefficient = (myElement, opponentElement) => {
  if (myElement === 'Energy' || opponentElement === 'Energy') return 1.0;

  const myPos = elementIndex(myElement) + 1;
  const opponentPos = elementIndex(opponentElement) + 1;
  const half = (5 - 1) / 2;

  let distance =
    myPos <= opponentPos ? opponentPos - myPos : 5 + opponentPos - myPos;

  if (distance === 0) return 1.0;
  if (distance > half) distance += 1;
  else distance -= 1;

  return 0.5 + (5 - distance) / 4;
};

const canPlayCard = (myCard, currentCard) => {
  if (!myCard || !currentCard) return false;

  const myElement = enumName(myCard.element);
  const myEffect = myCard.effect;
  const currentElement = enumName(currentCard.element);
  const currentEffect = currentCard.effect;

  const coef = elementCoefficient(myElement, currentElement);

  let opponentPower = 1;
  if (typeof currentEffect === 'object' && currentEffect.Atk != null) {
    opponentPower = currentEffect.Atk;
  }

  if (typeof myEffect === 'object' && myEffect.Atk != null) {
    const myPower = myEffect.Atk;
    const effectivePower = Math.round(myPower * coef);
    return effectivePower >= opponentPower;
  }

  return coef >= 1.0;
};

function useTurnCountdown(turnEnforcedAt) {
  const [secondsLeft, setSecondsLeft] = useState(null);

  useEffect(() => {
    if (turnEnforcedAt == null) {
      setSecondsLeft(null);
      return undefined;
    }

    const tick = () => {
      const ms = Number(turnEnforcedAt) - Date.now();
      setSecondsLeft(Math.max(0, Math.ceil(ms / 1000)));
    };

    tick();
    const id = setInterval(tick, 250);
    return () => clearInterval(id);
  }, [turnEnforcedAt]);

  return secondsLeft;
}

function CardFace({ card, className = '', children, ...props }) {
  const element = enumName(card?.element);
  const bg = elementColor(element);
  const effect = card?.effect;
  const effectDisplay = formatEffect(effect);

  const handleKeyDown = (event) => {
    if (props.onClick && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      props.onClick(event);
    }
  };

  return (
    <div
      className={`card-face ${className}`}
      style={{ backgroundColor: bg }}
      title={element ? `${element} — ${effectDisplay}` : undefined}
      onKeyDown={handleKeyDown}
      {...props}
    >
      {effectDisplay && (
        <>
          <span className="card-face__value card-face__value--top-left">{effectDisplay}</span>
          <span className="card-face__value card-face__value--bottom-right">{effectDisplay}</span>
        </>
      )}
      {element && (
        <img
          className="card-face__icon"
          src={`/textures/elements/${element.toLowerCase()}.svg`}
          alt=""
          aria-hidden
        />
      )}
      {children}
    </div>
  );
}

function CardBack({ className = '' }) {
  return (
    <div className={`card-back ${className}`} aria-hidden>
      <span className="card-back__mark">?</span>
    </div>
  );
}

function HandCard({ card, index, isPlayable, myTurn, busy, onPlay }) {
  const [raised, setRaised] = useState(false);
  const leaveTimerRef = useRef(null);

  const onMouseEnter = () => {
    if (leaveTimerRef.current) {
      clearTimeout(leaveTimerRef.current);
      leaveTimerRef.current = null;
    }
    setRaised(true);
  };

  const onMouseLeave = () => {
    leaveTimerRef.current = setTimeout(() => setRaised(false), 100);
  };

  useEffect(
    () => () => {
      if (leaveTimerRef.current) clearTimeout(leaveTimerRef.current);
    },
    []
  );

  return (
    <button
      type="button"
      className={`hand-card ${
        isPlayable ? 'hand-card--playable' : 'hand-card--locked'
      } ${raised ? 'hand-card--raised' : ''}`}
      style={{ zIndex: raised ? 100 : index }}
      disabled={!myTurn || busy || !isPlayable}
      title={
        isPlayable
          ? `${enumName(card.element)} — ${formatEffect(card.effect)}`
          : 'Cannot play'
      }
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      onClick={onPlay}
    >
      <CardFace card={card} className="hand-card__face" />
    </button>
  );
}

function OpponentHand({ count, orientation }) {
  const n = Math.max(0, Number(count) || 0);
  if (n === 0) return null;

  const layout = (orientation === 'left' || orientation === 'right') ? 'vertical' : 'horizontal';

  return (
    <div
      className={`opponent-hand opponent-hand--${orientation} opponent-hand--${layout}`}
      aria-label={`${n} cards`}
    >
      {Array.from({ length: n }, (_, i) => (
        <CardBack key={i} />
      ))}
    </div>
  );
}

function SeatBlock({
  seat,
  orientation,
  isCurrentTurn,
  secondsLeft,
  playerLabel,
  playerId,
  onOpenPlayerStats,
  children,
}) {
  if (!seat) return null;

  return (
    <div
      className={`game-seat game-seat--${orientation} ${
        isCurrentTurn ? 'game-seat--active' : ''
      }`}
    >
      <div className="game-seat__meta">
        <span className="game-seat__name">
          {playerId && onOpenPlayerStats ? (
            <button
              type="button"
              onClick={() => onOpenPlayerStats(playerId)}
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
              {playerLabel}
            </button>
          ) : (
            playerLabel
          )}
        </span>
        {isCurrentTurn && secondsLeft != null && (
          <span
            className={`game-seat__timer ${
              secondsLeft <= 5 ? 'game-seat__timer--urgent' : ''
            }`}
            aria-live="polite"
          >
            0:{String(secondsLeft).padStart(2, '0')}
          </span>
        )}
      </div>
      <div className="game-seat__cards">{children}</div>
    </div>
  );
}

export default function GameSession({
  handlers,
  gameId,
  myId,
  game,
  myCards = [],
  playerNames = {},
  roomPlayers = [],
  onPlayResult,
  onForceLobby,
  onOpenChat,
  onOpenHistory,
  onLeaveRoom,
  onOpenPlayerStats,
}) {
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);
  const [selectedOpponentIndex, setSelectedOpponentIndex] = useState(0);
  const hand = Array.isArray(myCards) ? myCards : [];

  useEffect(() => {
    setError(null);
  }, [game?.turn, game?.turn_enforced_at]);

  const seats = useMemo(() => seatsAroundTable(game, myId), [game, myId]);
  const opponents = useMemo(
    () => [seats.top, seats.left, seats.right].filter(Boolean),
    [seats]
  );
  const currentTurnPlayerId = useMemo(() => playerIdAtTurn(game), [game]);
  const secondsLeft = useTurnCountdown(game?.turn_enforced_at);

  const selectedOpponent = opponents[selectedOpponentIndex] || null;
  const selectedOpponentCount = selectedOpponent?.cards || 0;

  useEffect(() => {
    if (opponents.length === 0) {
      setSelectedOpponentIndex(0);
      return;
    }

    const opponentIndex = opponents.findIndex(
      (player) => String(player.id) === String(currentTurnPlayerId)
    );

    if (opponentIndex >= 0) {
      setSelectedOpponentIndex(opponentIndex);
      return;
    }

    setSelectedOpponentIndex((prev) => Math.min(prev, opponents.length - 1));
  }, [currentTurnPlayerId, opponents]);

  const showPreviousOpponent = () => {
    if (!opponents.length) return;
    setSelectedOpponentIndex((prev) =>
      prev === 0 ? opponents.length - 1 : prev - 1
    );
  };

  const showNextOpponent = () => {
    if (!opponents.length) return;
    setSelectedOpponentIndex((prev) => (prev + 1) % opponents.length);
  };

  const myTurn = useMemo(() => {
    if (!game || !myId || currentTurnPlayerId == null) return false;
    return String(currentTurnPlayerId) === String(myId);
  }, [game, myId, currentTurnPlayerId]);

  const playableCards = useMemo(() => {
    if (!game || hand.length === 0) return new Set();
    const playable = new Set();
    hand.forEach((card, idx) => {
      if (canPlayCard(card, game.card)) playable.add(idx);
    });
    return playable;
  }, [game, hand]);

  const hasPlayableCards = playableCards.size > 0;
  const shouldShowDrawHint = myTurn && hand.length > 0 && !hasPlayableCards;

  const playedCardKey = useMemo(() => {
    if (!game?.card) return 'none';
    const el = enumName(game.card.element);
    const eff = JSON.stringify(game.card.effect);
    return `${game.turn}-${el}-${eff}`;
  }, [game?.card, game?.turn]);

  const labelFor = (player) => {
    if (!player) return '';
    if (String(player.id) === String(myId)) {
      return 'You';
    }
    return playerNameFromSources(player.id, playerNames, roomPlayers);
  };

  const isTurn = (player) =>
    player && String(player.id) === String(currentTurnPlayerId);

  const playCard = async (idx) => {
    setBusy(true);
    setError(null);
    try {
      if (!gameId) throw new Error('Game session is not ready yet');
      const result = await api.playCardByGameId(handlers, gameId, idx);
      onPlayResult?.(result);
    } catch (e) {
      setError(e.message);
    } finally {
      setBusy(false);
    }
  };

  const drawCard = async () => {
    setBusy(true);
    setError(null);
    try {
      if (!gameId) throw new Error('Game session is not ready yet');
      const result = await api.drawCardByGameId(handlers, gameId);
      onPlayResult?.(result);
    } catch (e) {
      setError(e.message);
    } finally {
      setBusy(false);
    }
  };

  if (!game && !error) {
    return <div className="game-loading">Waiting for game state…</div>;
  }

  return (
    <div className="game-session">
      {error && (
        <div className="game-error-banner">
          <p>{error}</p>
          <button type="button" className="draw-btn" onClick={onForceLobby}>
            Return to Lobby
          </button>
        </div>
      )}

      {game && (
        <div className="game-table">
          <SeatBlock
            seat={selectedOpponent}
            orientation="top"
            isCurrentTurn={isTurn(selectedOpponent)}
            secondsLeft={secondsLeft}
            playerId={selectedOpponent?.id}
            onOpenPlayerStats={onOpenPlayerStats}
            playerLabel={labelFor(selectedOpponent)}
          >
            <div className="opponent-panel">
              {opponents.length > 1 && (
                <button
                  type="button"
                  className="opponent-nav-btn"
                  onClick={showPreviousOpponent}
                  aria-label="Previous opponent"
                >
                  ‹
                </button>
              )}
              <div className="opponent-panel__content">
                <OpponentHand
                  count={selectedOpponentCount}
                  orientation="horizontal"
                />
                {opponents.length > 1 && (
                  <span className="opponent-panel__pager">
                    {selectedOpponentIndex + 1} / {opponents.length}
                  </span>
                )}
              </div>
              {opponents.length > 1 && (
                <button
                  type="button"
                  className="opponent-nav-btn"
                  onClick={showNextOpponent}
                  aria-label="Next opponent"
                >
                  ›
                </button>
              )}
            </div>
          </SeatBlock>

          <div className="game-center">
           <div className="game-center__cards">
               {game.card && (
                 <CardFace
                   key={playedCardKey}
                   card={game.card}
                   className={`game-center__played game-center__played--deal ${
                     enumName(game.card.element) === 'Fire' || enumName(game.card.element) === 'Energy'
                       ? 'game-center__played--red-glow'
                       : ''
                   }`}
                   onClick={onOpenHistory}
                   role={onOpenHistory ? 'button' : undefined}
                   tabIndex={onOpenHistory ? 0 : undefined}
                 />
               )}
               <button
                 type="button"
                 className="game-center__deck"
                 disabled={!myTurn || busy}
                 onClick={drawCard}
                 title={myTurn ? 'Draw a card' : 'Wait for your turn'}
                 aria-label="Draw pile"
               >
                 <span className="game-center__deck-mark">?</span>
               </button>
             </div>
            {myTurn && (
              <p className="game-center__hint">
                {shouldShowDrawHint ? 'No playable cards — draw from the deck' : 'Your turn'}
              </p>
            )}
          </div>

          <SeatBlock
            seat={seats.bottom}
            orientation="bottom"
            isCurrentTurn={isTurn(seats.bottom)}
            secondsLeft={secondsLeft}
            playerId={seats.bottom?.id}
            onOpenPlayerStats={onOpenPlayerStats}
            playerLabel={labelFor(seats.bottom)}
          >
            <div className="player-hand">
              {hand.length > 0 ? (
                hand.map((card, idx) => (
                  <HandCard
                    key={idx}
                    card={card}
                    index={idx}
                    isPlayable={playableCards.has(idx)}
                    myTurn={myTurn}
                    busy={busy}
                    onPlay={() => playCard(idx)}
                  />
                ))
              ) : (
                <p className="empty-hand">No cards — draw from the deck</p>
              )}
            </div>
          </SeatBlock>
        </div>
      )}
    </div>
  );
}
