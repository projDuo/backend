export class WebSocketManager {
  constructor(token) {
    this.token = token;
    this.ws = null;
    this.listeners = {};
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 3000;

    this.isReady = false;
    this.eventsBuffer = [];
    this.paused = false;
    this.pauseBuffer = [];

    this.intentionalClose = false;
    this.heartbeatTimer = null;
    this.heartbeatIntervalSec = null;
  }

  parsePayload(data) {
    if (typeof data === 'string') {
      return [data, null];
    }

    if (!data || typeof data !== 'object' || Array.isArray(data)) {
      return [null, null];
    }

    if (typeof data.event === 'string') {
      return [data.event, data.data ?? null];
    }

    if (data.error_code && data.message) {
      return ['Error', data];
    }

    const entries = Object.entries(data);
    if (entries.length === 1) {
      const [outerKey, outerValue] = entries[0];
      if (outerValue && typeof outerValue === 'object' && !Array.isArray(outerValue)) {
        if (typeof outerValue.event === 'string') {
          return [outerValue.event, outerValue.data ?? null];
        }
      }
      return [outerKey, outerValue];
    }

    return [null, null];
  }

  connect() {
    return new Promise((resolve, reject) => {
      const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = `${wsProto}//${window.location.host}/api/v1/gateway`;

      this.ws = new WebSocket(wsUrl);
      let settled = false;

      this.ws.onopen = () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
        this.isReady = false;
        this.eventsBuffer = [];
        this.stopHeartbeat();
        this.intentionalClose = false;

        this.ws.send(JSON.stringify({
          event: 'Identify',
          data: { token: this.token },
        }));

        if (!settled) {
          settled = true;
          resolve();
        }
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.processIncomingMessage(data);
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      this.ws.onerror = (error) => {
        if (this.intentionalClose) {
          return;
        }
        console.error('WebSocket error:', error);
        if (!settled) {
          settled = true;
          reject(error);
        }
      };

      this.ws.onclose = () => {
        console.log('WebSocket closed');
        this.isReady = false;

        if (!this.intentionalClose) {
          this.attemptReconnect();
        }
      };
    });
  }

  attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`Attempting reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      setTimeout(() => this.connect().catch((e) => console.error(e)), this.reconnectDelay);
    }
  }

  processIncomingMessage(data) {
    const [eventType, payloadData] = this.parsePayload(data);
    if (!eventType) {
      console.warn('Unknown WebSocket payload format:', data);
      return;
    }

    const clientEvent = eventType === 'Authorized' ? 'Ready' : eventType;

    if (eventType === 'Authorized') {
      this.isReady = true;
      this.startHeartbeat(payloadData?.heartbeat_interval);
      this.broadcast('Ready', payloadData);

      while (this.eventsBuffer.length > 0) {
        const buffered = this.eventsBuffer.shift();
        this.processIncomingMessage(buffered);
      }
      return;
    }

    if (eventType === 'Error') {
      const code = payloadData?.error_code || payloadData?.code;
      if (
        code === 'DECLINED' ||
        code === 'INVALID_TOKEN' ||
        payloadData?.message?.toLowerCase?.().includes('token')
      ) {
        console.error('WebSocket auth rejected:', payloadData);
        this.isReady = false;
        this.broadcast('AuthError', payloadData);
        return;
      }
      this.broadcast('Error', payloadData);
      return;
    }

    if (!this.isReady) {
      this.eventsBuffer.push(data);
      return;
    }

    if (this.paused) {
      this.pauseBuffer.push(data);
      return;
    }

    this.broadcast(clientEvent, payloadData);
  }

  startHeartbeat(intervalSec) {
    this.stopHeartbeat();
    const seconds = Number(intervalSec);
    this.heartbeatIntervalSec = Number.isFinite(seconds) && seconds > 0 ? seconds : 60;
    const intervalMs = this.heartbeatIntervalSec * 1000;

    this.heartbeatTimer = setInterval(() => {
      if (this.isConnected() && this.isReady) {
        this.send({ event: 'Heartbeat', data: null });
      }
    }, intervalMs);
  }

  stopHeartbeat() {
    if (this.heartbeatTimer != null) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  pauseEvents() {
    this.paused = true;
  }

  resumeEvents() {
    this.paused = false;
    const queued = this.pauseBuffer.splice(0);
    for (const buffered of queued) {
      this.processIncomingMessage(buffered);
    }
  }

  broadcast(eventType, payloadData) {
    if (this.listeners[eventType]) {
      this.listeners[eventType].forEach((cb) => cb(payloadData));
    }
  }

  on(eventType, callback) {
    if (!this.listeners[eventType]) {
      this.listeners[eventType] = [];
    }
    this.listeners[eventType].push(callback);
  }

  off(eventType, callback) {
    if (this.listeners[eventType]) {
      this.listeners[eventType] = this.listeners[eventType].filter((cb) => cb !== callback);
    }
  }

  send(payload) {
    if (this.isConnected()) {
      this.ws.send(JSON.stringify(payload));
    } else {
      console.warn('Cannot send message, WebSocket is not open.');
    }
  }

  disconnect() {
    this.stopHeartbeat();
    if (this.ws) {
      this.intentionalClose = true;
      this.isReady = false;
      this.paused = false;
      this.pauseBuffer = [];
      this.ws.close();
    }
  }

  isConnected() {
    return this.ws && this.ws.readyState === WebSocket.OPEN;
  }
}
