// This is the skeleton for the CUDA implementation

use crate::cnn::*;
use rustacuda::function::{BlockSize, GridSize};
use rustacuda::launch;
use rustacuda::memory::DeviceBox;
use rustacuda::prelude::*;
use std::error::Error;
use std::ffi::CString;

// Fields need to be ordered this way so the DeviceBoxes are
// dropped before the Context. Otherwise the drop will panic.

// Constants for kernel execution configuration.
const NUM_THREADS: usize = 16;
const CONV_NUM_BLOCKS: u32 = 10;
const CONV_NUM_THREADS: u32 = 400; // Adjust to CONV_OUT_DIM*CONV_OUT_DIM if needed.
const OUT_NUM_BLOCKS: u32 = 1;
const KERNEL_SHARED_MEM_SIZE: u32 = 0;
// Use for debugging
const DEBUG_LOGS: bool = true;

pub struct CudaContext {
    conv_layer: DeviceBox<ConvLayer>,
    output_layer: DeviceBox<OutputLayer>,
    module: Module,
    stream: Stream,
    _context: Context,
}

impl CudaContext {
    pub fn init(cnn: &Cnn) -> Result<Self, Box<dyn Error>> {
        rustacuda::init(CudaFlags::empty())?;

        let device = Device::get_device(0)?;
        if DEBUG_LOGS {
            println!("Using CUDA device: {}", device.name()?);
        }
        let context =
            Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;

        let ptx_str = include_str!("../kernel/kernel.ptx");
        let ptx = CString::new(ptx_str)?;
        let module = Module::load_from_string(&ptx)?;

        let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

        let conv_layer = DeviceBox::new(&cnn.conv_layer)?;
        let output_layer = DeviceBox::new(&cnn.output_layer)?;

        Ok(CudaContext {
            conv_layer,
            output_layer,
            module,
            stream,
            _context: context,
        })
    }

    pub fn compute(&mut self, input: &InputMatrix) -> Result<OutputVec, Box<dyn Error>> {
        let module = &self.module;
        let stream = &self.stream;
        let mut conv_input = DeviceBox::new(input)?;
        let mut conv_output =
            DeviceBox::new(&[[[0.0f64; CONV_OUT_DIM]; CONV_OUT_DIM]; CONV_LAYER_SIZE])?;
        let mut relu_output =
            DeviceBox::new(&[[[0.0f64; CONV_OUT_DIM]; CONV_OUT_DIM]; CONV_LAYER_SIZE])?;
        let mut out_host = vec![0.0f64; OUT_LAYER_SIZE * NUM_THREADS];
        let mut out_output = DeviceBuffer::from_slice(&out_host)?;

        let conv_grid_size = GridSize::x(OUT_LAYER_SIZE as u32);
        let conv_block_size = BlockSize::xy(CONV_OUT_DIM as u32, CONV_OUT_DIM as u32);
        let relu_grid_size = GridSize::x(OUT_LAYER_SIZE as u32);
        let relu_block_size = BlockSize::xy(CONV_OUT_DIM as u32, CONV_OUT_DIM as u32);
        let output_grid_size = GridSize::x(OUT_LAYER_SIZE as u32);
        let output_block_size = BlockSize::x(NUM_THREADS as u32);

        unsafe {
            let _ = launch!(module.convolution_layer<<<conv_grid_size, conv_block_size, 0, stream>>>(
                conv_input.as_device_ptr(),
                self.conv_layer.as_device_ptr(),
                conv_output.as_device_ptr()
            ));

            let _ = launch!(module.relu_layer<<<relu_grid_size, relu_block_size, 0, stream>>>(
                conv_output.as_device_ptr(),
                relu_output.as_device_ptr()
            ));

            let _ = launch!(module.output_layer<<<output_grid_size, output_block_size, 0, stream>>>(
                relu_output.as_device_ptr(),
                self.output_layer.as_device_ptr(),
                out_output.as_device_ptr()
            ));
        }

        stream.synchronize()?;

        let mut res = OutputVec([0.0f64; OUT_LAYER_SIZE]);
        out_output.copy_to(&mut out_host)?;
        for i in 0..OUT_LAYER_SIZE {
            for j in 0..NUM_THREADS {
                res.0[i] += out_host[i * NUM_THREADS + j];
            }
        }

        Ok(res)
    }
}
