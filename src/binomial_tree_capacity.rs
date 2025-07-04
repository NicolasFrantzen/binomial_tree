use const_for::const_for;

pub(crate) const fn calculate_step_capacity(step_number: usize) -> usize {
    if step_number > 1 {
        calculate_capacity(step_number) - calculate_capacity(step_number-1)
    }
    else
    {
        1usize
    }
}

// A special case of binomial formula with k = 2 and n = number_of_steps + 2
pub(crate) const fn calculate_capacity(number_of_steps: usize) -> usize {
    binom(number_of_steps + 2usize, 2usize)
}

const fn binom(n: usize, k: usize) -> usize {
    let mut res = 1;
    const_for!(i in 0..k => {
        res = res * (n - i) /
            (i + 1);
    });
    res
}