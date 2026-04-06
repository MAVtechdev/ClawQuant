"""Coarse PnL attribution into entry timing, exit timing, and holding drift (toy decomposition)."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class TradeAttribution:
    entry_edge: float
    exit_edge: float
    drift: float


def attribute_pnl_buckets(
    entry_price: float,
    exit_price: float,
    path_high: float,
    path_low: float,
) -> TradeAttribution:
    """
    entry_edge: move from entry toward favorable extreme before exit bar (long: high - entry).
    exit_edge: residual from last path point to exit (simplified: exit - midpoint of range).
    drift: remainder so components sum to total return magnitude proxy.

    This is a *pedagogical* split for learning records — not brokerage-grade TCA.
    """
    total = exit_price - entry_price
    favorable = max(path_high - entry_price, 0.0) if total >= 0 else max(entry_price - path_low, 0.0)
    entry_edge = min(favorable, abs(total)) * (1.0 if total >= 0 else -1.0)
    mid = (path_high + path_low) / 2.0
    exit_edge = exit_price - mid
    drift = total - entry_edge - exit_edge
    return TradeAttribution(entry_edge=entry_edge, exit_edge=exit_edge, drift=drift)
