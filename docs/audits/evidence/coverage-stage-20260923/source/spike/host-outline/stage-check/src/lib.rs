#[cfg(test)]
mod tests {
    extern "C" { fn check_coverage_stage(case: i32) -> u32; }
    macro_rules! case {
        ($name:ident, $id:expr) => {
            #[test]
            fn $name() {
                assert_eq!(unsafe { check_coverage_stage($id) }, 0, stringify!($name));
            }
        };
    }
    case!(read_only_preserves_binding, 0);
    case!(idempotent_binding_does_not_write, 1);
    case!(updates_layer_and_stage_together, 2);
    case!(updates_layer_with_same_stage, 3);
    case!(updates_stage_with_same_layer, 4);
    case!(acquire_failure_owns_nothing, 5);
    case!(null_table_still_releases_suite, 6);
    case!(missing_callback_refuses_and_releases, 7);
    case!(stream_failure_releases_suite, 8);
    case!(null_stream_is_rejected, 9);
    case!(read_failure_disposes_stream, 10);
    case!(write_failure_disposes_value_and_stream, 11);
    case!(wrong_readback_layer_is_rejected, 12);
    case!(wrong_readback_stage_is_rejected, 13);
    case!(readback_failure_disposes_both_reads_correctly, 14);
    case!(value_disposal_failure_cannot_publish_success, 15);
    case!(stream_disposal_failure_still_releases_suite, 16);
    case!(suite_release_failure_cannot_publish_success, 17);
    case!(null_effect_never_calls_host, 18);
    case!(missing_basic_callback_never_calls_host, 19);
    case!(unapproved_write_stage_never_calls_host, 20);
    case!(read_preserves_unknown_stage_for_conflict_detection, 21);
}
