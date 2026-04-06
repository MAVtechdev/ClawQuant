import numpy as np
import pandas as pd

from claw_quant.learning.labelers import label_trade_horizon_return
from claw_quant.research.backtests import TradeEvent, equity_curve_from_trades
from claw_quant.research.benchmarks import compare_equity_curves


def test_horizon_label():
    fut = np.array([100.0, 101.0, 105.0])
    h = label_trade_horizon_return(100.0, fut, 3)
    assert h.realized and abs(h.simple_return - 0.05) < 1e-9


def test_equity_and_benchmark():
    ix = pd.date_range("2024-01-01", periods=5, freq="D")
    trades = [
        TradeEvent(ix[1], 500.0),
        TradeEvent(ix[3], -200.0),
    ]
    eq = equity_curve_from_trades(trades, initial_equity=10_000.0)
    out = compare_equity_curves(eq, eq)
    assert out["baseline"]["total_return"] == out["candidate"]["total_return"]
