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
  baseUrl?: string;
}

export interface UseEventStreamResult {
  events: EventEnvelope[];
  isConnected: boolean;
  clearEvents: () => void;
}

/**
 * Live domain events over Server-Sent Events, limited to what the session's user sees. The
 * session travels as the proxy's cookie, like any other request (EventSource sends no headers).
 */
export function useEventStream(options: UseEventStreamOptions = {}): UseEventStreamResult {
  const { topic, enabled = true, onEvent, maxBuffer = 100 } = options;

  const [events, setEvents] = useState<EventEnvelope[]>([]);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  useEffect(() => {
    if (!enabled) {
      setIsConnected(false);
      return;
    }

    const rawBase = (options.baseUrl ?? getApiGatewayBase()) || "";
    const cleanBase = rawBase.endsWith("/") ? rawBase.slice(0, -1) : rawBase;
    const apiBase = cleanBase
      ? cleanBase.endsWith("/admin/api")
        ? cleanBase
        : `${cleanBase}/admin/api`
      : "/admin/api";

    const queryParams = new URLSearchParams();
    if (topic) queryParams.set("topic", topic);

    const query = queryParams.toString();
    const eventSource = new EventSource(`${apiBase}/events/stream${query ? `?${query}` : ""}`);

    eventSource.onopen = () => {
      setIsConnected(true);
    };

    eventSource.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data);
        if (parsed && (parsed.id || parsed.topic)) {
          const envelope = parsed as EventEnvelope;
          setEvents((prev) => [envelope, ...prev].slice(0, maxBuffer));
          onEventRef.current?.(envelope);
        }
      } catch {
        // Ignore keep-alive frames
      }
    };

    eventSource.onerror = () => {
      setIsConnected(false);
    };

    return () => {
      eventSource.close();
    };
  }, [enabled, topic, maxBuffer]);

  return { events, isConnected, clearEvents };
}
