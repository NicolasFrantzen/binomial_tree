use crate::binomial_tree_map::nodes::NodeNameTrait;
use crate::binomial_tree_map::{BinomialTreeMapImpl, BinomialTreeStackImpl, GetValue};
use crate::instruments::OptionContract;

use std::fmt;
use std::marker::PhantomData;

pub struct CoxRossRubenstein<Stack, V = smoothing::None, U = truncation::None> {
    stack: Stack,
    params: VolatilityParameters,
    spot: Spot,
    expiry: Expiry,
    discount_factor: f32,
    time_step: f32,
    _phantom_data: PhantomData<V>,
    _phantom_data2: PhantomData<U>,
}

#[allow(private_bounds)]
impl<Stack: BinomialTreeStackImpl, V: smoothing::ValueAtLeaf, U: truncation::ValueAtBorder>
    CoxRossRubenstein<Stack, V, U>
{
    pub fn new(
        stack: Stack,
        initial_price: Spot,
        number_of_steps: usize,
        expiry: Expiry,
        volatility: f32,
        interest_rate: f32,
        dividends: f32,
    ) -> Self {
        let time_step = expiry.0 / number_of_steps as f32;
        let vol_params = VolatilityParameters::new(volatility, interest_rate, dividends, time_step);

        Self {
            stack,
            params: vol_params,
            spot: initial_price,
            expiry,
            discount_factor: (-interest_rate * time_step).exp(),
            time_step,
            _phantom_data: Default::default(),
            _phantom_data2: Default::default(),
        }
    }

    pub fn eval<T: OptionContract + Sync>(
        self,
        option: T,
    ) -> EvaluatedBinomialTreeModelImpl<Stack, V, U> {
        let p = self.params.p();

        let mut tree_map = <Stack as BinomialTreeStackImpl>::NodeNameContainerType::default();
        let truncation = U::new(
            self.spot.0,
            self.expiry.0,
            self.params.volatility,
            self.params.interest_rate,
            self.params.dividends,
        );

        let mut first_level = true;
        for (i, node_level) in self.stack.iter().enumerate().rev() {
            let current_expiry = self.expiry.0 - self.time_step * (i as f32); // Is the last step 0 or 1 timestep to expiry?

            node_level.iter().rev().enumerate().for_each(|(j, node)| {
                let up_value = tree_map.get(&node.up());
                let down_value = tree_map.get(&node.down());

                // TODO: Hide details
                let price =
                    self.spot.0 * self.params.u.powi(j as i32) * self.params.d.powi((i - j) as i32);

                //println!("{:?}{:?}", node.up(), up_value);
                //println!("{:?}{:?}", node.down(), down_value);

                match (up_value, down_value) {
                    (Some(up_value), Some(down_value)) => {
                        let up_value = up_value.get();
                        let down_value = down_value.get(); //.expect("The tree should be evaluated backwards");

                        // TODO: Hide details
                        let value = (up_value * p + down_value * (1.0 - p)) * self.discount_factor;

                        let option_value = option.value(value, price);
                        tree_map.set(node, option_value.into());
                    }
                    (Some(_up_value), None) => {
                        let option_value =
                            truncation.value(&option, 0.0, price, &self.params, current_expiry);
                        if let Some(option_value) = option_value {
                            tree_map.set(node, option_value.into());
                        }
                    }
                    (None, Some(_down_value)) => {
                        let option_value =
                            truncation.value(&option, 0.0, price, &self.params, current_expiry);
                        if let Some(option_value) = option_value {
                            tree_map.set(node, option_value.into());
                        }
                    }
                    (None, None) => {
                        if first_level {
                            let option_value =
                                V::value_at_leaf(&option, price, &self.params, current_expiry);
                            tree_map.set(node, option_value.into());
                        }
                    }
                }
            });

            first_level = false;
        }

        //println!("{:?}", tree_map);

        EvaluatedBinomialTreeModelImpl {
            model: self,
            map: tree_map,
        }
    }
}

// TODO: Move?
pub mod smoothing {
    use crate::black_scholes::black_value;
    use crate::instruments::OptionContract;
    use crate::model::VolatilityParameters;

