"""Causal replay hooks over OHLCV DataFrames (pairs with Rust `replay-core` for parity checks)."""

from __future__ import annotations

from collections.abc import Callable

import numpy as np
import pandas as pd


def bars_to_close_array(df: pd.DataFrame, close_col: str = "close") -> np.ndarray:
    if close_col not in df.columns:
        raise KeyError(close_col)
    return df[close_col].to_numpy(dtype=np.float64)


def replay_indicator_hooks(
    closes: np.ndarray,
    *,
    on_step: Callable[[int, np.ndarray], None] | None = None,
) -> None:
    """
    Walk closes[0:i+1] for each i. Optional callback receives (i, prefix_slice) for custom features.
    Useful to mirror Rust replay-bar semantics in Python tests.
    """
    for i in range(len(closes)):
        if on_step is not None:
            on_step(i, closes[: i + 1])
