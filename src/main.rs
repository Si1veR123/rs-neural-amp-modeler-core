use neural_amp_modeler::NeuralAmpModeler;

fn sine_wave(frequency: f32, sample_rate: f32, sample_length: usize, amplitude: f32) -> Vec<f32> {
    let mut buffer = vec![0.0; sample_length];
    for i in 0..sample_length {
        buffer[i] = (2.0 * std::f32::consts::PI * frequency * (i as f32 / sample_rate)).sin() * amplitude;
    }
    buffer
}

fn main() {
    let mut modeler = NeuralAmpModeler::new().unwrap();
    
    modeler.set_model(r"C:\Users\conno\Downloads\Fender Super Reverb 1977\Fender Super Reverb_ EQ Flat, Volume 3, sm57 and AKG 414.nam").expect("Failed to set model");
    let sine = sine_wave(440.0, 48000.0, 5120000, 10.0);
    assert_eq!(modeler.get_maximum_buffer_size(), 512);
    println!("Expected Sample Rate: {}", modeler.expected_sample_rate());

    for frame in 0..10000 {
        let mut buffer = sine[frame * 512..(frame + 1) * 512].to_vec();
        modeler.process_buffer(&mut buffer);
        println!("{:?}", buffer);
    }
}
