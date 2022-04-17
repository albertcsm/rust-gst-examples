use gstreamer as gst;
use glib::FlagsClass;
use gst::prelude::*;

use anyhow::Error;

use termion::event::Key;
use termion::input::TermRead;

use std::{thread, time};

fn main() {
    gst_examples::run(tutorial_main);
}

fn analyze_streams(playbin: &gst::Element) {
    let n_video = playbin.property::<i32>("n-video");
    let n_audio = playbin.property::<i32>("n-audio");
    let n_text = playbin.property::<i32>("n-text");
    println!(
        "{} video stream(s), {} audio stream(s), {} subtitle stream(s)",
        n_video, n_audio, n_text
    );

    for i in 0..n_video {
        let tags = playbin.emit_by_name::<Option<gst::TagList>>("get-video-tags", &[&i]);

        if let Some(tags) = tags {
            println!("video stream {}:", i);
            if let Some(codec) = tags.get::<gst::tags::VideoCodec>() {
                println!("    codec: {}", codec.get());
            }
        }
    }

    for i in 0..n_audio {
        let tags = playbin.emit_by_name::<Option<gst::TagList>>("get-audio-tags", &[&i]);

        if let Some(tags) = tags {
            println!("audio stream {}:", i);
            if let Some(codec) = tags.get::<gst::tags::AudioCodec>() {
                println!("    codec: {}", codec.get());
            }
            if let Some(codec) = tags.get::<gst::tags::LanguageCode>() {
                println!("    language: {}", codec.get());
            }
            if let Some(codec) = tags.get::<gst::tags::Bitrate>() {
                println!("    bitrate: {}", codec.get());
            }
        }
    }

    for i in 0..n_text {
        let tags = playbin.emit_by_name::<Option<gst::TagList>>("get-text-tags", &[&i]);

        if let Some(tags) = tags {
            println!("subtitle stream {}:", i);
            if let Some(codec) = tags.get::<gst::tags::LanguageCode>() {
                println!("    language: {}", codec.get());
            }
        } else {
            println!("no tags found for sub track");
        }
    }

    let current_video = playbin.property::<i32>("current-video");
    let current_audio = playbin.property::<i32>("current-audio");
    let current_text = playbin.property::<i32>("current-text");
    println!(
        "Currently playing video stream {}, audio stream {}, subtitle stream {}",
        current_video, current_audio, current_text
    );
    println!("Type any number and hit ENTER to select a different subtitle stream");
}

fn handle_keyboard(playbin: &gst::Element, main_loop: &glib::MainLoop) {
    let mut stdin = termion::async_stdin().keys();

    loop {
        if let Some(Ok(input)) = stdin.next() {
            match input {
                Key::Char(index) => {
                    if let Some(index) = index.to_digit(10) {
                        // Here index can only be 0-9
                        let index = index as i32;
                        let n_audio = playbin.property::<i32>("n-text");

                        if index < n_audio {
                            println!("Setting current subtitle stream to {}", index);
                            playbin.set_property("current-text", index);
                        } else {
                            eprintln!("Index out of bounds");
                        }
                    }
                }
                Key::Ctrl('c') => {
                    main_loop.quit();
                    break;
                }
                _ => continue,
            };
        }
        thread::sleep(time::Duration::from_millis(50));
    }
}

fn tutorial_main() -> Result<(), Error> {
    // Create the main loop
    let main_loop = glib::MainLoop::new(None, false);

    // Initialize GStreamer
    gst::init()?;

    // Create PlayBin element
    let playbin = gst::ElementFactory::make("playbin", Some("playbin"))?;

    // Set URI to play
    let uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.ogv";
    playbin.set_property("uri", uri);

    // Set the subtitle URI and font description
    let subtitle_uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer_gr.srt";
    playbin.set_property("suburi", subtitle_uri);
    playbin.set_property("subtitle-font-desc", "Sans, 18");

    // Set flags to show Audio, Video and Subtitles
    let flags = playbin.property_value("flags");
    let flags_class = FlagsClass::new(flags.type_()).unwrap();

    let flags = flags_class
        .builder_with_value(flags)
        .unwrap()
        .set_by_nick("audio")
        .set_by_nick("video")
        .set_by_nick("text")
        .build()
        .unwrap();
    playbin.set_property_from_value("flags", &flags);

    // Add a keyboard watch so we get notified of keystrokes
    let playbin_clone = playbin.clone();
    let main_loop_clone = main_loop.clone();
    thread::spawn(move || handle_keyboard(&playbin_clone, &main_loop_clone));

    // Add a bus watch, so we get notified when a message arrives
    let playbin_clone = playbin.clone();
    let main_loop_clone = main_loop.clone();
    let bus = playbin.bus().unwrap();
    bus.add_watch(move |_bus, message| {
        use gst::MessageView;
        match message.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?} {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                eprintln!("Debugging information: {:?}", err.debug());
                main_loop_clone.quit();
                Continue(false)
            }
            MessageView::StateChanged(state_changed) => {
                if state_changed
                    .src()
                    .map(|s| s == playbin_clone)
                    .unwrap_or(false)
                    && state_changed.current() == gst::State::Playing
                {
                    analyze_streams(&playbin_clone);
                }
                Continue(true)
            }
            MessageView::Eos(..) => {
                println!("Reached end of stream");
                main_loop_clone.quit();
                Continue(false)
            }
            _ => Continue(true),
        }
    })?;

    // Start playing
    playbin.set_state(gst::State::Playing)?;

    // Set GLib mainloop to run
    main_loop.run();

    // Clean up
    playbin.set_state(gst::State::Null)?;

    Ok(())
}