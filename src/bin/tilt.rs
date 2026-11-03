use anyhow::{bail, Context};
use kinect_firmware_utils::{
    get_motion_state, open_audio_device, set_tilt_degrees, Tilt, KINECT_AUDIO_INTERFACE,
};

fn main() -> anyhow::Result<()> {
    const USAGE: &str = "Usage: tilt [<degrees (-31..31)>]";
    let mut args = std::env::args_os().skip(1);
    let argument = args.next();
    if args.next().is_some() {
        bail!(USAGE);
    }

    let tilt = if let Some(argument) = argument {
        let degrees = argument
            .to_str()
            .and_then(|value| value.parse().ok())
            .context(USAGE)?;
        Some(Tilt::new(degrees).context(USAGE)?)
    } else {
        None
    };

    // Kinect for Windows audio product ID after firmware upload.
    let device = open_audio_device(0x02c3)?;
    device.claim_interface(KINECT_AUDIO_INTERFACE)?;
    match tilt {
        Some(tilt) => set_tilt_degrees(&device, 1, tilt)?,
        None => println!("{}", get_motion_state(&device, 1)?.tilt_degrees),
    }

    Ok(())
}
