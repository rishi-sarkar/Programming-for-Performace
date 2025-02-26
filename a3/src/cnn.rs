// Part of the starter code. Defines constants and data structures.

// You do not need to modify this file.

use rustacuda::memory::DeviceCopy;

// The CNN consists of 100x100 input matrix, a convolution layer of 10 5x5 filter matrices, a RELU
// layer, and an output layer of 10 4000x1 weight vectors. CNN output is a 10x1 vector.
pub const INPUT_DIM: usize = 100;
pub const FILTER_DIM: usize = 5; // should be factor of INPUT_DIM
pub const CONV_OUT_DIM: usize = INPUT_DIM / FILTER_DIM;
pub const CONV_LAYER_SIZE: usize = 10;
pub const OUT_NEURON_DIM: usize = CONV_OUT_DIM * CONV_OUT_DIM * CONV_LAYER_SIZE;
pub const OUT_LAYER_SIZE: usize = 10;

// Use repr(transparent) so the types have same memory layout as arrays

#[repr(transparent)]
pub struct InputMatrix(pub [[f64; INPUT_DIM]; INPUT_DIM]);
#[repr(transparent)]
pub struct ConvLayer(pub [[[f64; FILTER_DIM]; FILTER_DIM]; CONV_LAYER_SIZE]);
#[repr(transparent)]
// Each convolution filter generates a 20x20 output filter matrix, and there are 10 in total.
pub struct ConvOutput(pub [[[f64; CONV_OUT_DIM]; CONV_OUT_DIM]; CONV_LAYER_SIZE]);
#[repr(transparent)]
pub struct OutputLayer(pub [[f64; OUT_NEURON_DIM]; OUT_LAYER_SIZE]);
#[repr(transparent)]
// Each of the 10 output layer neurons generate 1 number
pub struct OutputVec(pub [f64; OUT_LAYER_SIZE]);

pub struct Cnn {
    pub conv_layer: ConvLayer,
    pub output_layer: OutputLayer,
}

// Allows DeviceBox::new() to be directly called on these types
// Safe as long as the type can be directly memcopied (shallow copied)
// Since Rust arrays are guaranteed to have row-major memory layout, this trait is safe to
// implement and we can rely on the ordering of elements inside CUDA code.
unsafe impl DeviceCopy for InputMatrix {}
unsafe impl DeviceCopy for ConvLayer {}
unsafe impl DeviceCopy for ConvOutput {}
unsafe impl DeviceCopy for OutputLayer {}
unsafe impl DeviceCopy for OutputVec {}
