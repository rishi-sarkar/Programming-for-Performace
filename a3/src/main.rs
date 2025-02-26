// Starter code for ECE 459 Lab 3, Winter 2021

// You should not need to modify this file.

#![warn(clippy::all)]
mod cnn;
mod cpu;
mod cuda;

use cnn::*;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

fn convert_row(record: csv::StringRecord, len: usize) -> Result<Vec<f64>, Box<dyn Error>> {
    let row = record
        .iter()
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()?;
    if row.len() != len {
        Err("mismatched row length".into())
    } else {
        Ok(row)
    }
}

// The input matrix file contains N input matrices, where N is the first line of the file. Each
// matrix consists of 100 lines of 100 floats. This function reads each input matrix in the file
// and passes it into a user-provided function to process. Provided to students.
fn for_each_input<F: FnMut(&InputMatrix) -> Result<OutputVec, Box<dyn Error>>>(
    csv_path: &Path,
    out_path: &Path,
    mut f: F,
) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(csv_path)?;
    let mut records = rdr.records();
    let mut writer = csv::Writer::from_path(out_path)?;

    // read the first line and parse the number of input matrices
    let size = records
        .next()
        .ok_or("empty file")??
        .get(0)
        .ok_or("size not found")?
        .parse::<u32>()? as usize;

    for _ in 0..size {  // read the matrices
        let mut input = [[0.0; INPUT_DIM]; INPUT_DIM];
        // 100x100 input matrix (100 lines of 100)
        for row in input.iter_mut() {
            let record = records.next().ok_or("Input matrix incomplete")??;
            let line = convert_row(record, INPUT_DIM)?;
            row.copy_from_slice(&line);
        }
        let output = f(&InputMatrix(input))?;  // invoke the user's function
        // Write output vector to output file
        writer.write_record(output.0.iter().map(|i| i.to_string()))?;
    }
    Ok(())
}

// CNN file format is 10 lines of 25 numbers (convolution filters) followed by 10 lines of 4000
// (output weights).
fn read_cnn(csv_path: &Path) -> Result<Box<Cnn>, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(csv_path)?;
    let mut records = rdr.records();

    let mut cnn = Box::new(Cnn {
        conv_layer: ConvLayer([[[0.0; FILTER_DIM]; FILTER_DIM]; CONV_LAYER_SIZE]),
        output_layer: OutputLayer([[0.0; OUT_NEURON_DIM]; OUT_LAYER_SIZE]),
    });

    // Convolution layer
    for i in 0..CONV_LAYER_SIZE {
        let record = records.next().ok_or("Convolution filters incomplete")??;
        let filter = convert_row(record, FILTER_DIM * FILTER_DIM)?;
        cnn.conv_layer.0[i]
            .iter_mut()
            .zip(filter.chunks(FILTER_DIM))
            .for_each(|(row, f)| row.copy_from_slice(&f));
    }

    // Output layer
    for i in 0..OUT_LAYER_SIZE {
        let record = records.next().ok_or("Output layer incomplete")??;
        let row = convert_row(record, OUT_NEURON_DIM)?;
        cnn.output_layer.0[i].copy_from_slice(&row);
    }
    Ok(cnn)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: [cpu|cuda] cnn_file input_file output_file");
        return Ok(());
    }
    let use_cuda = match &*args[1] {
        "cpu" => false,
        "cuda" => true,
        _ => {
            eprintln!("Specify either cpu or cuda");
            return Ok(());
        }
    };
    let cnn_file = &args[2];
    let input_file = &args[3];
    let output_file = &args[4];

    let mut work_time = 0;
    let cnn = read_cnn(&PathBuf::from(cnn_file))?;

    // initialize CUDA
    let mut ctx = if use_cuda {
        Some(cuda::CudaContext::init(&cnn)?)
    } else {
        None
    };

    for_each_input(  // for each input matrix in the file
        &PathBuf::from(input_file),
        &PathBuf::from(output_file),
        |input| {  // this closure gets run on each matrix
            let now = std::time::Instant::now();  // start time

            let output = if use_cuda {
                ctx.as_mut().unwrap().compute(&input)?
            } else {
                cpu::compute(&input, &cnn)
            };

            work_time += now.elapsed().as_micros();  // end time
            Ok(output)
        },
    )?;

    println!("{} microseconds of actual work done", work_time);
    Ok(())
}