    pub trait ValueAtLeaf {
        fn value_at_leaf<U: OptionContract + Sync>(
            option: &U,
            price: f32,
            vol_params: &VolatilityParameters,
            expiry: f32,
        ) -> f32;
    }

    impl ValueAtLeaf for None {
        fn value_at_leaf<U: OptionContract + Sync>(
            option: &U,
            price: f32,
            _vol_params: &VolatilityParameters,
            _expiry: f32,
        ) -> f32 {
            option.intrinsic_value(price)
        }
    }

    impl ValueAtLeaf for Black {
        fn value_at_leaf<U: OptionContract + Sync>(
            option: &U,
            price: f32,
            vol_params: &VolatilityParameters,
            expiry: f32,
        ) -> f32 {
            let time_to_expiry = expiry; // There is one timestep left to expiry
            let black_value = black_value(
                option.option_type(),
                price,
                option.strike(),
                vol_params.volatility,
                vol_params.interest_rate,
                vol_params.dividends,
                time_to_expiry,
            );
            option.value(black_value, price)
        }
    }

    pub struct None;
    pub struct Black;
}

// TODO: Move?
pub mod truncation {
    use crate::black_scholes::black_value;
    use crate::instruments::OptionContract;
    use crate::model::VolatilityParameters;

    pub trait ValueAtBorder {
        fn new(spot: f32, expiry: f32, volatility: f32, rate: f32, dividends: f32) -> Self;
        fn value<U: OptionContract + Sync>(
            &self,
            option: &U,
            value: f32,
            price: f32,
            vol_params: &VolatilityParameters,
            expiry: f32,
        ) -> Option<f32>;
        fn not_none() -> bool;
    }

    pub struct None;
    pub struct Black {
        price_bounds: PriceBounds,
    }

    impl ValueAtBorder for None {
        fn new(_spot: f32, _expiry: f32, _volatility: f32, _rate: f32, _dividends: f32) -> Self {
            Self {}
        }

        fn value<U: OptionContract + Sync>(
            &self,
            option: &U,
            value: f32,
            price: f32,
            _vol_params: &VolatilityParameters,
            _expiry: f32,
        ) -> Option<f32> {
            Some(option.value(value, price))
        }

        fn not_none() -> bool {
            false
        }
    }

    impl ValueAtBorder for Black {
        fn new(spot: f32, expiry: f32, volatility: f32, rate: f32, dividends: f32) -> Self {
            const NUM_OF_STD: usize = 6;
            Self {
                price_bounds: PriceBounds::new(
                    spot, expiry, volatility, rate, dividends, NUM_OF_STD,
                ),
            }
        }

        fn value<U: OptionContract + Sync>(
            &self,
            option: &U,
            _value: f32,
            price: f32,
            vol_params: &VolatilityParameters,
            current_expiry: f32,
        ) -> Option<f32> {
            if self.price_bounds.is_out_of_range(price) {
                return Option::None;
            }

            let black_value = black_value(
                option.option_type(),
                price,
                option.strike(),
                vol_params.volatility,
                vol_params.interest_rate,
                vol_params.dividends,
                current_expiry,
            );
            Some(option.value(black_value, price))
        }

        fn not_none() -> bool {
            true
        }
    }

    pub struct PriceBounds {
        lower_bound: f32,
        upper_bound: f32,
    }

    impl PriceBounds {
        /// Compute log-space and standard-space boundaries at num_std deviations under Q measure.
        fn new(
            spot: f32,
            expiry: f32,
            volatility: f32,
            rate: f32,
            dividends: f32,
            number_of_std: usize,
        ) -> PriceBounds {
            let mean_log = spot.ln() + (rate - dividends - 0.5 * volatility.powi(2) * expiry);
            let std_log = volatility * expiry.sqrt();

            let lower_log = mean_log - (number_of_std as f32) * std_log;
            let upper_log = mean_log + (number_of_std as f32) * std_log;

            let lower_price = lower_log.exp();
            let upper_price = upper_log.exp();

            PriceBounds {
                lower_bound: lower_price,
                upper_bound: upper_price,
            }
        }

