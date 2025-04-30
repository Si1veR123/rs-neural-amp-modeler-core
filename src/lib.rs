use std::{ffi::c_void, ptr::null_mut};

pub mod bindings;

pub const DEFAULT_BUFFER_SIZE: usize = 512;

pub struct NeuralAmpModeler {
    pub(crate) model_path: Option<String>,
    pub(crate) model: *mut bindings::nam_DSP,
    pub(crate) buffer: Vec<f32>,
    pub(crate) maximum_buffer_size: usize,
}
 
impl NeuralAmpModeler {
    /// Default maximum buffer size is 512 samples.
    pub fn new() -> Result<NeuralAmpModeler, String> {
        // Used in NeuralAmpModelerPlugin so....
        unsafe { bindings::nam_activations_Activation_enable_fast_tanh(); }

        Ok(NeuralAmpModeler {
            model_path: None,
            model: null_mut(),
            buffer: vec![0.0; DEFAULT_BUFFER_SIZE], // default, updated when processing if needed
            maximum_buffer_size: DEFAULT_BUFFER_SIZE
        })
    }

    pub fn new_with_maximum_buffer_size(maximum_buffer_size: usize) -> Result<NeuralAmpModeler, String> {
        unsafe { bindings::nam_activations_Activation_enable_fast_tanh(); }

        Ok(NeuralAmpModeler {
            model_path: None,
            model: null_mut(),
            buffer: vec![0.0; maximum_buffer_size],
            maximum_buffer_size
        })
    }

    pub fn get_model_path(&self) -> Option<&str> {
        self.model_path.as_ref().map(|s| s.as_str())
    }

    pub fn set_model(&mut self, model_path: &str) -> Result<(), String> {
        let sample_rate = self.expected_sample_rate() as usize;

        let model_path_c = std::ffi::CString::new(model_path).unwrap();
        let model = unsafe { bindings::get_dsp_from_string_path(model_path_c.as_ptr()) };

        if model.is_null() {
            return Err("Failed to load model".to_string());
        }

        let model_ptr = model as *mut bindings::nam_DSP;
        
        self.model_path = Some(model_path.to_string());
        if !self.model.is_null() {
            unsafe { bindings::destroy_dsp(self.model) };
        }
        self.model = model_ptr;

        self.reset_and_prewarm_model(sample_rate, self.maximum_buffer_size);

        Ok(())
    }

    pub fn get_maximum_buffer_size(&self) -> usize {
        self.maximum_buffer_size
    }

    pub fn set_maximum_buffer_size(&mut self, maximum_buffer_size: usize) {
        if self.model.is_null() {
            return;
        }

        if maximum_buffer_size > self.maximum_buffer_size {
            self.maximum_buffer_size = maximum_buffer_size;
            self.reset_model(self.expected_sample_rate(), self.maximum_buffer_size);
            self.buffer.resize(maximum_buffer_size, 0.0);
        }
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        if self.model.is_null() {
            return;
        }

        if buffer.len() > self.maximum_buffer_size {
            self.set_maximum_buffer_size(buffer.len());
        }

        println!("Sending pointers: {:?} {:?} {:?}", self.model, buffer.as_mut_ptr(), self.buffer.as_mut_ptr());
        unsafe { bindings::dsp_process(self.model, buffer.as_mut_ptr(), self.buffer.as_mut_ptr(), buffer.len() as i32) };
        
        buffer.copy_from_slice(&self.buffer[..buffer.len()]);
    }

    pub fn expected_sample_rate(&self) -> usize {
        if self.model.is_null() {
            return 0;
        }

        unsafe { bindings::get_dsp_expected_sample_rate(self.model) as usize }
    }

    pub fn reset_model(&mut self, sample_rate: usize, buffer_size: usize) {
        if self.model.is_null() {
            return;
        }

        unsafe { bindings::nam_DSP_Reset(self.model as *mut c_void, sample_rate as f64, buffer_size as i32) };
    }

    pub fn prewarm_model(&mut self) {
        if self.model.is_null() {
            return;
        }

        unsafe { bindings::nam_DSP_prewarm(self.model as *mut c_void) };
    }

    pub fn reset_and_prewarm_model(&mut self, sample_rate: usize, buffer_size: usize) {
        self.reset_model(sample_rate, buffer_size);
        self.prewarm_model();
    }
}

impl Drop for NeuralAmpModeler {
    fn drop(&mut self) {
        if !self.model.is_null() {
            unsafe { bindings::destroy_dsp(self.model) };
            self.model = null_mut();
        }
    }
}

// Pointer to heap can be shared between threads
unsafe impl Send for NeuralAmpModeler {}
unsafe impl Sync for NeuralAmpModeler {}
