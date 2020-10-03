use audiobench::*;

const TYPICAL_BUFFER_LENGTH: usize = 512;
const TYPICAL_SAMPLE_RATE: usize = 44100;
const TYPICAL_TEST_DURATION: usize = 100;
const TYPICAL_NUM_NOTES: usize = 10;
/// The 'default' patch from the factory library.
const PATCH_DEFAULT: &[u8] =
    "AQAHRGVmYXVsdAAAAQAE_wHgAKgFAVAAqAYAeADACgAwADAOIgABAliAAGqqAAA7o9cKAD5MzM0APpmZmgMD"
        .as_bytes();
/// The 'pluckypluckypluck' patch from the factory library.
const PATCH_PLUCK: &[u8] = "AQARUGx1Y2t5cGx1Y2t5cGx1Y2sAAAEACv__DwK4AKgFAKj_6A4AqABgEgEIAWgOAfgAqBEBOADYBgE4ASAGADABaAz_cABgDv-gAPAHEomiQgAEAgUGCQcJIgKUmQIJAADo9QAAAAAAAAAAO6PXCgA9d85jAD55hhgAADvqv2gBAAA7o9cKAD5MzIYAPpmZmgAAAAcAAAAAO6PXCgA9lewBAD6ZmZogMwAI8zMFHgHuFAMDCBR6frg".as_bytes();

struct TestParameters {
    buffer_length: usize,
    sample_rate: usize,
    num_seconds: usize,
    num_notes: usize,
    patch_name: &'static str,
    patch_data: &'static [u8],
}

fn do_benchmark(test_id: usize, params: &TestParameters) {
    let TestParameters {
        buffer_length,
        sample_rate,
        num_seconds,
        num_notes,
        patch_name,
        patch_data,
    } = params;

    println!("================================================================================");
    println!("EXECUTING TEST {}", test_id);
    println!("Buffer lenth:     {} samples", buffer_length);
    println!("Sample rate:      {} hertz", sample_rate);
    println!("Render time:      {} seconds", num_seconds);
    println!("Number of notes:  {}", num_notes);
    println!("Patch name:       {}", patch_name);

    let mut instance = Instance::new();
    instance.set_host_format(*buffer_length, *sample_rate);
    instance.deserialize_patch(patch_data);
    // We need to do this at least once so that the audio thread can adjust to the new parameters.
    // Without this, the audio thread will shut down the notes when the actual benchmark starts.
    instance.render_audio();
    let note_distance = 100 / *num_notes;
    for offset in 0..*num_notes {
        instance.start_note(10 + offset * note_distance, 0.8);
    }
    let mut anti_optimization_accumulator = 0.0;
    let num_render_cycles = sample_rate * num_seconds / buffer_length;
    for _ in 0..num_render_cycles {
        let audio = instance.render_audio();
        for sample in audio {
            anti_optimization_accumulator += sample;
        }
    }

    println!("");
    println!("RESULTS:");
    println!("{}", instance.perf_report());
    println!("");
}

const TESTS: [TestParameters; 2] = [
    TestParameters {
        buffer_length: TYPICAL_BUFFER_LENGTH,
        sample_rate: TYPICAL_SAMPLE_RATE,
        num_seconds: TYPICAL_TEST_DURATION,
        num_notes: TYPICAL_NUM_NOTES,
        patch_name: "Default",
        patch_data: PATCH_DEFAULT,
    },
    TestParameters {
        buffer_length: TYPICAL_BUFFER_LENGTH,
        sample_rate: TYPICAL_SAMPLE_RATE,
        num_seconds: TYPICAL_TEST_DURATION,
        num_notes: TYPICAL_NUM_NOTES,
        patch_name: "Pluck",
        patch_data: PATCH_PLUCK,
    },
];

fn main() {
    for (index, test) in TESTS.iter().enumerate() {
        do_benchmark(index, test);
    }
}