        fn is_out_of_range(&self, price: f32) -> bool {
            let in_range = (self.lower_bound..=self.upper_bound).contains(&price);

            !in_range
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_calculate_price_bounds() {
            let bounds = PriceBounds::new(100.0, 0.5, 0.3, 0.05, 0.0, 6);

            assert_eq!(bounds.lower_bound, 28.78568);
            assert_eq!(bounds.upper_bound, 367.03702);

            assert!(!bounds.is_out_of_range(100.0));
            assert!(bounds.is_out_of_range(0.0));
            assert!(bounds.is_out_of_range(400.0));
            assert!(!bounds.is_out_of_range(367.03702));
        }
    }
}

#[allow(private_bounds)]
pub struct EvaluatedBinomialTreeModelImpl<
    Stack: BinomialTreeStackImpl,
    V: smoothing::ValueAtLeaf,
    U: truncation::ValueAtBorder,
> {
    model: CoxRossRubenstein<Stack, V, U>,
    map: <Stack as BinomialTreeStackImpl>::NodeNameContainerType,
}

impl<Stack: BinomialTreeStackImpl, V: smoothing::ValueAtLeaf, U: truncation::ValueAtBorder>
    fmt::Display for EvaluatedBinomialTreeModelImpl<Stack, V, U>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const GAP: usize = 8; // minimum spacing between sibling nodes

        let levels = self.model.stack.iter().len();
        if levels == 0 {
            return writeln!(f, "<empty tree>");
        }

        /* ---------- Build node labels ---------- */

        let mut node_data: Vec<Vec<(String, String)>> = Vec::with_capacity(levels);
        for (i, level) in self.model.stack.iter().enumerate() {
            let mut row = Vec::with_capacity(level.len());
            for (j, node) in level.iter().enumerate() {
                let value = self.map.get(node).unwrap().get();
                let price = self.model.spot.0
                    * self.model.params.u.powi(j as i32)
                    * self.model.params.d.powi((i - j) as i32);

                let price_str = format!("{:.2}", price);
                let value_str = format!("{:.4}", value);
                row.push((price_str, value_str));
            }
            node_data.push(row);
        }

        // Calculate max width needed for labels
        let max_price_width = node_data
            .iter()
            .flat_map(|r| r.iter())
            .map(|(p, _)| p.len() + 2) // "P: " prefix
            .max()
            .unwrap_or(6);

        let max_value_width = node_data
            .iter()
            .flat_map(|r| r.iter())
            .map(|(_, v)| v.len() + 2) // "V: " prefix
            .max()
            .unwrap_or(6);

        let node_width = max_price_width.max(max_value_width);

        /* ---------- Canvas sizing ---------- */

        let leaf_count = node_data[levels - 1].len();
        let spacing = node_width + GAP;
        let width = (leaf_count * spacing).max(80);
        // Each level needs 2 rows for data + 1 for connectors (except last level)
        let rows = levels * 3 - 1;

        let mut canvas = vec![vec![' '; width]; rows];

        /* ---------- Compute positions ---------- */

        let mut positions: Vec<Vec<usize>> = vec![Vec::new(); levels];

        // bottom level: fixed spacing
        {
            let y = levels - 1;
            let mut x = 0usize;
            for _ in 0..leaf_count {
                positions[y].push(x);
                x += spacing;
            }
        }

        // parents: midpoint of children, preserving spacing
        for level in (0..levels - 1).rev() {
            let child = positions[level + 1].clone();
            let mut parent = Vec::with_capacity(node_data[level].len());

            for j in 0..node_data[level].len() {
                if j + 1 < child.len() {
                    let left = child[j] + node_width / 2;
                    let right = child[j + 1] + node_width / 2;
                    let mid = (left + right) / 2;
                    parent.push(mid.saturating_sub(node_width / 2));
                }
            }

            positions[level] = parent;
        }

        /* ---------- Render nodes ---------- */

        for level_idx in 0..levels {
            let row_offset = level_idx * 3;
            let price_row = row_offset;
            let value_row = row_offset + 1;

            for (node_idx, (price_str, value_str)) in node_data[level_idx].iter().enumerate() {
                let x = positions[level_idx][node_idx];

                // Render price line (centered)
                let price_label = format!("P:{}", price_str);
                let padding_left = (node_width.saturating_sub(price_label.len())) / 2;
                for (k, ch) in price_label.chars().enumerate() {
                    if x + padding_left + k < width {
                        canvas[price_row][x + padding_left + k] = ch;
                    }
                }

                // Render value line (centered)
                let value_label = format!("V:{}", value_str);
                let padding_left = (node_width.saturating_sub(value_label.len())) / 2;
                for (k, ch) in value_label.chars().enumerate() {
                    if x + padding_left + k < width && value_row < rows {
                        canvas[value_row][x + padding_left + k] = ch;
                    }
                }
            }
        }

