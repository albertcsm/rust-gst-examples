use gstreamer as gst;
use gst::prelude::*;

fn main() {
    gst_examples::run(tutorial_main);
}

fn tutorial_main() {
    // Initialize GStreamer
    gst::init().unwrap();

    // Create the elements
    let source = gst::ElementFactory::make("videotestsrc", Some("source"))
        .expect("Could not create source element.");
    let sink = gst::ElementFactory::make("autovideosink", Some("sink"))
        .expect("Could not create sink element");

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Build the pipeline
    pipeline.add_many(&[&source, &sink]).unwrap();
    source.link(&sink).expect("Elements could not be linked.");

    // Modify the source's properties
    source.set_property_from_str("pattern", "smpte");

    // Start playing
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    // Wait until error or EOS
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