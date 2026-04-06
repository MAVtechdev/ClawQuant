# ClawQuant (Python)

Research, labeling, attribution, benchmarks, and replay framing for ClawQuant.

Install (editable):

```bash
cd python
python -m venv .venv && source .venv/bin/activate
pip install -e ".[dev]"
pytest
```

This package is **intentionally independent** from the TypeScript runtime; exchange JSON and `data/` artifacts are the integration surface.
