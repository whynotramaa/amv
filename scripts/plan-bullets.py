"""Refresh the literal PLAN bullet inventory without losing recorded evidence."""
from pathlib import Path
import hashlib
import re

root = Path(__file__).resolve().parent.parent
plan = (root / 'PLAN.md').read_text().splitlines()
output = root / 'PLAN_BULLETS.md'
previous = output.read_text() if output.exists() else ''
recorded = {m[1]: (m[0], m[2]) for m in re.findall(r'^- \[([ x])\] `(B-[a-f0-9]+)`.* Evidence: (.*)$', previous, re.M)}
rows = []
section = 'Preamble'
code = False
i = 0
while i < len(plan):
    line = plan[i]
    if line.lstrip().startswith('```'):
        code = not code
    if not code and line.startswith('#'):
        section = line.lstrip('#').strip()
    if not code and re.match(r'^\s*-\s+\S', line):
        start = i + 1
        text = re.sub(r'^\s*-\s+', '', line).strip()
        while i + 1 < len(plan) and plan[i + 1].startswith('    ') and plan[i + 1].strip() and not re.match(r'^\s*-\s+', plan[i + 1]):
            i += 1
            text += ' ' + plan[i].strip()
        identifier = 'B-' + hashlib.sha256((section + '\n' + text).encode()).hexdigest()[:12]
        state, evidence = recorded.get(identifier, (' ', 'pending individual verification; see REQUIREMENTS.md for the section assessment.'))
        rows.append((section, f'- [{state}] `{identifier}` · PLAN lines {start}–{i + 1}: {text} Evidence: {evidence}'))
    i += 1
assert len(rows) > 150, 'Unexpectedly small plan: inspect input before replacing the inventory.'
assert len(rows) == len({row.split('`')[1] for _, row in rows}), 'Duplicate identifiers need disambiguation.'
parts = ['# Literal PLAN bullet inventory\n', f'{len(rows)} bullets outside code fences. This includes informational reference bullets; those are not independent runtime obligations. An unchecked row is not a bug claim. Mark a row complete only with evidence. REQUIREMENTS.md supplies the grouped assessment.\n']
last = None
for section, row in rows:
    if section != last:
        parts.append('\n## ' + section + '\n')
        last = section
    parts.append(row)
output.write_text('\n'.join(parts) + '\n')
print(f'Wrote {output.name}: {len(rows)} bullets; preserved {len(recorded)} evidence records.')
