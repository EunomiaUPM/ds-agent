#!/usr/bin/env python3
# Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
# SPDX-License-Identifier: GPL-3.0-or-later
"""Zero-dependency mock upstream & consumer sink for ds-protocol examples.

Exposes three logical surfaces on port 8080:
1. EDACCIT-style ArcGIS FeatureServer (HTTP PULL upstream):
   - GET /arcgis/rest/services/edaccit/FeatureServer/0/query?token=<API_KEY>
2. ECOSTARS-style ESG Webhook Publisher (HTTP PUSH upstream):
   - POST   /esg/webhooks        (Bearer auth; stores callbackUrl, returns {"id": "sub-N"})
   - GET    /esg/webhooks        (lists active webhook subscriptions)
   - DELETE /esg/webhooks/<id>   (Bearer auth; removes subscription)
   - POST   /esg/emit            (pushes an ESG event to all active callbackUrls)
3. Consumer Webhook Sink (HTTP PUSH destination):
   - POST   /consumer-sink/events
   - GET    /consumer-sink/events
   - DELETE /consumer-sink/events
"""

from __future__ import annotations

import json
import os
import threading
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

EXPECTED_ARCGIS_TOKEN = os.environ.get("ARCGIS_API_KEY", "edaccit-demo-key-2026")
EXPECTED_ESG_BEARER = os.environ.get("ESG_WEBHOOK_TOKEN", "ecostars-demo-secret-2026")

LOCK = threading.Lock()
SUB_SEQ = 0
SUBSCRIPTIONS: dict[str, dict[str, str]] = {}
RECEIVED_SINK_EVENTS: list[dict[str, object]] = []

GEOJSON_FEATURES = {
    "type": "FeatureCollection",
    "name": "EDACCIT_Transport_Climate_Stations",
    "features": [
        {
            "type": "Feature",
            "id": 1,
            "geometry": {"type": "Point", "coordinates": [-3.5676, 40.4839]},
            "properties": {
                "station_code": "LEMD-01",
                "corridor": "Madrid-Barajas Aviation & Access",
                "temp_c": 24.6,
                "wind_kt": 11.2,
                "visibility_m": 9999,
            },
        },
        {
            "type": "Feature",
            "id": 2,
            "geometry": {"type": "Point", "coordinates": [-3.4821, 40.4512]},
            "properties": {
                "station_code": "A2-KM15",
                "corridor": "A-2 Henares Freight Corridor",
                "temp_c": 25.1,
                "heavy_vehicles_h": 418,
                "precipitation_mm": 0.0,
            },
        },
        {
            "type": "Feature",
            "id": 3,
            "geometry": {"type": "Point", "coordinates": [-3.6883, 40.3917]},
            "properties": {
                "station_code": "M30-SUR",
                "corridor": "M-30 Southern Ring Urban Mobility",
                "temp_c": 26.0,
                "nox_ug_m3": 37.4,
                "precipitation_mm": 0.0,
            },
        },
    ],
}


class MockHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _send_json(self, status: int, payload: object) -> None:
        raw = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def _read_json_body(self) -> dict[str, object]:
        length = int(self.headers.get("Content-Length", "0"))
        if length <= 0:
            return {}
        raw = self.rfile.read(length)
        return json.loads(raw.decode("utf-8")) if raw else {}

    def _check_esg_bearer(self) -> bool:
        auth = self.headers.get("Authorization", "")
        return auth == f"Bearer {EXPECTED_ESG_BEARER}"

    def do_GET(self) -> None:  # noqa: N802
        parsed = urllib.parse.urlsplit(self.path)
        query = urllib.parse.parse_qs(parsed.query)

        if parsed.path == "/health":
            self._send_json(200, {"status": "ok"})
            return

        if parsed.path.startswith("/arcgis/rest/services/edaccit/FeatureServer/0/query"):
            token = (query.get("token") or [""])[0]
            if token != EXPECTED_ARCGIS_TOKEN:
                self._send_json(
                    401,
                    {"error": "missing or invalid ArcGIS API key in query param `token`"},
                )
                return
            response = dict(GEOJSON_FEATURES)
            response["queryEcho"] = {
                k: v[0] for k, v in query.items() if k.lower() != "token"
            }
            self._send_json(200, response)
            return

        if parsed.path == "/esg/webhooks":
            with LOCK:
                items = [
                    {
                        "id": sub_id,
                        "topic": sub["topic"],
                        # Redact capability token in diagnostic listing
                        "callbackOrigin": urllib.parse.urlsplit(sub["callbackUrl"])._replace(
                            query=""
                        ).geturl(),
                    }
                    for sub_id, sub in SUBSCRIPTIONS.items()
                ]
            self._send_json(200, {"count": len(items), "items": items})
            return

        if parsed.path == "/consumer-sink/events":
            with LOCK:
                events = list(RECEIVED_SINK_EVENTS)
            self._send_json(200, {"count": len(events), "items": events})
            return

        self._send_json(404, {"error": f"unknown GET route: {parsed.path}"})

    def do_POST(self) -> None:  # noqa: N802
        global SUB_SEQ
        parsed = urllib.parse.urlsplit(self.path)

        if parsed.path == "/esg/webhooks":
            if not self._check_esg_bearer():
                self._send_json(401, {"error": "missing or invalid ESG Bearer token"})
                return
            body = self._read_json_body()
            callback_url = str(body.get("callbackUrl", ""))
            topic = str(body.get("topic", "fleet.esg.rating"))
            if not callback_url:
                self._send_json(400, {"error": "callbackUrl is required"})
                return
            with LOCK:
                SUB_SEQ += 1
                sub_id = f"sub-{SUB_SEQ}"
                SUBSCRIPTIONS[sub_id] = {
                    "callbackUrl": callback_url,
                    "topic": topic,
                }
            self._send_json(201, {"id": sub_id, "status": "active", "topic": topic})
            return

        if parsed.path == "/esg/emit":
            body = self._read_json_body()
            event_payload = {
                "eventType": "EcostarsFleetAssessmentUpdated",
                "emittedAt": datetime.now(timezone.utc).isoformat(),
                "fleetId": body.get("fleetId", "ES-MAD-BUS-042"),
                "euroRating": body.get("euroRating", "5-Star-ZeroTailpipe"),
                "co2GramsPerKm": body.get("co2GramsPerKm", 18.4),
            }
            raw_event = json.dumps(event_payload).encode("utf-8")
            with LOCK:
                targets = list(SUBSCRIPTIONS.items())
            deliveries = []
            for sub_id, sub in targets:
                req = urllib.request.Request(
                    sub["callbackUrl"],
                    data=raw_event,
                    headers={"Content-Type": "application/json"},
                    method="POST",
                )
                try:
                    with urllib.request.urlopen(req, timeout=5) as resp:
                        deliveries.append({"subscriptionId": sub_id, "status": resp.status})
                except urllib.error.HTTPError as exc:
                    deliveries.append({"subscriptionId": sub_id, "status": exc.code})
                except Exception as exc:  # noqa: BLE001
                    deliveries.append({"subscriptionId": sub_id, "error": str(exc)})
            self._send_json(200, {"delivered": deliveries, "event": event_payload})
            return

        if parsed.path == "/consumer-sink/events":
            length = int(self.headers.get("Content-Length", "0"))
            raw_bytes = self.rfile.read(length) if length > 0 else b""
            if raw_bytes:
                try:
                    body: object = json.loads(raw_bytes.decode("utf-8"))
                except Exception:  # noqa: BLE001
                    body = raw_bytes.decode("utf-8", errors="replace")
            else:
                body = {}
            captured_headers = {
                k.lower(): v
                for k, v in self.headers.items()
                if k.lower() in ("authorization", "content-type", "x-forwarded-for")
                or k.lower().startswith("x-kafka-")
            }
            record = {
                "path": self.path,
                "headers": captured_headers,
                "payload": body,
            }
            with LOCK:
                RECEIVED_SINK_EVENTS.append(record)
            self._send_json(200, {"received": True, "total": len(RECEIVED_SINK_EVENTS)})
            return

        self._send_json(404, {"error": f"unknown POST route: {parsed.path}"})

    def do_DELETE(self) -> None:  # noqa: N802
        parsed = urllib.parse.urlsplit(self.path)

        if parsed.path.startswith("/esg/webhooks/"):
            if not self._check_esg_bearer():
                self._send_json(401, {"error": "missing or invalid ESG Bearer token"})
                return
            sub_id = parsed.path.rsplit("/", 1)[-1]
            with LOCK:
                existed = SUBSCRIPTIONS.pop(sub_id, None)
            if existed is None:
                self._send_json(404, {"error": f"subscription {sub_id} not found"})
                return
            self._send_json(200, {"deleted": sub_id, "status": "unsubscribed"})
            return

        if parsed.path == "/consumer-sink/events":
            with LOCK:
                RECEIVED_SINK_EVENTS.clear()
            self._send_json(200, {"cleared": True})
            return

        self._send_json(404, {"error": f"unknown DELETE route: {parsed.path}"})


def main() -> None:
    port = int(os.environ.get("PORT", "8080"))
    server = ThreadingHTTPServer(("0.0.0.0", port), MockHandler)
    print(f"mock-backend listening on 0.0.0.0:{port}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
