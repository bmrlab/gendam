/// This file implement Kernel Temporal Segmentation algorithm in Rust
/// reference python code: https://github.com/TatsuyaShirakawa/KTS?tab=readme-ov-file
use ndarray::Array2;

#[allow(warnings)]
/// `cpd_auto` function in KTS
///
/// ```python
/// def cpd_auto(K, ncp, vmax, desc_rate=1, **kwargs):
///     m = ncp
///     (_, scores) = cpd_nonlin(K, m, backtrack=False, **kwargs)
///
///     N = K.shape[0]
///     N2 = N * desc_rate  # length of the video before subsampling
///
///     penalties = np.zeros(m + 1)
///     # Prevent division by zero (in case of 0 changes)
///     ncp = np.arange(1, m + 1)
///     penalties[1:] = (vmax * ncp / (2.0 * N2)) * (np.log(float(N2) / ncp) + 1)
///
///     costs = scores / float(N) + penalties
///     m_best = np.argmin(costs)
///     (cps, scores2) = cpd_nonlin(K, m_best, **kwargs)
///
///     return (cps, costs)
/// ```
pub fn cpd_auto(
    K: Array2<f64>,
    ncp: usize,
    vmax: f64,
    desc_rate: Option<usize>,
) -> anyhow::Result<Vec<usize>> {
    let m = ncp;
    let N = K.shape()[0];
    let N2 = N * desc_rate.unwrap_or(1);

    let (_, scores) = cpd_nonlin(K.clone(), m, None, None, Some(false))?;

    let mut penalties = vec![0.0; ncp + 1];

    for i in 1..=ncp {
        penalties[i] = (vmax * i as f64 / (2.0 * N as f64)) * ((N2 as f64).ln() / (i as f64) + 1.0);
    }

    let mut costs = vec![0.0; ncp + 1];

    for i in 0..=ncp {
        costs[i] = scores[i] / (N as f64) + penalties[i];
    }

    let m_best = costs
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(b.1))
        .map(|(idx, _)| idx)
        .unwrap();

    let (cps, _) = cpd_nonlin(K, m_best, None, None, None)?;

    Ok(cps)
}

///`cpd_nonlin` function in KTS
///
/// ```python
/// def cpd_nonlin(K, ncp, lmin=1, lmax=100000, backtrack=True, verbose=True,
///                out_scatters=None):
///     m = int(ncp)  # prevent numpy.int64
///
///     (n, n1) = K.shape
///     assert (n == n1), "Kernel matrix awaited."
///
///     assert (n >= (m + 1) * lmin)
///     assert (n <= (m + 1) * lmax)
///     assert (lmax >= lmin >= 1)
///
///     if verbose:
///         # print "n =", n
///         print("Precomputing scatters.")
///     J = calc_scatters(K)
///
///     if out_scatters != None:
///         out_scatters[0] = J
///
///     if verbose:
///         print("Inferring best change points.")
///     I = 1e101 * np.ones((m + 1, n + 1))
///     I[0, lmin:lmax] = J[0, lmin - 1:lmax - 1]
///
///     if backtrack:
///         p = np.zeros((m + 1, n + 1), dtype=int)
///     else:
///         p = np.zeros((1, 1), dtype=int)
///
///     for k in range(1, m + 1):
///         for l in range((k + 1) * lmin, n + 1):
///             I[k, l] = 1e100
///             for t in range(max(k * lmin, l - lmax), l - lmin + 1):
///                 c = I[k - 1, t] + J[t, l - 1]
///                 if (c < I[k, l]):
///                     I[k, l] = c
///                     if (backtrack == 1):
///                         p[k, l] = t
///
///     # Collect change points
///     cps = np.zeros(m, dtype=int)
///
///     if backtrack:
///         cur = n
///         for k in range(m, 0, -1):
///             cps[k - 1] = p[k, cur]
///             cur = cps[k - 1]
///
///     scores = I[:, n].copy()
///     scores[scores > 1e99] = np.inf
///     return cps, scores
/// ```
#[allow(warnings)]
fn cpd_nonlin(
    K: Array2<f64>,
    ncp: usize,
    lmin: Option<usize>,
    lmax: Option<usize>,
    backtrack: Option<bool>,
) -> anyhow::Result<(Vec<usize>, Vec<f64>)> {
    let n = K.shape()[0];
    let n1 = K.shape()[1];

    let lmin = lmin.unwrap_or(1);
    let lmax = lmax.unwrap_or(100_000);

    assert!(n == n1, "K should be square");
    assert!(n >= (ncp + 1) * lmin, "");
    assert!(n <= (ncp + 1) * lmax, "");
    assert!(lmax >= lmin && lmin >= 1, "");

    let J = calc_scatters(K.clone());

    let mut I: Array2<f64> = Array2::ones((ncp + 1, n + 1));
    let mut I = I * 1e101;
    for i in lmin..lmax.min(n + 1) {
        I[[0, i]] = J[[0, i - 1]]
    }

    let backtrace = backtrack.unwrap_or(true);
    let mut p = {
        if backtrace {
            Array2::<usize>::zeros((ncp + 1, n + 1))
        } else {
            Array2::<usize>::zeros((1, 1))
        }
    };

    for k in 1..(ncp + 1) {
        for l in ((k + 1) * lmin)..(n + 1) {
            I[[k, l]] = 1e100;
            for t in (k * lmin).max(if l > lmax { l - lmax } else { 0 })..(l - lmin + 1) {
                let c = I[[k - 1, t]] + J[[t, l - 1]];
                if c < I[[k, l]] {
                    I[[k, l]] = c;
                    if backtrace {
                        p[[k, l]] = t;
                    }
                }
            }
        }
    }

    let mut cps = vec![0; ncp];

    if backtrace {
        let mut cur = n;
        for k in (1..=ncp).rev() {
            cps[k - 1] = p[[k, cur]];
            cur = cps[k - 1];
        }
    }

    let scores = I.column(n).clone();

    let scores = scores
        .iter()
        .map(|&v| if v > 1e99 { f64::MAX } else { v })
        .collect();

    Ok((cps, scores))
}

