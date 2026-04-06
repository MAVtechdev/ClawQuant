"""Compare candidate vs baseline equity curves (simple risk-aware summaries)."""

from __future__ import annotations

import numpy as np
import pandas as pd


def _daily_returns(equity: pd.Series) -> pd.Series:
    s = equity.sort_index().astype(float)
    return s.pct_change().dropna()


def benchmark_summary(equity: pd.Series, risk_free_annual: float = 0.0) -> dict[str, float]:
    """Annualized Sharpe (252 days), max drawdown, total return."""
    r = _daily_returns(equity)
    if r.empty or r.std(ddof=1) == 0:
        sharpe = 0.0
    else:
        excess = r.mean() * 252 - risk_free_annual
        sharpe = float(excess / (r.std(ddof=1) * np.sqrt(252)))
    cum = equity.sort_index().astype(float)
    peak = cum.cummax()
    dd = float(((cum / peak) - 1.0).min())
    total_ret = float(cum.iloc[-1] / cum.iloc[0] - 1.0) if len(cum) >= 2 else 0.0
    return {"sharpe_annual": sharpe, "max_drawdown": dd, "total_return": total_ret}


def compare_equity_curves(
    baseline: pd.Series,
    candidate: pd.Series,
    risk_free_annual: float = 0.0,
) -> dict[str, dict[str, float]]:
    """Side-by-side summaries for promotion gates."""
    return {
        "baseline": benchmark_summary(baseline, risk_free_annual),
        "candidate": benchmark_summary(candidate, risk_free_annual),
    }
