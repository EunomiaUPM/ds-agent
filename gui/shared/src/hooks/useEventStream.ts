/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import { useState, useEffect, useRef, useCallback } from "react";
import { EventEnvelope } from "../data/orval/model";
import { getApiGatewayBase } from "../data/orval-mutator";

export interface UseEventStreamOptions {
  topic?: string;
  enabled?: boolean;
  onEvent?: (event: EventEnvelope) => void;
  maxBuffer?: number;
  useWebSocket?: boolean;
  baseUrl?: string;
}

export interface UseEventStreamResult {
  events: EventEnvelope[];
  isConnected: boolean;
  clearEvents: () => void;
  sendWsMessage?: (msg: unknown) => void;
  connectionType: "sse" | "ws";
}

/**
 * Real-time event streaming hook supporting Server-Sent Events (SSE) and WebSockets.
 */
export function useEventStream(options: UseEventStreamOptions = {}): UseEventStreamResult {
  const { topic, enabled = true, onEvent, maxBuffer = 100, useWebSocket = false } = options;

  const [events, setEvents] = useState<EventEnvelope[]>([]);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  const wsRef = useRef<WebSocket | null>(null);

  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  const sendWsMessage = useCallback((msg: unknown) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(typeof msg === "string" ? msg : JSON.stringify(msg));
    }
  }, []);

  useEffect(() => {
    if (!enabled) {
      setIsConnected(false);
      return;
    }

    const token =
      typeof localStorage !== "undefined"
        ? localStorage.getItem("eunomia_token") ||
          localStorage.getItem("access_token") ||
          localStorage.getItem("pat_token")
        : null;

    const rawBase = (options.baseUrl ?? getApiGatewayBase()) || "";
    const cleanBase = rawBase.endsWith("/") ? rawBase.slice(0, -1) : rawBase;

    if (useWebSocket) {
      let wsUrl: string;
      if (cleanBase.startsWith("http")) {
        const wsPrefix = cleanBase.replace(/^http/, "ws");
        wsUrl = `${wsPrefix}/admin/api/ws${token ? `?token=${encodeURIComponent(token)}` : ""}`;
      } else {
        const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
        const host = window.location.host;
        wsUrl = `${protocol}//${host}/admin/api/ws${token ? `?token=${encodeURIComponent(token)}` : ""}`;
      }

      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setIsConnected(true);
        if (topic) {
          ws.send(JSON.stringify({ type: "subscribe", pattern: topic }));
        }
      };

      ws.onmessage = (event) => {
        try {
          const parsed = JSON.parse(event.data);
          if (parsed && (parsed.id || parsed.topic)) {
            const envelope = parsed as EventEnvelope;
            setEvents((prev) => [envelope, ...prev].slice(0, maxBuffer));
            if (onEventRef.current) {
              onEventRef.current(envelope);
            }
          }
        } catch {
          // Ignore non-JSON heartbeat or control frames
        }
      };

      ws.onerror = () => {
        setIsConnected(false);
      };

      ws.onclose = () => {
        setIsConnected(false);
      };

      return () => {
        ws.close();
        wsRef.current = null;
      };
    } else {
      // Use SSE (/admin/api/events/stream)
      const queryParams = new URLSearchParams();
      if (topic) queryParams.set("topic", topic);
      if (token) queryParams.set("token", token);

      const streamPath = cleanBase ? `${cleanBase}/admin/api/events/stream` : "/admin/api/events/stream";
      const sseUrl = `${streamPath}${queryParams.toString() ? `?${queryParams.toString()}` : ""}`;
      const eventSource = new EventSource(sseUrl);

      eventSource.onopen = () => {
        setIsConnected(true);
      };

      eventSource.onmessage = (event) => {
        try {
          const parsed = JSON.parse(event.data);
          if (parsed && (parsed.id || parsed.topic)) {
            const envelope = parsed as EventEnvelope;
            setEvents((prev) => [envelope, ...prev].slice(0, maxBuffer));
            if (onEventRef.current) {
              onEventRef.current(envelope);
            }
          }
        } catch {
          // Ignore ping or keepalive
        }
      };

      eventSource.onerror = () => {
        setIsConnected(false);
      };

      return () => {
        eventSource.close();
      };
    }
  }, [enabled, topic, maxBuffer, useWebSocket]);

  return {
    events,
    isConnected,
    clearEvents,
    sendWsMessage: useWebSocket ? sendWsMessage : undefined,
    connectionType: useWebSocket ? "ws" : "sse",
  };
}