/// `calc_scatters` function in KTS
///
/// ```python
/// def calc_scatters(K):
///     n = K.shape[0]
///     K1 = np.cumsum([0] + list(np.diag(K)))
///     K2 = np.zeros((n + 1, n + 1)).astype(np.double())
///     K2[1:, 1:] = np.cumsum(np.cumsum(K, 0), 1)  # TODO: use the fact that K - symmetric

///     scatters = np.zeros((n, n))

///     for i in range(n):
///         for j in range(i, n):
///             scatters[i, j] = K1[j + 1] - K1[i] - (K2[j + 1, j + 1] + K2[i, i] - K2[j + 1, i] - K2[i, j + 1]) / (
///                         j - i + 1)
///     return scatters
/// ```
#[allow(warnings)]
fn calc_scatters(K: Array2<f64>) -> Array2<f64> {
    let n = K.shape()[0];
    let mut K_diag = Vec::with_capacity(n);
    for i in 0..n {
        K_diag.push(K[[i, i]]);
    }

    let mut K1 = Vec::with_capacity(n + 1);

    let mut sum = 0.0;
    K1.push(0.0);
    for i in 0..n {
        sum += K_diag[i];
        K1.push(sum);
    }

    let mut K2: Array2<f64> = Array2::default((n + 1, n + 1));

    // np.cumsum(K, 0)
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            sum += K[[j, i]];
            K2[[j + 1, i + 1]] = sum;
        }
    }

    // np.cumsum(np.cumsum(K, 0), 1)
    for i in 0..n + 1 {
        let mut sum = 0.0;
        for j in 0..n + 1 {
            sum += K2[[i, j]];
            K2[[i, j]] = sum;
        }
    }

    let mut scatters: Array2<f64> = Array2::default((n, n));

    for i in 0..n {
        for j in i..n {
            scatters[[i, j]] = K1[j + 1]
                - K1[i]
                - (K2[[j + 1, j + 1]] + K2[[i, i]] - K2[[j + 1, i]] - K2[[i, j + 1]])
                    / ((j - i + 1) as f64)
        }
    }

    scatters
}

#[test_log::test]
fn test_calc_scatters() {
    use ndarray::array;
    // this is matrix is normalized by row
    let array = array![
        [0.18257385, 0.36514771, 0.54772156, 0.73029541],
        [0.35634801, 0.44543501, 0.53452201, 0.62360901],
        [0.40824805, 0.4665692, 0.52489035, 0.5832115],
        [0.43274214, 0.47601635, 0.51929056, 0.56256478]
    ];

    let result = calc_scatters(array);
    tracing::info!("{:?}", result);
}

#[test_log::test]
fn test_cpd_nonlin() {
    use ndarray::array;
    // this is matrix is normalized by row
    let array = array![
        [0.18257385, 0.36514771, 0.54772156, 0.73029541],
        [0.35634801, 0.44543501, 0.53452201, 0.62360901],
        [0.40824805, 0.4665692, 0.52489035, 0.5832115],
        [0.43274214, 0.47601635, 0.51929056, 0.56256478]
    ];

    let res = cpd_nonlin(array, 2, None, None, None);

    tracing::info!("{:?}", res);

    assert!(res.is_ok())
}

#[test_log::test]
fn test_cpd_auto() {
    use ndarray::array;
    // this is matrix is normalized by row
    let array = array![
        [0.18257385, 0.36514771, 0.54772156, 0.73029541],
        [0.35634801, 0.44543501, 0.53452201, 0.62360901],
        [0.40824805, 0.4665692, 0.52489035, 0.5832115],
        [0.43274214, 0.47601635, 0.51929056, 0.56256478]
    ];

    let res = cpd_auto(array, 2, 1.0, None);

    tracing::info!("{:?}", res);

    assert!(res.is_ok())
}
