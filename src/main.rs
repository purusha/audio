//mod stt;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Local;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let date = Local::now();
    let formatted_date = date.format("%d-%m-%Y").to_string(); // Formato gg-mm-yyyy
    let file_name = format!("{}.wav", formatted_date);

    let host = cpal::default_host();
    let device = host.default_input_device().expect("Nessun microfono trovato");
    let config = device.default_input_config()?;
    println!("Registrazione per 5 secondi...");

    let spec = WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = Arc::new(Mutex::new(WavWriter::new(
        BufWriter::new(File::create(&file_name)?),
        spec,
    )?));

    let writer_clone = writer.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[f32], _| write_input_f32(data, &writer_clone),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[i16], _| write_input_i16(data, &writer_clone),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[u16], _| write_input_u16(data, &writer_clone),
            err_fn,
            None,
        )?,
        _ => todo!()
    };

    stream.play()?;
    std::thread::sleep(Duration::from_secs(5));
    drop(stream);

    if let Ok(writer) = Arc::try_unwrap(writer) {
        let writer = writer.into_inner().unwrap();
        writer.finalize()?;
    } else {
        eprintln!("Errore nel chiudere il writer");
    }

    println!("Registrazione terminata. File salvato: {}", &file_name);
    Ok(())
}

fn write_input_f32(input: &[f32], writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>) {
    let mut writer = writer.lock().unwrap();
    for &sample in input {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16).unwrap();
    }
}

fn write_input_i16(input: &[i16], writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>) {
    let mut writer = writer.lock().unwrap();
    for &sample in input {
        writer.write_sample(sample).unwrap();
    }
}

fn write_input_u16(input: &[u16], writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>) {
    let mut writer = writer.lock().unwrap();
    for &sample in input {
        // Conversione da unsigned a signed
        let sample_i16 = (sample as i32 - i16::MAX as i32) as i16;
        writer.write_sample(sample_i16).unwrap();
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Errore nello stream: {}", err);
}
