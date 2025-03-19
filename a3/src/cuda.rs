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

const NUM_THREADS: usize = 16;
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
        // Setup device buffers and CUDA execution configuration.
        let kernel_module = &self.module;
        let cuda_stream = &self.stream;

        let mut d_input = DeviceBox::new(input)?;
        let mut d_conv_out =
            DeviceBox::new(&[[[0.0f64; CONV_OUT_DIM]; CONV_OUT_DIM]; CONV_LAYER_SIZE])?;
        let mut d_relu_out =
            DeviceBox::new(&[[[0.0f64; CONV_OUT_DIM]; CONV_OUT_DIM]; CONV_LAYER_SIZE])?;
        let mut host_buffer = vec![0.0f64; OUT_LAYER_SIZE * NUM_THREADS];
        let mut d_output_buf = DeviceBuffer::from_slice(&host_buffer)?;

        // Define grid and block dimensions.
        let grid_conv = GridSize::x(OUT_LAYER_SIZE as u32);
        let block_conv = BlockSize::xy(CONV_OUT_DIM as u32, CONV_OUT_DIM as u32);
        let grid_relu = GridSize::x(OUT_LAYER_SIZE as u32);
        let block_relu = BlockSize::xy(CONV_OUT_DIM as u32, CONV_OUT_DIM as u32);
        let grid_output = GridSize::x(OUT_LAYER_SIZE as u32);
        let block_output = BlockSize::x(NUM_THREADS as u32);

        // Launch CUDA kernels to process the input.
        unsafe {
            let _ = launch!(kernel_module.convolution_layer<<<grid_conv, block_conv, 0, cuda_stream>>>(
                d_input.as_device_ptr(),
                self.conv_layer.as_device_ptr(),
                d_conv_out.as_device_ptr()
            ));

            let _ = launch!(kernel_module.relu_layer<<<grid_relu, block_relu, 0, cuda_stream>>>(
                d_conv_out.as_device_ptr(),
                d_relu_out.as_device_ptr()
            ));

            let _ = launch!(kernel_module.output_layer<<<grid_output, block_output, 0, cuda_stream>>>(
                d_relu_out.as_device_ptr(),
                self.output_layer.as_device_ptr(),
                d_output_buf.as_device_ptr()
            ));
        }

        // Wait until the kernels have completed.
        cuda_stream.synchronize()?;

        // Retrieve output data from device and reduce per-thread results.
        d_output_buf.copy_to(&mut host_buffer)?;
        let mut final_output = OutputVec([0.0f64; OUT_LAYER_SIZE]);
        for (i, chunk) in host_buffer.chunks(NUM_THREADS).enumerate() {
            final_output.0[i] = chunk.iter().sum();
        }

        Ok(final_output)
    }
}
