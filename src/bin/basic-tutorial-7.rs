use gstreamer as gst;
use gst::prelude::*;

fn main() {
    gst_examples::run(tutorial_main);
}

fn tutorial_main() {
    // Initialize GStreamer
    if let Err(err) = gst::init() {
        eprintln!("Failed to initialize Gst: {}", err);
        return;
    }

    let audio_source = gst::ElementFactory::make("audiotestsrc", Some("audio_source")).unwrap();
    let tee = gst::ElementFactory::make("tee", Some("tee")).unwrap();
    let audio_queue = gst::ElementFactory::make("queue", Some("audio_queue")).unwrap();
    let audio_convert = gst::ElementFactory::make("audioconvert", Some("audio_convert")).unwrap();
    let audio_resample =
        gst::ElementFactory::make("audioresample", Some("audio_resample")).unwrap();
    let audio_sink = gst::ElementFactory::make("autoaudiosink", Some("audio_sink")).unwrap();
    let video_queue = gst::ElementFactory::make("queue", Some("video_queue")).unwrap();
    let visual = gst::ElementFactory::make("wavescope", Some("visual")).unwrap();
    let video_convert = gst::ElementFactory::make("videoconvert", Some("video_convert")).unwrap();
    let video_sink = gst::ElementFactory::make("autovideosink", Some("video_sink")).unwrap();

    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    audio_source.set_property("freq", 215.0);
    visual.set_property_from_str("shader", "none");
    visual.set_property_from_str("style", "lines");

    pipeline
        .add_many(&[
            &audio_source,
            &tee,
            &audio_queue,
            &audio_convert,
            &audio_resample,
            &audio_sink,
            &video_queue,
            &visual,
            &video_convert,
            &video_sink,
        ])
        .unwrap();

    gst::Element::link_many(&[&audio_source, &tee]).unwrap();
    gst::Element::link_many(&[&audio_queue, &audio_convert, &audio_resample, &audio_sink]).unwrap();
    gst::Element::link_many(&[&video_queue, &visual, &video_convert, &video_sink]).unwrap();

    let tee_audio_pad = tee.request_pad_simple("src_%u").unwrap();
    println!(
        "Obtained request pad {} for audio branch",
        tee_audio_pad.name()
    );
    let queue_audio_pad = audio_queue.static_pad("sink").unwrap();
    tee_audio_pad.link(&queue_audio_pad).unwrap();

    let tee_video_pad = tee.request_pad_simple("src_%u").unwrap();
    println!(
        "Obtained request pad {} for video branch",
        tee_video_pad.name()
    );
    let queue_video_pad = video_queue.static_pad("sink").unwrap();
    tee_video_pad.link(&queue_video_pad).unwrap();

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?}: {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                eprintln!("Debugging information: {:?}", err.debug());
                break;
            }
            MessageView::Eos(..) => break,
            _ => (),
        }
    }

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}