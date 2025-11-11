use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("no input device");

    println!("Default input device: {}", device.name().unwrap_or("Unknown".to_string()));
    println!("\nSupported configs:");

    let configs = device.supported_input_configs().expect("error querying configs");
    for config in configs {
        println!("  Channels: {}, Sample Rate: {:?}, Format: {:?}",
                 config.channels(),
                 config.min_sample_rate().0..=config.max_sample_rate().0,
                 config.sample_format());
    }

    println!("\nDefault config:");
    let default = device.default_input_config().expect("no default config");
    println!("  Channels: {}, Sample Rate: {}, Format: {:?}",
             default.channels(),
             default.sample_rate().0,
             default.sample_format());
}