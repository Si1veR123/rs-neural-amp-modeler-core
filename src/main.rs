use neural_amp_modeler::NeuralAmpModeler;

fn main() {
    let modeler = NeuralAmpModeler::new().unwrap();
    assert_eq!(modeler.get_maximum_buffer_size(), 512);
}
