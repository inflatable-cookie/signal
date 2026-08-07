        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum SourceKind {
            LowTone,
            MidTone,
            Chord,
            HarmonicPad,
            Impulse,
            ImpulseTrain,
            SilenceGap,
            UniformNoise,
            RademacherNoise,
            AmplitudeModulatedNoise,
        }

        impl SourceKind {
            const ALL: [Self; 10] = [
                Self::LowTone,
                Self::MidTone,
                Self::Chord,
                Self::HarmonicPad,
                Self::Impulse,
                Self::ImpulseTrain,
                Self::SilenceGap,
                Self::UniformNoise,
                Self::RademacherNoise,
                Self::AmplitudeModulatedNoise,
            ];

            fn id(self) -> &'static str {
                match self {
                    Self::LowTone => "low-tone",
                    Self::MidTone => "mid-tone",
                    Self::Chord => "chord",
                    Self::HarmonicPad => "harmonic-pad",
                    Self::Impulse => "impulse",
                    Self::ImpulseTrain => "impulse-train",
                    Self::SilenceGap => "silence-gap",
                    Self::UniformNoise => "uniform-noise",
                    Self::RademacherNoise => "rademacher-noise",
                    Self::AmplitudeModulatedNoise => "amplitude-modulated-noise",
                }
            }

            fn expected_hash(self) -> &'static str {
                match self {
                    Self::AmplitudeModulatedNoise => "ba6b9c244618939769e7283fac92f198690238db0c96d99c280892ee358ab31b",
                    Self::Chord => "b7c85b6faed8d670fd7eefa66f7be6a89df0f7c5c3a4444146a2d083a70792e7",
                    Self::HarmonicPad => "732895709a05fa724d9dd76a03bc22c64865b84ba93ba351e49354b31f95e96c",
                    Self::ImpulseTrain => "47314d3121745479660fb0d0350b41aec987074f75a503181805e5f4545e8138",
                    Self::Impulse => "fc73433e0fab2786572b6a98bd0cc9f86145960581d77e3dbc7d1bfa6abca57b",
                    Self::LowTone => "2c6d1c766ce73ac75000f8e9cbd6238fafbf180c64baa33d62a55fb9517f32e1",
                    Self::MidTone => "36397e016a1d00a5bf1884d049a1454ab7342965ffd3cf21179474610a218b33",
                    Self::RademacherNoise => "c1ae606691767937990e38a314ceadeee6c7cb0a9da63c7ed3d3a3ef31b838b5",
                    Self::SilenceGap => "1c17fdc3cecd09cfcc403c39a9c7aadb75c41239433c20863cb967fbcef0013e",
                    Self::UniformNoise => "cde1917d6afdfe3dfb260da2a6273a243e261032bb9fe6624e49020089ee9923",
                }
            }

            fn support(self) -> (usize, usize) {
                match self {
                    Self::Impulse => (48_000, 48_001),
                    Self::ImpulseTrain => (19_200, 77_798),
                    _ => (24_000, 72_000),
                }
            }

            fn generate(self) -> Vec<f32> {
                let mut samples = vec![0.0_f32; SYNTHETIC_SOURCE_FRAMES];
                match self {
                    Self::Impulse => samples[48_000] = 1.0,
                    Self::ImpulseTrain => {
                        for (frame, value) in [(19_200, 1.0), (38_937, -0.8), (58_103, 0.65), (77_797, -0.5)] {
                            samples[frame] = value;
                        }
                    }
                    _ => {
                        for (frame, sample) in samples.iter_mut().enumerate() {
                            if !(24_000..72_000).contains(&frame) {
                                continue;
                            }
                            let raw = match self {
                                Self::LowTone => sinusoid(frame, 110.0, 0.5),
                                Self::MidTone => sinusoid(frame, 440.0, 0.5),
                                Self::Chord => [110.0, 164.813_778, 220.0, 277.182_631, 329.627_557]
                                    .into_iter()
                                    .map(|frequency| sinusoid(frame, frequency, 0.1))
                                    .sum(),
                                Self::HarmonicPad | Self::SilenceGap => (1..=8)
                                    .map(|partial| {
                                        (0.35 / partial as f64)
                                            * (2.0
                                                * std::f64::consts::PI
                                                * 110.0
                                                * partial as f64
                                                * frame as f64
                                                / SAMPLE_RATE as f64)
                                                .sin()
                                    })
                                    .sum(),
                                Self::UniformNoise => {
                                    let unit = high_53(mix64(frame as u64 ^ TEST_TAG)) as f64
                                        / (1_u64 << 53) as f64;
                                    0.5 * (2.0 * unit - 1.0)
                                }
                                Self::RademacherNoise => rademacher_sign(frame) * 0.5,
                                Self::AmplitudeModulatedNoise => {
                                    rademacher_sign(frame)
                                        * 0.5
                                        * (0.5
                                            + 0.375
                                                * (2.0
                                                    * std::f64::consts::PI
                                                    * 1.7
                                                    * frame as f64
                                                    / SAMPLE_RATE as f64)
                                                    .sin())
                                }
                                Self::Impulse | Self::ImpulseTrain => unreachable!(),
                            };
                            let gap = self == Self::SilenceGap && (42_000..54_000).contains(&frame);
                            let weight = support_weight(frame);
                            *sample = if gap {
                                0.0
                            } else {
                                (raw * weight) as f32
                            };
                        }
                    }
                }
                samples
            }
        }

        fn sinusoid(frame: usize, frequency: f64, amplitude: f64) -> f64 {
            amplitude
                * (2.0 * std::f64::consts::PI * frequency * frame as f64
                    / SAMPLE_RATE as f64)
                    .sin()
        }

        fn rademacher_sign(frame: usize) -> f64 {
            if mix64(frame as u64 ^ TEST_TAG) >> 63 == 1 { 1.0 } else { -1.0 }
        }

        fn support_weight(frame: usize) -> f64 {
            match frame {
                24_000..=26_047 => {
                    0.5 - 0.5 * (std::f64::consts::PI * (frame - 24_000) as f64 / 2_047.0).cos()
                }
                26_048..=69_951 => 1.0,
                69_952..=71_999 => {
                    0.5 - 0.5 * (std::f64::consts::PI * (71_999 - frame) as f64 / 2_047.0).cos()
                }
                _ => 0.0,
            }
        }

        fn sha256_hex(bytes: &[u8]) -> String {
            const INITIAL: [u32; 8] = [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ];
            const K: [u32; 64] = [
                0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
                0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
                0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
                0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
                0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
                0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
                0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
                0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
            ];
            let bit_len = (bytes.len() as u64).wrapping_mul(8);
            let mut padded = Vec::with_capacity((bytes.len() + 72) & !63);
            padded.extend_from_slice(bytes);
            padded.push(0x80);
            while padded.len() % 64 != 56 { padded.push(0); }
            padded.extend_from_slice(&bit_len.to_be_bytes());
            let mut hash = INITIAL;
            for chunk in padded.chunks_exact(64) {
                let mut words = [0_u32; 64];
                for (index, word) in words[..16].iter_mut().enumerate() {
                    *word = u32::from_be_bytes(chunk[index * 4..index * 4 + 4].try_into().unwrap());
                }
                for index in 16..64 {
                    let s0 = words[index - 15].rotate_right(7) ^ words[index - 15].rotate_right(18) ^ (words[index - 15] >> 3);
                    let s1 = words[index - 2].rotate_right(17) ^ words[index - 2].rotate_right(19) ^ (words[index - 2] >> 10);
                    words[index] = words[index - 16].wrapping_add(s0).wrapping_add(words[index - 7]).wrapping_add(s1);
                }
                let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = hash;
                for index in 0..64 {
                    let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                    let choice = (e & f) ^ (!e & g);
                    let temp1 = h.wrapping_add(s1).wrapping_add(choice).wrapping_add(K[index]).wrapping_add(words[index]);
                    let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                    let majority = (a & b) ^ (a & c) ^ (b & c);
                    let temp2 = s0.wrapping_add(majority);
                    h = g; g = f; f = e; e = d.wrapping_add(temp1);
                    d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
                }
                for (state, value) in hash.iter_mut().zip([a, b, c, d, e, f, g, h]) {
                    *state = state.wrapping_add(value);
                }
            }
            hash.iter().map(|word| format!("{word:08x}")).collect()
        }

        fn hash_f32(samples: &[f32]) -> String {
            let mut bytes = Vec::with_capacity(std::mem::size_of_val(samples));
            for sample in samples { bytes.extend_from_slice(&sample.to_le_bytes()); }
            sha256_hex(&bytes)
        }

        fn checkpoint_identity() -> String {
            let output = Command::new("git").args(["rev-parse", "HEAD"]).output().unwrap();
            assert!(output.status.success());
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        }

        fn json_string(value: &str) -> String {
            let mut output = String::from("\"");
            for character in value.chars() {
                match character {
                    '\\' => output.push_str("\\\\"),
                    '"' => output.push_str("\\\""),
                    '\n' => output.push_str("\\n"),
                    '\r' => output.push_str("\\r"),
                    '\t' => output.push_str("\\t"),
                    other => output.push(other),
                }
            }
            output.push('"');
            output
        }

        struct Receipt<'a> {
            owner: &'a str,
            row_index: usize,
            row_id: &'a str,
            status: &'a str,
            render_count: usize,
            output_frames: usize,
            input_hash: &'a str,
            output_hash: &'a str,
            assertions: Vec<String>,
            diagnostics: Vec<String>,
        }

        fn receipt_directory(owner: &str) -> PathBuf {
            let stage = std::env::var("DIRECT_RENEWAL_STAGE").unwrap_or_else(|_| "synthetic".into());
            let round = std::env::var("DIRECT_RENEWAL_ROUND").unwrap_or_else(|_| "0".into());
            PathBuf::from("target/creative-stretch-direct-renewal-31-66")
                .join(checkpoint_identity())
                .join(stage)
                .join(round)
                .join(owner)
        }

        fn write_receipt(receipt: Receipt<'_>) {
            let directory = receipt_directory(receipt.owner);
            fs::create_dir_all(&directory).unwrap();
            let path = directory.join("rows.jsonl");
            let mut file = OpenOptions::new().create(true).append(true).open(path).unwrap();
            let assertions = receipt.assertions.iter().map(|item| json_string(item)).collect::<Vec<_>>().join(",");
            let diagnostics = receipt.diagnostics.iter().map(|item| json_string(item)).collect::<Vec<_>>().join(",");
            writeln!(
                file,
                "{{\"schema\":{},\"checkpoint\":{},\"stage\":{},\"round\":{},\"owner\":{},\"row_index\":{},\"row_id\":{},\"status\":{},\"render_count\":{},\"output_frames\":{},\"input_sha256\":{},\"output_sha256\":{},\"assertions\":[{}],\"diagnostics\":[{}]}}",
                json_string(DIRECT_RENEWAL_DREAM_RECEIPT_SCHEMA),
                json_string(&checkpoint_identity()),
                json_string(&std::env::var("DIRECT_RENEWAL_STAGE").unwrap_or_else(|_| "synthetic".into())),
                json_string(&std::env::var("DIRECT_RENEWAL_ROUND").unwrap_or_else(|_| "0".into())),
                json_string(receipt.owner), receipt.row_index, json_string(receipt.row_id),
                json_string(receipt.status), receipt.render_count, receipt.output_frames,
                json_string(receipt.input_hash), json_string(receipt.output_hash), assertions, diagnostics
            ).unwrap();
            file.flush().unwrap();
            file.sync_all().unwrap();
        }

        fn write_summary(owner: &str, expected_rows: usize, completed_rows: usize, expected_renders: usize, completed_renders: usize, status: &str, errors: &[String]) {
            let directory = receipt_directory(owner);
            fs::create_dir_all(&directory).unwrap();
            let mut file = OpenOptions::new().create(true).truncate(true).write(true).open(directory.join("summary.json")).unwrap();
            let errors = errors.iter().map(|error| json_string(error)).collect::<Vec<_>>().join(",");
            writeln!(file, "{{\"schema\":{},\"checkpoint\":{},\"owner\":{},\"status\":{},\"expected_rows\":{},\"complete_rows\":{},\"expected_renders\":{},\"complete_renders\":{},\"errors\":[{}]}}", json_string(DIRECT_RENEWAL_DREAM_SUMMARY_SCHEMA), json_string(&checkpoint_identity()), json_string(owner), json_string(status), expected_rows, completed_rows, expected_renders, completed_renders, errors).unwrap();
            file.flush().unwrap();
            file.sync_all().unwrap();
        }

        fn mapped_support(source: SourceKind, ratio: usize) -> (usize, usize) {
            let (start, end) = source.support();
            (start * ratio, end * ratio)
        }

        fn hard_integrity(output: &[f32], ratio: usize, channels: usize) -> Result<(), String> {
            let frames = SYNTHETIC_SOURCE_FRAMES * ratio;
            if output.len() != frames * channels { return Err("exact-length".into()); }
            if output.iter().any(|sample| !sample.is_finite()) { return Err("finite".into()); }
            if output.iter().any(|sample| sample.abs() > RENDER_SPEC.max_abs_sample) { return Err("max-abs".into()); }
            for channel in 0..channels {
                if output[channel].to_bits() != 0.0_f32.to_bits() || output[(frames - 1) * channels + channel].to_bits() != 0.0_f32.to_bits() {
                    return Err("exact-zero-endpoints".into());
                }
            }
            Ok(())
        }

        fn no_dropout(output: &[f32], source: SourceKind, ratio: usize) -> bool {
            let (start, end) = mapped_support(source, ratio);
            let window_frames = fft_size(SAMPLE_RATE) / 2;
            if end.saturating_sub(start) < window_frames { return true; }
            (start..=end - window_frames).step_by(window_frames).all(|window_start| {
                let window_end = window_start + window_frames;
                let authored_gap = source == SourceKind::SilenceGap
                    && window_start >= 42_000 * ratio
                    && window_end <= 54_000 * ratio;
                authored_gap
                    || output[window_start..window_end]
                        .iter()
                        .any(|sample| *sample != 0.0)
            })
        }

        fn rms(samples: &[f32]) -> f64 {
            if samples.is_empty() { return 0.0; }
            (samples.iter().map(|sample| (*sample as f64).powi(2)).sum::<f64>() / samples.len() as f64).sqrt()
        }

        fn difference_crest_db(samples: &[f32]) -> f64 {
            if samples.len() < 2 { return 0.0; }
            let mut maximum = 0.0_f64;
            let mut energy = 0.0_f64;
            for pair in samples.windows(2) {
                let difference = pair[1] as f64 - pair[0] as f64;
                maximum = maximum.max(difference.abs());
                energy += difference * difference;
            }
            let difference_rms = (energy / (samples.len() - 1) as f64).sqrt();
            if maximum == 0.0 { 0.0 } else { 20.0 * (maximum / difference_rms).log10() }
        }

        const DIFFERENCE_CREST_REFERENCE: [[f64; 3]; 10] = [
            [9.905726, 10.894208, 10.519063],
            [9.556341, 10.436868, 10.905863],
            [11.802457, 12.936199, 12.552822],
            [11.915677, 13.552276, 14.040544],
            [21.672820, 21.668892, 21.489501],
            [16.312956, 16.540906, 17.186745],
            [13.453803, 15.084147, 15.790229],
            [14.905336, 15.539239, 15.680264],
            [14.348783, 15.440176, 15.456703],
            [16.083822, 16.292147, 16.221062],
        ];

        fn finish_owner(owner: &str, errors: Vec<String>, rows: usize, renders: usize) -> Result<(), String> {
            write_summary(owner, rows, rows, renders, renders, if errors.is_empty() { "pass" } else { "fail" }, &errors);
            if errors.is_empty() { Ok(()) } else { Err(errors.join("; ")) }
        }
