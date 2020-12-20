use crate::prelude::*;
use std::time::{Duration, Instant};

pub struct PerfCountSection {
    index: usize,
    name: &'static str,
}

pub mod sections {
    use super::PerfCountSection;

    pub const GLOBAL_SETUP: PerfCountSection = PerfCountSection {
        index: 0,
        name: "Global Setup",
    };
    pub const NOTE_SETUP: PerfCountSection = PerfCountSection {
        index: 1,
        name: "Note Setup",
    };
    pub const NODESPEAK_EXEC: PerfCountSection = PerfCountSection {
        index: 2,
        name: "Nodespeak Exec",
    };
    pub const NOTE_FINALIZE: PerfCountSection = PerfCountSection {
        index: 3,
        name: "Note Finalize",
    };
    pub const GLOBAL_FINALIZE: PerfCountSection = PerfCountSection {
        index: 4,
        name: "Global Finalize",
    };
    pub const GENERATE_CODE: PerfCountSection = PerfCountSection {
        index: 8,
        name: "Generate Code",
    };
    pub const COMPILE_CODE: PerfCountSection = PerfCountSection {
        index: 5,
        name: "Compile Code",
    };
    pub const COLLECT_AUTOCON_DATA: PerfCountSection = PerfCountSection {
        index: 6,
        name: "Collect Autocon Data",
    };
    pub const COLLECT_STATICON_DATA: PerfCountSection = PerfCountSection {
        index: 7,
        name: "Collect Staticon Data",
    };
    pub const COMPILER_AST_PHASE: PerfCountSection = PerfCountSection {
        index: 9,
        name: "Compiler AST Phase",
    };
    pub const COMPILER_VAGUE_PHASE: PerfCountSection = PerfCountSection {
        index: 10,
        name: "Compiler Vague Phase",
    };
    pub const COMPILER_RESOLVED_PHASE: PerfCountSection = PerfCountSection {
        index: 11,
        name: "Compiler Resolved Phase",
    };
    pub const COMPILER_TRIVIAL_PHASE: PerfCountSection = PerfCountSection {
        index: 12,
        name: "Compiler Trivial Phase",
    };
    pub const COMPILER_LLVMIR_PHASE: PerfCountSection = PerfCountSection {
        index: 13,
        name: "Compiler LLVMIR Phase",
    };

    pub const NUM_SECTIONS: usize = 14;
    pub const ALL_SECTIONS: [&'static PerfCountSection; NUM_SECTIONS] = [
        &GENERATE_CODE,
        &COMPILE_CODE,
        &COMPILER_AST_PHASE,
        &COMPILER_VAGUE_PHASE,
        &COMPILER_RESOLVED_PHASE,
        &COMPILER_TRIVIAL_PHASE,
        &COMPILER_LLVMIR_PHASE,
        &COLLECT_AUTOCON_DATA,
        &COLLECT_STATICON_DATA,
        &GLOBAL_SETUP,
        &NOTE_SETUP,
        &NODESPEAK_EXEC,
        &NOTE_FINALIZE,
        &GLOBAL_FINALIZE,
    ];
}

use sections::NUM_SECTIONS;

pub struct PerfSectionGuard {
    section_index: usize,
    start_time: Instant,
    handled: bool,
}

impl Drop for PerfSectionGuard {
    fn drop(&mut self) {
        if !self.handled {
            let mut name: &'static str = "";
            for section in &sections::ALL_SECTIONS {
                if section.index == self.section_index {
                    name = &section.name;
                }
            }
            panic!(
                "PerfSectionGuard({}) dropped before being handled by end_section().",
                name
            );
        }
    }
}

pub trait PerfCounter {
    fn new() -> Self;
    fn begin_section(&mut self, section: &PerfCountSection) -> PerfSectionGuard {
        PerfSectionGuard {
            section_index: section.index,
            start_time: Instant::now(),
            handled: false,
        }
    }
    fn end_section(&mut self, section: PerfSectionGuard);
    fn add_externally_timed_section(&mut self, section: &PerfCountSection, duration: Duration);
    fn report(&self) -> String;
}

/// Does nothing.
pub struct NoopPerfCounter;

impl PerfCounter for NoopPerfCounter {
    fn new() -> Self {
        Self
    }

    fn end_section(&mut self, mut section: PerfSectionGuard) {
        section.handled = true;
    }

    fn add_externally_timed_section(&mut self, _section: &PerfCountSection, _duration: Duration) {}

    fn report(&self) -> String {
        "No report available (NoopPerfCounter)".to_owned()
    }
}

/// Limited statistics, but fast enough to run in production builds without
/// screwing with anything.
pub struct SimplePerfCounter {
    num_invocations: [u32; NUM_SECTIONS],
    cumulative_time: [Duration; NUM_SECTIONS],
}

impl PerfCounter for SimplePerfCounter {
    fn new() -> Self {
        Self {
            num_invocations: [0; NUM_SECTIONS],
            cumulative_time: [Duration::from_secs(0); NUM_SECTIONS],
        }
    }

    fn end_section(&mut self, mut section: PerfSectionGuard) {
        // We do this first to make the timing statistics as accurate as possible.
        let time = section.start_time.elapsed();
        self.cumulative_time[section.section_index] += time;
        self.num_invocations[section.section_index] += 1;
        section.handled = true;
    }

    /// This allows timing sections of code where it may be inconvenient to pass a reference to the
    /// entire performance counter.
    fn add_externally_timed_section(&mut self, section: &PerfCountSection, duration: Duration) {
        self.cumulative_time[section.index] += duration;
        self.num_invocations[section.index] += 1;
    }

    fn report(&self) -> String {
        let mut report = String::new();
        report +=
            &format!("SECTION NAME                   | TOTAL TIME | SAMPLES | TIME PER SAMPLE \n");
        let mut everything_time = 0.0;
        for section in &sections::ALL_SECTIONS {
            let invocations = self.num_invocations[section.index];
            if invocations == 0 {
                continue;
            }
            let total_time = self.cumulative_time[section.index].as_secs_f64();
            everything_time += total_time;
            let average_time = total_time / (invocations as f64);
            report += &format!(
                "{:<30} | {:>10} | {:>7} | {:>15} \n",
                section.name,
                total_time.format_metric(6, "s"),
                invocations,
                average_time.format_metric(6, "s")
            );
        }
        report += &format!(
            "                                 {:>10}",
            everything_time.format_metric(6, "s")
        );
        report
    }
}
