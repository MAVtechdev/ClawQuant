#![forbid(unsafe_code)]

//! Technical indicators over closing prices (and OHLC for ATR).
//! All functions return `Vec<Option<f64>>` aligned with input indices: `None` until
//! the window is warm.

/// Simple moving average.
pub fn sma(values: &[f64], period: usize) -> Vec<Option<f64>> {
    if period == 0 {
        return vec![None; values.len()];
    }
    let mut out = vec![None; values.len()];
    if values.len() < period {
        return out;
    }
    let mut sum: f64 = values[..period].iter().sum();
    out[period - 1] = Some(sum / period as f64);
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        out[i] = Some(sum / period as f64);
    }
    out
}

/// Exponential moving average (Wilder-style seed: first value is SMA of first `period`).
pub fn ema(values: &[f64], period: usize) -> Vec<Option<f64>> {
    if period == 0 || values.is_empty() {
        return vec![None; values.len()];
    }
    let mut out = vec![None; values.len()];
    let sma_init = match sma(values, period) {
        v if v.get(period - 1).and_then(|x| *x).is_some() => v[period - 1].unwrap(),
        _ => return out,
    };
    let k = 2.0 / (period as f64 + 1.0);
    out[period - 1] = Some(sma_init);
    let mut prev = sma_init;
    for i in period..values.len() {
        prev = k * values[i] + (1.0 - k) * prev;
        out[i] = Some(prev);
    }
    out
}

fn gains_losses(closes: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut gains = vec![0.0; closes.len()];
    let mut losses = vec![0.0; closes.len()];
    for i in 1..closes.len() {
        let d = closes[i] - closes[i - 1];
        if d >= 0.0 {
            gains[i] = d;
        } else {
            losses[i] = -d;
        }
    }
    (gains, losses)
}

/// Relative Strength Index (Wilder smoothing on average gain/loss).
pub fn rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    if period == 0 || closes.len() <= period {
        return vec![None; closes.len()];
    }
    let (gains, losses) = gains_losses(closes);
    let mut out = vec![None; closes.len()];

    let mut avg_gain: f64 = gains[1..=period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[1..=period].iter().sum::<f64>() / period as f64;

    let rs = if avg_loss == 0.0 {
        f64::INFINITY
    } else {
        avg_gain / avg_loss
    };
    out[period] = Some(100.0 - (100.0 / (1.0 + rs)));

    for i in (period + 1)..closes.len() {
        avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;
        let rs = if avg_loss == 0.0 {
            f64::INFINITY
        } else {
            avg_gain / avg_loss
        };
        out[i] = Some(100.0 - (100.0 / (1.0 + rs)));
    }
    out
}

/// MACD line (fast EMA − slow EMA), signal EMA of MACD, histogram.
pub fn macd(
    closes: &[f64],
    fast: usize,
    slow: usize,
    signal: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let n = closes.len();
    let empty = (vec![None; n], vec![None; n], vec![None; n]);
    if fast == 0 || slow == 0 || signal == 0 || slow <= fast {
        return empty;
    }
    let ef = ema(closes, fast);
    let es = ema(closes, slow);
    let mut line = vec![None; n];
    for i in 0..n {
        match (ef[i], es[i]) {
            (Some(a), Some(b)) => line[i] = Some(a - b),
            _ => {}
        }
    }
    // Build dense macd series for EMA (pad with 0 is wrong). Extract present values with index map.
    let mut macd_vals: Vec<f64> = Vec::with_capacity(n);
    let mut idx_map: Vec<usize> = Vec::with_capacity(n);
    for i in 0..n {
        if let Some(v) = line[i] {
            macd_vals.push(v);
            idx_map.push(i);
        }
    }
    if macd_vals.len() < signal {
        return (line, vec![None; n], vec![None; n]);
    }
    let sig_ema = ema(&macd_vals, signal);
    let mut signal_out = vec![None; n];
    let mut hist = vec![None; n];
    for (j, &orig_i) in idx_map.iter().enumerate() {
        if let Some(sv) = sig_ema[j] {
            signal_out[orig_i] = Some(sv);
            if let Some(lv) = line[orig_i] {
                hist[orig_i] = Some(lv - sv);
            }
        }
    }
    (line, signal_out, hist)
}

fn true_range(high: &[f64], low: &[f64], close: &[f64], i: usize) -> f64 {
    let h_l = high[i] - low[i];
    if i == 0 {
        return h_l;
    }
    let h_pc = (high[i] - close[i - 1]).abs();
    let l_pc = (low[i] - close[i - 1]).abs();
    h_l.max(h_pc).max(l_pc)
}

/// Average True Range (Wilder smoothing).
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = high.len().min(low.len()).min(close.len());
    if period == 0 || n == 0 {
        return vec![None; n];
    }
    let mut tr = vec![0.0; n];
    for i in 0..n {
        tr[i] = true_range(high, low, close, i);
    }
    let mut out = vec![None; n];
    if n < period {
        return out;
    }
    let first: f64 = tr[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(first);
    let mut prev = first;
    for i in period..n {
        prev = (prev * (period as f64 - 1.0) + tr[i]) / period as f64;
        out[i] = Some(prev);
    }
    out
}

/// Log-return sample standard deviation over `window` (ddof=1). If `annualize`, scales by √252.
pub fn rolling_volatility_log_returns(
    closes: &[f64],
    window: usize,
    annualize: bool,
) -> Vec<Option<f64>> {
    if window < 2 || closes.len() <= window {
        return vec![None; closes.len()];
    }
    let mut log_r = vec![0.0; closes.len()];
    for i in 1..closes.len() {
        log_r[i] = (closes[i] / closes[i - 1]).ln();
    }
    let mut out = vec![None; closes.len()];
    // need window returns -> window log_r points
    for i in window..closes.len() {
        let slice = &log_r[(i - window + 1)..=i];
        let m = slice.iter().sum::<f64>() / slice.len() as f64;
        let var = slice.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (slice.len() as f64 - 1.0);
        let mut sd = var.sqrt();
        if annualize {
            sd *= (252_f64).sqrt();
        }
        out[i] = Some(sd);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn sma_basic() {
        let c = [1.0, 2.0, 3.0, 4.0, 5.0];
        let s = sma(&c, 3);
        assert!(s[1].is_none());
        assert_relative_eq!(s[2].unwrap(), 2.0);
        assert_relative_eq!(s[4].unwrap(), 4.0);
    }

    #[test]
    fn rsi_bounded() {
        let c: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64) * 0.5).collect();
        let r = rsi(&c, 14);
        let last = r.last().and_then(|x| *x).expect("rsi");
        assert!(last > 0.0 && last < 100.0);
    }

    #[test]
    fn macd_lengths_align() {
        let c: Vec<f64> = (0..80).map(|i| 50.0 + (i as f64) * 0.1).collect();
        let (line, signal, hist) = macd(&c, 12, 26, 9);
        assert_eq!(line.len(), c.len());
        assert_eq!(signal.len(), c.len());
        assert_eq!(hist.len(), c.len());
        assert!(line.last().and_then(|x| *x).is_some());
        assert!(hist.last().and_then(|x| *x).is_some());
    }

    #[test]
    fn atr_positive() {
        let h = [10.0, 11.0, 12.0, 11.5, 12.5];
        let l = [9.0, 10.0, 11.0, 10.5, 11.5];
        let c = [9.5, 10.5, 11.5, 11.0, 12.0];
        let a = atr(&h, &l, &c, 3);
        assert!(a[4].unwrap() > 0.0);
    }
}
