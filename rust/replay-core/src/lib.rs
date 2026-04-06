#![forbid(unsafe_code)]

//! Deterministic replay: walk an OHLCV series and produce per-bar indicator snapshots.
//! Used for offline evaluation and Parity checks against the TypeScript analysis kit.

use clawquant_indicator_engine::{atr, ema, rsi, sma, rolling_volatility_log_returns};

/// Single candle (e.g. 1m bar).
#[derive(Clone, Copy, Debug)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Indicator values available at bar index `i` after full-series computation (causal: values at i use data ≤ i).
#[derive(Clone, Debug, Default)]
pub struct IndicatorSnapshot {
    pub sma_20: Option<f64>,
    pub ema_12: Option<f64>,
    pub rsi_14: Option<f64>,
    pub atr_14: Option<f64>,
    /// Sample stdev of log returns, annualized (√252).
    pub log_return_vol_annualized_20: Option<f64>,
}

/// Replay candles and attach SMA(20), EMA(12), RSI(14), ATR(14) per bar.
pub fn replay_with_indicators(candles: &[Candle]) -> Vec<IndicatorSnapshot> {
    let n = candles.len();
    let mut out: Vec<IndicatorSnapshot> = (0..n).map(|_| IndicatorSnapshot::default()).collect();
    if n == 0 {
        return out;
    }
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

    let sma20 = sma(&closes, 20);
    let ema12 = ema(&closes, 12);
    let rsi14 = rsi(&closes, 14);
    let atr14 = atr(&highs, &lows, &closes, 14);
    let vol20 = rolling_volatility_log_returns(&closes, 20, true);

    for i in 0..n {
        out[i].sma_20 = sma20[i];
        out[i].ema_12 = ema12[i];
        out[i].rsi_14 = rsi14[i];
        out[i].atr_14 = atr14[i];
        out[i].log_return_vol_annualized_20 = vol20[i];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_uptrend(n: usize) -> Vec<Candle> {
        (0..n)
            .map(|i| {
                let p = 100.0 + i as f64 * 0.1;
                Candle {
                    open: p,
                    high: p + 0.05,
                    low: p - 0.05,
                    close: p + 0.02,
                    volume: 1.0,
                }
            })
            .collect()
    }

    #[test]
    fn replay_populates_late_bars() {
        let candles = synth_uptrend(50);
        let snaps = replay_with_indicators(&candles);
        assert!(snaps[49].sma_20.is_some());
        assert!(snaps[49].rsi_14.is_some());
    }
}