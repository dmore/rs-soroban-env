// Run this with
// $ cargo bench --features wasmi,testutils --bench variation_histograms -- --nocapture
mod common;
use common::*;

struct LinearModelTables;
impl Benchmark for LinearModelTables {
    fn bench<HCM: HostCostMeasurement>() -> std::io::Result<(FPCostModel, FPCostModel)> {
        let mut measurements = measure_cost_variation::<HCM>(100)?;
        measurements.preprocess();
        measurements.report_histogram("cpu", |m| m.cpu_insns);
        measurements.report_histogram("mem", |m| m.mem_bytes);
        Ok(Default::default())
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
fn main() -> std::io::Result<()> {
    for_each_host_cost_measurement::<LinearModelTables>()?;
    Ok(())
}
