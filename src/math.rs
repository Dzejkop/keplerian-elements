const MAX_STEPS: usize = 100_000;

/// Approximates the root of a function using the Newton-Raphson method.
///
/// # Arguments
/// f - The function to approximate the root of.
/// f_prime - The derivative of the function.
/// x0 - The initial guess.
/// epsilon - The maximum error allowed.
///
/// # Returns
/// The approximate root of the function
pub fn newton_approx(
    f: impl Fn(f32) -> f32,
    f_prime: impl Fn(f32) -> f32,
    x0: f32,
    epsilon: f32,
) -> f32 {
    let mut x = x0;

    for _ in 0..MAX_STEPS {
        let x_next = x - f(x) / f_prime(x);

        let error = (x_next - x).abs();

        if error < epsilon {
            return x_next;
        }

        x = x_next;
    }

    panic!(
        "Failed to converge after {} iterations (x0 = {}, x = {})",
        MAX_STEPS, x0, x
    );
}