        /* ---------- Render connectors ---------- */

        for level_idx in 0..levels - 1 {
            let connector_row = (level_idx + 1) * 3 - 1;

            for j in 0..node_data[level_idx].len() {
                let p = positions[level_idx][j] + node_width / 2;
                if j + 1 >= positions[level_idx + 1].len() {
                    continue;
                }
                let l = positions[level_idx + 1][j] + node_width / 2;
                let r = positions[level_idx + 1][j + 1] + node_width / 2;

                // Horizontal line
                #[allow(clippy::needless_range_loop)]
                for x in l.min(p)..=r.max(p) {
                    if x < width && connector_row < rows && canvas[connector_row][x] == ' ' {
                        canvas[connector_row][x] = '─';
                    }
                }

                if p < width && connector_row < rows {
                    canvas[connector_row][p] = '┼';
                }
            }
        }

        // Render child connectors (check if each child has one or two parents)
        for level_idx in 0..levels - 1 {
            let connector_row = (level_idx + 1) * 3 - 1;
            let num_children = positions[level_idx + 1].len();
            let num_parents = positions[level_idx].len();

            for (child_idx, _) in positions.iter().enumerate().take(num_children) {
                let child_pos = positions[level_idx + 1][child_idx] + node_width / 2;

                // Check if this child has one or two parents
                let has_left_parent = child_idx < num_parents;
                let has_right_parent = child_idx > 0 && child_idx - 1 < num_parents;

                if child_pos < width && connector_row < rows {
                    match (has_left_parent, has_right_parent) {
                        (true, true) => {
                            // Two parents: show \/ pattern
                            canvas[connector_row][child_pos] = '\\';
                            if child_pos + 1 < width {
                                canvas[connector_row][child_pos + 1] = '/';
                            }
                        }
                        (true, false) => {
                            // Only left parent: show /
                            canvas[connector_row][child_pos] = '╱';
                        }
                        (false, true) => {
                            // Only right parent: show \
                            canvas[connector_row][child_pos] = '╲';
                        }
                        (false, false) => {
                            // No parents (shouldn't happen)
                        }
                    }
                }
            }
        }

        /* ---------- Output ---------- */

        for row in canvas {
            let line: String = row.into_iter().collect();
            writeln!(f, "{}", line.trim_end())?;
        }

        Ok(())
    }
}

#[allow(private_bounds)]
impl<Stack: BinomialTreeStackImpl, V: smoothing::ValueAtLeaf, U: truncation::ValueAtBorder>
    EvaluatedBinomialTreeModelImpl<Stack, V, U>
{
    pub fn value(&self) -> Value {
        let initial_node = <<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default();
        let value = self.map.get(&initial_node).unwrap().get();
        Value(*value)
    }

    pub fn delta(&self) -> Delta {
        self.delta_from(&<<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default())
    }

    fn delta_from(
        &self,
        from_node: &<<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType,
    ) -> Delta {
        let last_up = from_node.up();
        let last_up_value = self.map.get(&last_up).unwrap().get();
        let last_down = from_node.down();
        let last_down_value = self.map.get(&last_down).unwrap().get();
        let h = last_up.value(self.model.spot.0, self.model.params.u, self.model.params.d)
            - last_down.value(self.model.spot.0, self.model.params.u, self.model.params.d);

        if h != 0.0 {
            let delta = (last_up_value - last_down_value) / h;
            Delta(delta)
        } else {
            Delta(0.0)
        }
    }

    pub fn gamma(&self) -> Gamma {
        let initial_node = <<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default();
        let node_u = initial_node.up();
        let node_d = initial_node.down();
        let delta_u = self.delta_from(&node_u);
        let delta_d = self.delta_from(&node_d);
        let spot_u = self.map.get(&node_u).unwrap().get();
        let spot_d = self.map.get(&node_d).unwrap().get();

        if spot_u == spot_d {
            Gamma(0.0)
        } else {
            Gamma((delta_u.0 - delta_d.0) / (spot_u - spot_d))
        }
    }

    pub fn theta(&self) -> Theta {
        let initial_node = <<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default();
        let val_0 = self.map.get(&initial_node).unwrap().get();
        let val_2 = self.map.get_next_step(&initial_node).unwrap().get();

        assert_ne!(self.model.time_step, 0.0);
        Theta((val_2 - val_0) / (2.0 * self.model.time_step))
    }

    pub fn greeks(&self) -> Greeks {
        Greeks {
            value: self.value(),
            delta: self.delta(),
            gamma: self.gamma(),
            theta: self.theta(),
            // TODO: Implement vega and rho
        }
    }
}

