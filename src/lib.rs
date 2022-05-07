use ac_ffmpeg::{
	codec::{
		video::{PixelFormat, VideoDecoder, VideoEncoder, VideoFrame},
		CodecParameters, Decoder, Encoder,
	},
	format::{
		demuxer::{Demuxer, DemuxerWithStreamInfo},
		io::IO,
		muxer::{Muxer, OutputFormat},
	},
	time::{TimeBase, Timestamp},
};
use anyhow::{Context, Result};
use std::fs::File;

pub fn open_input(path: &str) -> Result<DemuxerWithStreamInfo<File>> {
	let input = File::open(path)
		.context(format!("unable to open input file {}", path))?;

	let io = IO::from_seekable_read_stream(input);

	let demuxer = Demuxer::builder()
		.build(io)?
		.find_stream_info(None)
		.map_err(|(_, err)| err)?;

	Ok(demuxer)
}

/// Open a given output file.
pub fn open_output(
	path: &str,
	elementary_streams: &[CodecParameters],
) -> Result<Muxer<File>> {
	let output_format =
		OutputFormat::guess_from_file_name(path).context(anyhow::anyhow!(
			format!("unable to guess output format for file: {}", path)
		))?;

	let output = File::create(path).context(anyhow::anyhow!(format!(
		"unable to create output file {}",
		path
	)))?;

	let io = IO::from_seekable_write_stream(output);

	let mut muxer_builder = Muxer::builder();

	for codec_parameters in elementary_streams {
		muxer_builder.add_stream(codec_parameters)?;
	}

	let muxer = muxer_builder.build(io, output_format)?;

	Ok(muxer)
}

/// Create h264 encoded black video file of a given length and with a given
/// resolution.
pub fn encode_video(
	output: &str,
	width: u32,
	height: u32,
	pixel_format: PixelFormat,
	mut frames: impl Iterator<Item = VideoFrame>,
) -> Result<()> {
	let time_base = TimeBase::new(1, 25);

	let mut encoder = VideoEncoder::builder("libx265")?
		.pixel_format(pixel_format)
		.width(width as _)
		.height(height as _)
		.time_base(time_base)
		.build()
		.context("Failed to create video encoder")?;

	let codec_parameters = encoder.codec_parameters().into();

	let mut muxer = open_output(output, &[codec_parameters])?;

	let mut frame_idx = 0;
	let mut frame_timestamp = Timestamp::new(frame_idx, time_base);

	while let Some(frame) = frames.next() {
		let cloned_frame = frame.with_pts(frame_timestamp);

		encoder.push(cloned_frame)?;

		while let Some(packet) = encoder.take()? {
			muxer.push(packet.with_stream_index(0))?;
		}

		frame_idx += 1;
		frame_timestamp = Timestamp::new(frame_idx, time_base);
	}

	encoder.flush()?;

	while let Some(packet) = encoder.take()? {
		muxer.push(packet.with_stream_index(0))?;
	}

	muxer.flush()?;

	Ok(())
}

pub fn decode_video(
	input: &str,
	frame_fn: Box<dyn Fn(VideoFrame)>,
) -> Result<()> {
	let mut demuxer = open_input(input)?;

	let (stream_index, (stream, _)) = demuxer
		.streams()
		.iter()
		.map(|stream| (stream, stream.codec_parameters()))
		.enumerate()
		.find(|(_, (_, params))| params.is_video_codec())
		.context("no video stream")?;

	let mut decoder = VideoDecoder::from_stream(stream)?.build()?;

	// process data
	while let Some(packet) = demuxer.take()? {
		if packet.stream_index() != stream_index {
			continue;
		}

		decoder.push(packet)?;

		while let Some(frame) = decoder.take()? {
			frame_fn(frame);
		}
	}

	decoder.flush()?;

	Ok(())
}
