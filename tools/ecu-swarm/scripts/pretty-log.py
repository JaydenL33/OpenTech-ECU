#!/usr/bin/env python3
"""Human-friendly printer for OpenCode/Codex/Claude JSONL (not a full transcript parser)."""
import json
import sys
from datetime import datetime

role = sys.argv[1] if len(sys.argv) > 1 else 'agent'
for raw in sys.stdin:
    raw = raw.rstrip('\n')
    if not raw:
        continue
    try:
        event = json.loads(raw)
    except json.JSONDecodeError:
        print(f'[{role}] {raw[:1200]}', flush=True)
        continue
    kind = event.get('type', event.get('event', 'event'))
    item = event.get('item') or {}
    text = event.get('text') or event.get('message') or ''
    if isinstance(text, dict):
        text = text.get('text') or text.get('content') or ''
    if kind == 'assistant' and isinstance(event.get('message'), dict):
        pieces = event['message'].get('content') or []
        text = '\n'.join(p.get('text', '') for p in pieces if isinstance(p, dict) and p.get('type') == 'text')
    if kind in ('item.completed', 'item.started', 'item.updated'):
        t = item.get('type', 'item')
        text = item.get('text') or item.get('command') or item.get('title') or t
        kind = f'{kind}/{t}'
    if kind == 'stream_event':
        delta = event.get('event', {}).get('delta', {})
        text = delta.get('text') or ''
        if not text:
            continue
    if kind == 'content_block_delta':
        text = event.get('delta', {}).get('text', '')
        if not text:
            continue
    if kind == 'tool_use' and isinstance(event.get('part'), dict):
        text = event['part'].get('tool', '')
    if kind == 'text' and isinstance(event.get('part'), dict):
        text = event['part'].get('text', '')
    if not text:
        text = event.get('sessionID') or event.get('thread_id') or ''
    if not isinstance(text, str):
        text = str(text)
    compact = text.replace('\r', ' ').strip()
    if len(compact) > 900:
        compact = compact[:900] + '…'
    if compact or kind in ('turn.completed', 'turn.failed', 'result', 'error'):
        print(f'[{datetime.now().strftime("%H:%M:%S")}] [{role}] {kind}: {compact}', flush=True)