/// Type-erased trait for evaluated binomial trees.
///
/// This trait enables runtime polymorphism for `EvaluatedBinomialTreeModelImpl`,
/// allowing the `eval_binomial_tree!` macro to return a type-erased object
/// without exposing the generic type parameters to the caller.
pub trait EvaluatedBinomialTree: fmt::Display {
    /// Get the option value at the root node
    fn value(&self) -> Value;

    /// Get the delta greek (sensitivity to spot price changes)
    fn delta(&self) -> Delta;

    /// Get the gamma greek (second derivative of option value w.r.t. spot)
    fn gamma(&self) -> Gamma;

    /// Get the theta greek (sensitivity to time decay)
    fn theta(&self) -> Theta;

    /// Get all greeks together
    fn greeks(&self) -> Greeks;

    /// Display the entire binomial tree (for debugging/visualization)
    ///
    /// This is a convenience method. You can also use the Display trait directly
    /// by calling `println!("{}", tree)` or similar.
    fn display_tree(&self) -> String {
        format!("{}", self)
    }
}

impl<Stack: BinomialTreeStackImpl, V: smoothing::ValueAtLeaf, U: truncation::ValueAtBorder>
    EvaluatedBinomialTree for EvaluatedBinomialTreeModelImpl<Stack, V, U>
{
    fn value(&self) -> Value {
        EvaluatedBinomialTreeModelImpl::value(self)
    }

    fn delta(&self) -> Delta {
        EvaluatedBinomialTreeModelImpl::delta(self)
    }

    fn gamma(&self) -> Gamma {
        EvaluatedBinomialTreeModelImpl::gamma(self)
    }

    fn theta(&self) -> Theta {
        EvaluatedBinomialTreeModelImpl::theta(self)
    }

    fn greeks(&self) -> Greeks {
        EvaluatedBinomialTreeModelImpl::greeks(self)
    }
}

pub struct Spot(pub f32);
pub struct Expiry(pub f32);

/// Type-erased evaluated binomial tree result
pub type EvaluatedTree = Box<dyn EvaluatedBinomialTree>;

/// Helper function to create a type-erased tree from any concrete implementation
/// **Note:** While this is a public function, it's primarily intended for
/// use by the `eval_binomial_tree!` and related macros. Direct usage is
/// not recommended; use the macros instead.
#[doc(hidden)] // Hide from public docs since it's for macro use
#[allow(private_bounds)]
pub fn erase_type<
    Stack: BinomialTreeStackImpl + 'static,
    V: smoothing::ValueAtLeaf + 'static,
    U: truncation::ValueAtBorder + 'static,
>(
    tree: EvaluatedBinomialTreeModelImpl<Stack, V, U>,
) -> EvaluatedTree {
    Box::new(tree)
}

#[derive(Copy, Clone)]
pub struct VolatilityParameters {
    a: f32,
    pub(crate) u: f32,
    pub(crate) d: f32,

    volatility: f32,
    interest_rate: f32,
    dividends: f32,
}

impl VolatilityParameters {
    pub fn new(
        volatility: f32,
        interest_rate: f32,
        dividends: f32,
        timestep: f32,
    ) -> VolatilityParameters {
        let u = (volatility * timestep.sqrt()).exp();
        VolatilityParameters {
            a: ((interest_rate - dividends) * timestep).exp(),
            u,
            d: 1.0 / u,
            volatility,
            interest_rate,
            dividends,
        }
    }

