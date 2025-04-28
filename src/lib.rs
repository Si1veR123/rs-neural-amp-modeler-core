use std::{ffi::c_void, ptr::null_mut};

pub mod bindings;

const DEFAULT_BUFFER_SIZE: usize = 512;

pub struct NeuralAmpModeler {
    model_path: Option<String>,
    model: *mut bindings::nam_DSP,
    buffer: Vec<f32>,
    maximum_buffer_size: usize
}

impl NeuralAmpModeler {
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

    pub fn get_maximum_buffer_size(&self) -> usize {
        self.maximum_buffer_size
    }

    pub fn set_model(&mut self, model_path: &str) -> Result<(), String> {
        let model_path_c = std::ffi::CString::new(model_path).unwrap();
        let model = unsafe { bindings::get_dsp_from_string_path(model_path_c.as_ptr()) };
        let model_ptr = model as *mut bindings::nam_DSP;
        if model_ptr.is_null() {
            return Err("Failed to load model".to_string());
        }
        self.model_path = Some(model_path.to_string());
        self.model = model_ptr;

        self.reset_and_prewarm_model();

        Ok(())
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        if self.model.is_null() {
            return;
        }

        if buffer.len() > self.maximum_buffer_size {
            self.maximum_buffer_size = buffer.len();
            self.reset_and_prewarm_model();
        }

        self.buffer.resize(buffer.len(), 0.0);

        unsafe { bindings::dsp_process(self.model, buffer.as_mut_ptr(), self.buffer.as_mut_ptr(), buffer.len() as i32) };
        
        buffer.copy_from_slice(&self.buffer[..buffer.len()]);
    }

    pub fn expected_sample_rate(&self) -> f64 {
        if self.model.is_null() {
            return 0.0;
        }

        unsafe { bindings::get_dsp_expected_sample_rate(self.model) }
    }

    pub fn reset_and_prewarm_model(&mut self) {
        if self.model.is_null() {
            return;
        }

        unsafe {
            bindings::nam_DSP_Reset(self.model as *mut c_void, self.expected_sample_rate(), self.maximum_buffer_size as i32);
            bindings::nam_DSP_prewarm(self.model as *mut c_void);
        };
    }
}

// Pointer to heap can be shared between threads
unsafe impl Send for NeuralAmpModeler {}
unsafe impl Sync for NeuralAmpModeler {}
