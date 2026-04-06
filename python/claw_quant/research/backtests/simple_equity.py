"""Equity curve from discrete trade events (no fill simulation — explicit PnL per trade)."""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import pandas as pd


@dataclass(frozen=True)
class TradeEvent:
    """One closed trade with timestamp and realized PnL in account currency."""

    ts: pd.Timestamp
    pnl: float


def equity_curve_from_trades(
    trades: list[TradeEvent],
    initial_equity: float = 100_000.0,
) -> pd.Series:
    if not trades:
        return pd.Series([initial_equity], index=pd.DatetimeIndex([pd.Timestamp.utcnow()]))
    df = pd.DataFrame([{"ts": t.ts, "pnl": t.pnl} for t in trades]).sort_values("ts")
    equity = initial_equity + np.cumsum(df["pnl"].to_numpy(dtype=float))
    return pd.Series(equity, index=pd.DatetimeIndex(df["ts"]), name="equity")
