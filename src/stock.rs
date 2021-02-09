use std::io;
use tokio::task;
use crate::common::{chart_data_to_prices, Price, TimeFrame};
use crate::api::model::ChartMeta;

#[derive(Debug)]
pub struct Stock {
    pub high: f64,
    pub low: f64,
    pub symbol: String,
    pub prices: Vec<Price>,
    pub current_price: f64,
}

impl Stock {
    pub async fn new(symbol: &str) -> Stock {


        let (current_regular_price, current_post_price, volume) = fetch_current_price(symbol.to_string()).await.unwrap().unwrap();
        let (_time_frame, _chartdata, prices) = fetch_symbol_data(symbol.to_string()).await.unwrap().unwrap();
        let current_price = get_current_price(current_regular_price, current_post_price);
        let (high, low) = get_high_low(current_price, &prices);

        Stock { 
            symbol: symbol.to_string(),
            prices, 
            high,
            low,
            current_price,
        }
    }
}

fn remove_zeros_lows(prices: Vec<Price>) -> Vec<Price> {
    prices.into_iter().filter(|x| x.low.ne(&0.0)).collect()
}

fn fetch_current_price(symbol: String) -> task::JoinHandle<Result<(f64, Option<f64>, String), io::Error>> {
    task::spawn(async move {
        /*if let Ok(response) = crate::CLIENT
          .get_chart_data(&symbol, interval, time_frame.as_range(), include_pre_post)
          .await;
          */
        let time_frame = TimeFrame::Day1;
        let response = crate::CLIENT
            .get_company_data(&symbol)
            .await;

        match response {  
            Ok(payload) => Ok((
                    payload.price.regular_market_price.price,
                    payload.price.post_market_price.price,
                    payload.price.regular_market_volume.fmt,
            )),
            Err(err) =>{
                Err(io::Error::new(io::ErrorKind::Other, err))
            }
        }
    })
}

fn fetch_symbol_data(symbol: String) -> task::JoinHandle<Result<(TimeFrame, ChartMeta, Vec<Price>), io::Error>> {
    task::spawn(async move {
        /*if let Ok(response) = crate::CLIENT
          .get_chart_data(&symbol, interval, time_frame.as_range(), include_pre_post)
          .await;
          */
        let time_frame = TimeFrame::Day1;
        let response = crate::CLIENT
            .get_chart_data(&symbol, time_frame.api_interval(), time_frame.as_range(), true)
            .await;

        match response {  
            Ok(payload) => Ok((
                    time_frame,
                    payload.meta.clone(),
                    chart_data_to_prices(payload),
            )),
            Err(err) =>{
                Err(io::Error::new(io::ErrorKind::Other, err))
            }
        }
    })
}

pub fn get_current_price(current_regular_price: f64, current_post_price: Option<f64>) -> f64 {
    if current_post_price.is_some() {
        current_post_price
            .unwrap_or(current_regular_price)
    } else {
        current_regular_price
    }
}

pub fn get_high_low(current_price: f64, data: &[Price]) -> (f64, f64) {
    let mut data = data.to_vec();

    data.sort_by(|a, b| a.high.partial_cmp(&b.high).unwrap());
    let mut max = data.last().map(|d| d.high).unwrap_or(0.0);

    data = remove_zeros_lows(data);
    data.sort_by(|a, b| a.low.partial_cmp(&b.low).unwrap());
    let mut min = data.first().map(|d| d.low).unwrap_or(0.0);

    if current_price.le(&min) {
        min = current_price;
    }

    if current_price.gt(&max) {
        max = current_price;
    }

    (max, min)
}
