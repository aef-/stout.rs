use plotters::prelude::*;
use super::stock::Stock;


pub fn build_chart(stock: Stock) -> Result<(), Box<dyn std::error::Error>>  {
    let root = BitMapBackend::new("0.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(stock.prices[0].date as f32..stock.prices.last().unwrap().date as f32, stock.low as f32..stock.high as f32)?;
        chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            stock.prices.iter().map(|x| (x.date as f32, x.close as f32)),
            &RED,
        ))?
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    Ok(())
}
