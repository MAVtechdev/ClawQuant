"""Delayed outcome labeling: forward return after entry at fixed bar horizon."""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np


@dataclass(frozen=True)
class HorizonLabel:
    horizon_bars: int
    simple_return: float
    realized: bool  # False if series ended before horizon


def label_trade_horizon_return(
    entry_price: float,
    future_closes: np.ndarray,
    horizon_bars: int,
) -> HorizonLabel:
    if horizon_bars < 1:
        raise ValueError("horizon_bars must be >= 1")
    if future_closes.size < horizon_bars:
        return HorizonLabel(
            horizon_bars=horizon_bars,
            simple_return=float("nan"),
            realized=False,
        )
    exit_px = float(future_closes[horizon_bars - 1])
    r = (exit_px / entry_price) - 1.0 if entry_price != 0 else float("nan")
    return HorizonLabel(
        horizon_bars=horizon_bars,
        simple_return=float(r),
        realized=True,
    )