    pub(crate) fn p(&self) -> f32 {
        (self.a - self.d) / (self.u - self.d)
    }
}

#[derive(Debug, PartialEq)]
pub struct Greeks {
    pub value: Value,
    pub delta: Delta,
    pub gamma: Gamma,
    pub theta: Theta,
}

#[derive(Debug, PartialEq)]
pub struct Value(pub f32);

#[derive(Debug, PartialEq)]
pub struct Delta(pub f32);

#[derive(Debug, PartialEq)]
pub struct Gamma(pub f32);

#[derive(Debug, PartialEq)]
pub struct Theta(pub f32);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binomial_tree_map::r#static::StaticBinomialTreeMap;
    use crate::instruments::{AmericanOption, EuropeanOption, OptionType};
    use crate::model::smoothing::Black;
    use crate::{binomial_tree_map, eval_binomial_tree_with_steps};
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn test_binomial_tree_display_connector_patterns() {
        let tree_map = binomial_tree_map!(3);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(100.0), 3, Expiry(0.5), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Call, 95.0, 0.5);
        let eval = model.eval(option);

        let display_output = format!("{}", eval);

        let expected = r#"                        P:100.00
                        V:12.7897
                    ╱───────┼────────╲
                 P:88.47         P:113.03
                V:21.1817        V:4.4950
            ╱───────┼────────\/──────┼────────╲
         P:78.27         P:100.00         P:127.76
        V:33.5440        V:9.0022         V:0.0000
    ╱───────┼────────\/──────┼────────\/──────┼────────╲
 P:69.25          P:88.47         P:113.03         P:144.40
