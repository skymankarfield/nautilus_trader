// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2024 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::fmt::{Debug, Display};

use anyhow::Result;
use nautilus_model::data::{bar::Bar, quote::QuoteTick, trade::TradeTick};
use pyo3::prelude::*;

use crate::{
    average::{MovingAverageFactory, MovingAverageType},
    indicator::{Indicator, MovingAverage},
};

/// An indicator which calculates a Average True Range (ATR) across a rolling window.
#[repr(C)]
#[derive(Debug)]
#[pyclass(module = "nautilus_trader.core.nautilus_pyo3.indicators")]
pub struct AverageTrueRange {
    pub period: usize,
    pub ma_type: MovingAverageType,
    pub use_previous: bool,
    pub value_floor: f64,
    pub value: f64,
    pub count: usize,
    pub is_initialized: bool,
    has_inputs: bool,
    _previous_close: f64,
    _ma: Box<dyn MovingAverage + Send + 'static>,
}

impl Display for AverageTrueRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({},{},{},{})",
            self.name(),
            self.period,
            self.ma_type,
            self.use_previous,
            self.value_floor,
        )
    }
}

impl Indicator for AverageTrueRange {
    fn name(&self) -> String {
        stringify!(AverageTrueRange).to_string()
    }

    fn has_inputs(&self) -> bool {
        self.has_inputs
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn handle_quote_tick(&mut self, _tick: &QuoteTick) {
        // Function body intentionally left blank.
    }

    fn handle_trade_tick(&mut self, _tick: &TradeTick) {
        // Function body intentionally left blank.
    }

    fn handle_bar(&mut self, bar: &Bar) {
        self.update_raw((&bar.high).into(), (&bar.low).into(), (&bar.close).into());
    }

    fn reset(&mut self) {
        self._previous_close = 0.0;
        self.value = 0.0;
        self.count = 0;
        self.has_inputs = false;
        self.is_initialized = false;
    }
}

impl AverageTrueRange {
    pub fn new(
        period: usize,
        ma_type: Option<MovingAverageType>,
        use_previous: Option<bool>,
        value_floor: Option<f64>,
    ) -> Result<Self> {
        Ok(Self {
            period,
            ma_type: ma_type.unwrap_or(MovingAverageType::Simple),
            use_previous: use_previous.unwrap_or(true),
            value_floor: value_floor.unwrap_or(0.0),
            value: 0.0,
            count: 0,
            _previous_close: 0.0,
            _ma: MovingAverageFactory::create(MovingAverageType::Simple, period),
            has_inputs: false,
            is_initialized: false,
        })
    }

    pub fn update_raw(&mut self, high: f64, low: f64, close: f64) {
        if self.use_previous {
            if !self.has_inputs {
                self._previous_close = close;
            }
            self._ma.update_raw(
                f64::max(self._previous_close, high) - f64::min(low, self._previous_close),
            );
            self._previous_close = close;
        } else {
            self._ma.update_raw(high - low);
        }

        self._floor_value();
        self.increment_count();
    }

    fn _floor_value(&mut self) {
        if self.value_floor == 0.0 || self.value_floor < self._ma.value() {
            self.value = self._ma.value();
        } else {
            // Floor the value
            self.value = self.value_floor;
        }
    }

    fn increment_count(&mut self) {
        self.count += 1;

        if !self.is_initialized {
            self.has_inputs = true;
            if self.count >= self.period {
                self.is_initialized = true;
            }
        }
    }
}