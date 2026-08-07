        #[test]
        fn direct_renewal_dream_construction_manifest() {
            assert_eq!(GATE_OWNERS.len(), 15);
            assert_eq!(RENDER_SPEC.ratios, [4, 8, 16]);
            assert_eq!(RENDER_SPEC.min_sample_rate, 8_000);
            assert_eq!(RENDER_SPEC.max_sample_rate, 192_000);
            assert_eq!(EVIDENCE_SPEC.admission_seed, 0x0123_4567_89ab_cdef);
            assert_eq!(RUN_SPEC.test_threads, 1);
            assert_eq!(RUN_SPEC.retries, 0);
            assert_eq!(RUN_SPEC.conformance_rounds, 2);
            const { assert!(MEMORY_SPEC.duration_independent); }
            assert_eq!(MEMORY_SPEC.max_working_bytes, 32 * 1024 * 1024);
            assert_eq!(
                GATE_OWNERS[..10]
                    .iter()
                    .map(|owner| owner.rows)
                    .sum::<usize>(),
                EVIDENCE_SPEC.structural_rows
            );
            assert_eq!(
                GATE_OWNERS[..10]
                    .iter()
                    .map(|owner| owner.renders)
                    .sum::<usize>(),
                EVIDENCE_SPEC.structural_renders
            );
            assert_eq!(
                GATE_OWNERS[10..]
                    .iter()
                    .map(|owner| owner.rows)
                    .sum::<usize>(),
                EVIDENCE_SPEC.synthetic_rows
            );
            assert_eq!(
                GATE_OWNERS[10..]
                    .iter()
                    .map(|owner| owner.renders)
                    .sum::<usize>(),
                EVIDENCE_SPEC.synthetic_renders
            );
            for (index, owner) in GATE_OWNERS.iter().enumerate() {
                let expected = if index < 10 {
                    format!("S{:02}", index + 1)
                } else {
                    format!("Y{:02}", index - 9)
                };
                assert_eq!(owner.id, expected);
                assert!(owner.test_name.starts_with("direct_renewal_dream_"));
                assert_ne!(owner.owner as usize, 0);
                assert_ne!(owner.assertion_mask, 0);
                assert_ne!(owner.receipt_field_mask, 0);
                assert_eq!(owner.deadline_seconds, 600);
                assert!(owner.worst_output_frames <= 26_880_000);
                if owner.construction_oracle {
                    (owner.owner)().unwrap();
                }
            }
            let ledger = include_str!("../regression_manifest.tsv");
            let mut ledger_lines = ledger.lines();
            assert_eq!(
                ledger_lines.next(),
                Some("owner\ttest_name\trow_index\trow_id\trender_count\towner_output_frames_bound")
            );
            let ledger_rows = ledger_lines.map(|line| line.split('\t').collect::<Vec<_>>()).collect::<Vec<_>>();
            assert_eq!(ledger_rows.len(), EVIDENCE_SPEC.structural_rows + EVIDENCE_SPEC.synthetic_rows);
            for owner in GATE_OWNERS {
                let rows = ledger_rows.iter().filter(|row| row[0] == owner.id).collect::<Vec<_>>();
                assert_eq!(rows.len(), owner.rows);
                assert_eq!(rows.iter().map(|row| row[4].parse::<usize>().unwrap()).sum::<usize>(), owner.renders);
                for (row_index, row) in rows.into_iter().enumerate() {
                    assert_eq!(row.len(), 6);
                    assert_eq!(row[1], owner.test_name);
                    assert_eq!(row[2].parse::<usize>().unwrap(), row_index);
                    assert!(!row[3].is_empty());
                    assert_eq!(row[5].parse::<usize>().unwrap(), owner.worst_output_frames);
                }
            }
            assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
            for source in SourceKind::ALL {
                assert_eq!(hash_f32(&source.generate()), source.expected_hash());
            }
        }