V:49.4009        V:18.0290        V:0.0000         V:0.0000"#;

        pretty_assertions::assert_eq!(display_output.trim_end(), expected);
    }

    #[test]
    fn test_binomial_tree_european_call() {
        let tree_map = binomial_tree_map!(2);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(100.0), 2, Expiry(0.5), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Call, 95.0, 0.5);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(12.3578));
        assert_eq!(greeks.delta(), Delta(0.6599607));
    }

    #[test]
    fn test_binomial_tree_european_call2() {
        let tree_map = binomial_tree_map!(2);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(810.0), 2, Expiry(0.5), 0.2, 0.05, 0.02);
        let option = EuropeanOption::new(OptionType::Call, 800.0, 0.5);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(53.394733));
        assert_eq!(greeks.delta(), Delta(0.5891357));
    }

    #[test]
    fn test_binomial_tree_european_call3() {
        let tree_map = binomial_tree_map!(3);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(0.61), 3, Expiry(0.25), 0.12, 0.05, 0.07);
        let option = EuropeanOption::new(OptionType::Call, 0.6, 0.25);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(0.018597357));
        assert_eq!(greeks.delta(), Delta(0.6000447));
    }

    #[test]
    fn test_binomial_tree_european_put1() {
        let tree_map = binomial_tree_map!(2);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Put, 52.0, 2.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(6.2457113));
        assert_eq!(greeks.delta(), Delta(-0.37732533));
    }

    #[test]
    fn test_binomial_tree_american_put1() {
        let tree_map = binomial_tree_map!(2);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = AmericanOption::new(OptionType::Put, 52.0, 2.0);
        let eval = model.eval(option);

        assert_eq!(
            eval.greeks(),
            Greeks {
                value: Value(7.428405),
                delta: Delta(-0.4606061),
                gamma: Gamma(-0.0), // Hmm
                theta: Theta(-2.7142024),
            }
        )
    }

    #[test]
    fn test_binomial_tree_american_put2() {
        let tree_map = binomial_tree_map!(3);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(31.0), 3, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(2.8356347));
        assert_eq!(greeks.delta(), Delta(-0.38601997));
        //assert_eq!(val.risk_free_probability, 0.4626);
    }

    #[test]
    fn test_binomial_tree_american_put3() {
        let tree_map = binomial_tree_map!(3);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(60.0), 3, Expiry(3.0 / 12.0), 0.45, 0.1, 0.00);
        let option = AmericanOption::new(OptionType::Put, 60.0, 3.0 / 12.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(5.1627836));
        assert_eq!(greeks.delta(), Delta(-0.43557432));
        //println!("{:?}", greeks.model.tree_map.map);
    }

    #[test]
    fn test_binomial_tree_american_fut_call1() {
        let tree_map = binomial_tree_map!(3);
        // Notice r = q for futs
        let model: CoxRossRubenstein<StaticBinomialTreeMap> = CoxRossRubenstein::new(
            tree_map,
            Spot(400.0),
            3,
            Expiry(9.0 / 12.0),
            0.35,
            0.06,
            0.06,
        );
        let option = AmericanOption::new(OptionType::Call, 420.0, 9.0 / 12.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(42.06769));
        assert_eq!(greeks.delta(), Delta(0.48716724));
        //println!("{:?}", greeks.model.tree_map.map);
    }

    #[test]
    fn test_binomial_tree_american_put2_100steps() {
        let tree_map = binomial_tree_map!(100);
        let model: CoxRossRubenstein<StaticBinomialTreeMap> =
            CoxRossRubenstein::new(tree_map, Spot(31.0), 100, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(2.6043036));
        assert_eq!(greeks.delta(), Delta(-0.38875455));
    }

    #[allow(dead_code)]
    // Estimate for rate of convergence
    // Senning, Jonathan R. "Computing and Estimating the Rate of Convergence https://www.math-cs.gordon.edu/courses/ma342/handouts/rate.pdf
    fn rate_of_convergence(errors: [f32; 4]) -> f32 {
        ((errors[3] - errors[2]) / (errors[2] - errors[1]))
            .abs()
            .ln()
            / ((errors[2] - errors[1]) / (errors[1] - errors[0]))
                .abs()
                .ln()
    }

    // Mark S. Joshi, "The Convergence of Binomial Trees For Pricing the American Put"
    // https://fbe.unimelb.edu.au/__data/assets/pdf_file/0010/2591884/170.pdf
    fn relative_error(tree_price: f32, true_price: f32, intrinsic_value: f32) -> f32 {
        (tree_price - true_price) / (0.5 + true_price - intrinsic_value)
    }

    fn eval_american_option_example(steps: usize) -> f32 {
        if steps > 128 {
            let tree_map = crate::binomial_tree_map::dynamic::DynamicBinomialTreeMap::new(steps);
            let binom_tree: CoxRossRubenstein<
                crate::binomial_tree_map::dynamic::DynamicBinomialTreeMap,
                Black,
            > = CoxRossRubenstein::new(tree_map, Spot(100.0), steps, Expiry(0.5), 0.3, 0.05, 0.0);

            binom_tree
                .eval(AmericanOption::new(OptionType::Call, 95.0, 0.5))
                .value()
                .0
        } else {
            eval_binomial_tree_with_steps!(
                steps,
                AmericanOption,
                Call,
                95.0,
                100.0,
                0.5,
                0.3,
                0.05,
                0.0
            )
            .value()
            .0
        }
    }

    fn eval_and_calculate_relative_error(steps: usize) -> f32 {
        let true_value = 12.32791655;

        let val = eval_american_option_example(steps);

        let option = AmericanOption::new(OptionType::Call, 95.0, 0.5);
        let intrinsic = option.intrinsic_value(100.0);

        relative_error(val, true_value, intrinsic)
    }

    #[ignore = "convergence check"]
    #[test]
    fn test_binomial_tree_american_call_convergence() {
        /*assert_eq!(eval_and_calculate_relative_error(11), -0.005980755);
        assert_eq!(eval_and_calculate_relative_error(51), 0.0036523752);
        assert_eq!(eval_and_calculate_relative_error(71), 0.0031288671);
        assert_eq!(eval_and_calculate_relative_error(101), 0.0013165652);
        assert_eq!(eval_and_calculate_relative_error(113), 0.0005097442);
        assert_eq!(eval_and_calculate_relative_error(127), -0.0003817296);
        assert_eq!(eval_and_calculate_relative_error(1001), 0.00057088915);*/

        for i in vec![11, 51, 71, 101, 113, 127, 1001] {
            println!("{}: {}", i, eval_and_calculate_relative_error(i))
        }
    }

    #[test]
    fn test_binomial_tree_american_call_convergence2() {
        for i in 1..128 {
            println!("{}", eval_american_option_example(i))
        }
    }
}
