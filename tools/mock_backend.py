#!/usr/bin/env python3
"""Minimal mock backend for callmeju audio-link verification.

Listens on :8000, accepts POST /v1/converse with raw little-endian 16-bit PCM
in the body, and replies with a JSON carrying a short test tone (8 kHz, stereo)
so the device can verify capture -> upload -> playback end-to-end.
"""

import base64
import json
import math
import struct
from http.server import BaseHTTPRequestHandler, HTTPServer

SAMPLE_RATE = 8000
CHANNELS = 2
PORT = 8000


def make_tone(freq=440.0, duration=1.0, amplitude=20000):
    n = int(SAMPLE_RATE * duration)
    frames = bytearray()
    for i in range(n):
        v = int(amplitude * math.sin(2 * math.pi * freq * i / SAMPLE_RATE))
        # interleaved stereo: same value on both channels
        frames += struct.pack("<hh", v, v)
    return bytes(frames)


TONE_B64 = base64.b64encode(make_tone()).decode()


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path != "/v1/converse":
            self.send_response(404)
            self.end_headers()
            return
        length = int(self.headers.get("Content-Length", 0))
        pcm = self.rfile.read(length)
        print(f"[mock] received {length} bytes PCM")
        reply = {
            "text": f"mock: got {length} bytes, playing test tone",
            "audio_base64": TONE_B64,
            "continue_conversation": True,
        }
        body = json.dumps(reply).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    print(f"[mock] listening on :{PORT} (POST /v1/converse)")
    HTTPServer(("0.0.0.0", PORT), Handler).serve_forever()
