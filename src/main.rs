use ac_ffmpeg::codec::video::{VideoFrame, VideoFrameMut};
use clap::{Arg, Command};
use std::time::Duration;

fn main() {
	let matches = Command::new("encoding")
		.arg(
			Arg::new("output")
				.required(true)
				.takes_value(true)
				.value_name("OUTPUT")
				.help("Output file"),
		)
		.arg(
			Arg::new("width")
				.short('w')
				.takes_value(true)
				.value_name("WIDTH")
				.help("width")
				.default_value("640"),
		)
		.arg(
			Arg::new("height")
				.short('h')
				.takes_value(true)
				.value_name("HEIGHT")
				.help("height")
				.default_value("480"),
		)
		.arg(
			Arg::new("duration")
				.short('d')
				.takes_value(true)
				.value_name("DURATION")
				.help("duration in seconds")
				.default_value("10"),
		)
		.get_matches();

	let output_filename = matches.value_of("output").unwrap();
	let width = matches.value_of("width").unwrap().parse().unwrap();
	let height = matches.value_of("height").unwrap().parse().unwrap();
	let duration = matches.value_of("duration").unwrap().parse().unwrap();

	let duration = Duration::from_secs_f32(duration);

	let pixel_format =
		ac_ffmpeg::codec::video::frame::get_pixel_format("yuv420p");

	let frames: Vec<VideoFrame> =
		vec![
			VideoFrameMut::black(pixel_format, width as _, height as _)
				.freeze();
			24 * duration.as_secs() as usize
		];

	if let Err(err) = vida::encode_video(
		output_filename,
		width,
		height,
		pixel_format,
		frames.into_iter(),
	) {
		eprintln!("ERROR: {}", err);
	}
}
